//! Emit an [`InjectPlan`] to the focused app: Backspace presses, then Unicode
//! characters, via `SendInput`. Every synthesized event carries [`INJECT_TAG`] in
//! `dwExtraInfo` so the hook ignores them (no re-entrancy).

use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use funput_desktop::InjectPlan;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_BACK,
};

use crate::shell::INJECT_TAG;

/// Real-time gap inserted between the Backspace batch and the text batch on the
/// staged (browser) path. Chrome's omnibox runs inline autocomplete and processes
/// input asynchronously; firing the deletions and the replacement text in one
/// `SendInput` burst lets a Backspace land on the autocomplete *selection* instead
/// of the base character, so a stray vowel survives ("bộ" → "boộ"). Spacing the two
/// batches in wall-clock time lets the omnibox settle the deletion first. Kept small
/// so typing latency stays imperceptible; tune if some Chrome builds still slip.
const STAGED_GAP: Duration = Duration::from_millis(12);

fn vk_event(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    let dw_flags = if up {
        KEYEVENTF_KEYUP
    } else {
        KEYBD_EVENT_FLAGS(0)
    };
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: dw_flags,
                time: 0,
                dwExtraInfo: INJECT_TAG,
            },
        },
    }
}

fn unicode_event(unit: u16, up: bool) -> INPUT {
    let mut dw_flags = KEYEVENTF_UNICODE;
    if up {
        dw_flags |= KEYEVENTF_KEYUP;
    }
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: unit,
                dwFlags: dw_flags,
                time: 0,
                dwExtraInfo: INJECT_TAG,
            },
        },
    }
}

/// Backspace key presses (down+up) for `n` deletions.
fn deletions(n: usize) -> Vec<INPUT> {
    let mut v = Vec::with_capacity(n * 2);
    for _ in 0..n {
        v.push(vk_event(VK_BACK, false));
        v.push(vk_event(VK_BACK, true));
    }
    v
}

/// Unicode key presses (down+up) for the composed `units`.
fn text(units: &[u16]) -> Vec<INPUT> {
    let mut v = Vec::with_capacity(units.len() * 2);
    for &unit in units {
        v.push(unicode_event(unit, false));
        v.push(unicode_event(unit, true));
    }
    v
}

fn raw_send(inputs: &[INPUT]) {
    if inputs.is_empty() {
        return;
    }
    unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32) };
}

/// Send the deletions then the new text as one atomic `SendInput` batch. This is
/// the default path used for every app except Chrome (see [`send_plan_staged`]).
pub fn send_plan(plan: &InjectPlan) {
    if plan.is_noop() {
        return;
    }
    let mut inputs = deletions(plan.backspaces);
    inputs.extend(text(&plan.units));
    raw_send(&inputs);
}

/// Like [`send_plan`] but fired through a dedicated worker thread that splits the
/// deletions and the text into two `SendInput` calls separated by [`STAGED_GAP`].
///
/// The split **must** run off the hook thread: the LL keyboard hook only advances
/// when its thread pumps messages, and our injected Backspaces re-enter that hook
/// (they carry [`INJECT_TAG`] and are dropped, but still gate delivery). Sleeping
/// inside the hook callback would stall those very Backspaces, so no real gap would
/// reach Chrome. The worker keeps the hook thread free to pump, so the two batches
/// arrive at the omnibox spaced apart in wall-clock time. Plans are queued FIFO, so
/// fast typing stays in order.
pub fn send_plan_staged(plan: &InjectPlan) {
    if plan.is_noop() {
        return;
    }
    // Every Chrome plan goes through the same queue (even ones with no deletions) so
    // the worker preserves strict FIFO order; mixing a synchronous path here could
    // let a later plan overtake one still queued. `Sender` is `!Sync`, so the static
    // holds it behind a `Mutex`; the lock is uncontended (only the hook enqueues).
    let _ = worker().lock().expect("inject worker mutex poisoned").send(plan.clone());
}

/// Lazily-spawned worker that serializes staged injections.
fn worker() -> &'static Mutex<Sender<InjectPlan>> {
    static WORKER: OnceLock<Mutex<Sender<InjectPlan>>> = OnceLock::new();
    WORKER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<InjectPlan>();
        std::thread::spawn(move || {
            while let Ok(plan) = rx.recv() {
                if plan.backspaces > 0 {
                    raw_send(&deletions(plan.backspaces));
                    std::thread::sleep(STAGED_GAP); // let the omnibox settle the delete
                }
                raw_send(&text(&plan.units));
            }
        });
        Mutex::new(tx)
    })
}
