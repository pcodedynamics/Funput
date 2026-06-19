//! End-to-end remove-tone key: VNI `0` and Telex `z` strip the tone mark.

mod support;

use funput_core::InputMethod;
use funput_engine::Action;

#[test]
fn vni_zero_removes_tone() {
    let (buffer, results) = support::type_keys(InputMethod::Vni, "a1");
    assert_eq!(buffer, "á");

    // Type "a10": the trailing 0 deletes the tone, sending plain "a".
    let (buffer, results2) = support::type_keys(InputMethod::Vni, "a10");
    assert_eq!(buffer, "a");
    let last = results2.last().unwrap();
    assert_eq!(last.action, Action::Send);
    assert_eq!(last.backspace, 1);
    assert_eq!(last.output, "a");
    let _ = results;
}

#[test]
fn vni_zero_keeps_shape() {
    // â (a6) + sắc (1) + remove (0) → â, circumflex preserved.
    let (buffer, _) = support::type_keys(InputMethod::Vni, "a610");
    assert_eq!(buffer, "â");
}

#[test]
fn telex_z_removes_tone() {
    let (buffer, _) = support::type_keys(InputMethod::Telex, "asz");
    assert_eq!(buffer, "a");

    // việt → viêt (tone gone, ê kept).
    let (buffer, _) = support::type_keys(InputMethod::Telex, "vieetjz");
    assert_eq!(buffer, "viêt");
}

#[test]
fn remove_tone_with_no_tone_is_literal() {
    let (buffer, _) = support::type_keys(InputMethod::Vni, "a0");
    assert_eq!(buffer, "a0");
}
