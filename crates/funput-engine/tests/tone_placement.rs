//! End-to-end tone placement: `k`+`y` words and triphthong (kiểu truyền thống).

use funput_engine::Engine;
use funput_core::InputMethod;
fn run(m: InputMethod, keys: &str) -> String {
    let mut e = Engine::new(); e.set_method(m);
    for k in keys.chars() { e.process_char(k); }
    e.buffer().to_string()
}
#[test]
fn traditional_tone_placement_and_ky_words() {
    assert_eq!(run(InputMethod::Vni, "ky2"), "kỳ");
    assert_eq!(run(InputMethod::Telex, "kyf"), "kỳ");
    assert_eq!(run(InputMethod::Telex, "ngoaif"), "ngoài");
    assert_eq!(run(InputMethod::Vni, "ngoai2"), "ngoài");
    assert_eq!(run(InputMethod::Telex, "kyx"), "kỹ"); // kỹ
}
