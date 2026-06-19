mod support;

use funput_core::InputMethod;
use funput_engine::Action;

#[test]
fn vni_a1_tone() {
    let (buffer, results) = support::type_keys(InputMethod::Vni, "a1");
    assert_eq!(buffer, "á");
    assert_eq!(results[1].action, Action::Send);
    assert_eq!(results[1].backspace, 1);
    assert_eq!(results[1].output, "á");
}

#[test]
fn vni_d9_stroke() {
    let (buffer, results) = support::type_keys(InputMethod::Vni, "d9");
    assert_eq!(buffer, "đ");
    assert_eq!(results[1].action, Action::Send);
    assert_eq!(results[1].backspace, 1);
    assert_eq!(results[1].output, "đ");
}

#[test]
fn vni_a11_revert() {
    // Double tone key restores raw keystrokes: "á" + "1" → "a1".
    assert_eq!(support::type_keys_buffer(InputMethod::Vni, "a11"), "a1");
}

#[test]
fn vni_ng1_literal_ignored() {
    let (buffer, results) = support::type_keys(InputMethod::Vni, "ng1");
    assert_eq!(buffer, "ng1");
    assert_eq!(results[2].action, Action::None);
    assert_eq!(results[2].backspace, 0);
    assert!(results[2].output.is_empty());
}

#[test]
fn vni_reposition_multi_char_output() {
    // "to1an": `1` puts sắc on `o` (`tó`); the open `oa` keeps it there (`tóa`,
    // traditional rule), then the coda `n` moves it onto `a` (`toán`) — deleting
    // `óa` and injecting `oán`.
    let (buffer, results) = support::type_keys(InputMethod::Vni, "to1an");
    assert_eq!(buffer, "toán");
    let last = results.last().unwrap();
    assert_eq!(last.action, Action::Send);
    assert_eq!(last.backspace, 2);
    assert_eq!(last.output, "oán");
}
