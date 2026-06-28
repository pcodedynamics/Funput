mod support;

use funput_core::{apply, InputMethod, ToneStyle, TransformKind, TransformResult};

fn type_keys(keys: &str) -> String {
    support::type_keys(InputMethod::Telex, keys)
}

#[test]
fn telex_stroke_and_tone_basics() {
    assert_eq!(type_keys("dd"), "đ");
    assert_eq!(type_keys("DD"), "Đ");
    assert_eq!(type_keys("as"), "á");
    assert_eq!(type_keys("af"), "à");
    assert_eq!(type_keys("ar"), "ả");
    assert_eq!(type_keys("ax"), "ã");
    assert_eq!(type_keys("aj"), "ạ");
    assert_eq!(type_keys("mas"), "má");
}

#[test]
fn telex_shape_basics() {
    assert_eq!(type_keys("aa"), "â");
    assert_eq!(type_keys("ee"), "ê");
    assert_eq!(type_keys("oo"), "ô");
    assert_eq!(type_keys("ow"), "ơ");
    assert_eq!(type_keys("uw"), "ư");
    assert_eq!(type_keys("aw"), "ă");
    assert_eq!(type_keys("uow"), "uơ");
    assert_eq!(type_keys("uowr"), "uở");
    assert_eq!(type_keys("thuowr"), "thuở");
    assert_eq!(type_keys("thuowngf"), "thường");
    assert_eq!(type_keys("quowis"), "quới");
}

#[test]
fn telex_shape_then_tone() {
    assert_eq!(type_keys("oos"), "ố");
    assert_eq!(type_keys("aas"), "ấ");
}

#[test]
fn telex_reposition() {
    assert_eq!(type_keys("hoaf"), "hòa");
    assert_eq!(type_keys("thuyr"), "thủy");
}

#[test]
fn telex_revert() {
    // Double modifier restores raw keystrokes: strip diacritic + append the key.
    assert_eq!(type_keys("ass"), "as");
    assert_eq!(type_keys("aaa"), "aa");
    assert_eq!(type_keys("ddd"), "dd");
    assert_eq!(type_keys("aas"), "ấ"); // single sắc on â — not a revert
    assert_eq!(type_keys("aass"), "âs");
    assert_eq!(type_keys("hoaff"), "hoaf");
}

#[test]
fn telex_multi_syllable_words() {
    assert_eq!(
        support::type_words(InputMethod::Telex, "xins chaof banj"),
        "xín chào bạn"
    );
}
#[test]
fn telex_complex_syllables() {
    assert_eq!(type_keys("truowng"), "trương");
    assert_eq!(type_keys("nguwowif"), "người");
    assert_eq!(type_keys("vietj"), "việt");
    assert_eq!(type_keys("truwownfg"), "trường");
    assert_eq!(type_keys("nuocws"), "nước");
}

#[test]
fn telex_free_position_marks() {
    // Marks can be typed anywhere in the syllable, not only adjacent to their
    // target — the user may place the dấu at any position.
    // Breve typed after the coda: "lamws" → lắm (a→ă via w, then sắc via s).
    assert_eq!(type_keys("lamws"), "lắm");
    // Conventional order still works: "lawms" → lắm.
    assert_eq!(type_keys("lawms"), "lắm");
    // Stroke đ typed after the whole rhyme: "duocwjd" → được.
    assert_eq!(type_keys("duocwjd"), "được");
    assert_eq!(type_keys("dduocwj"), "được"); // đ first — unchanged
    // Horn after the coda.
    assert_eq!(type_keys("conw"), "cơn");
    assert_eq!(type_keys("anw"), "ăn");
}

#[test]
fn telex_validation_and_pass_through() {
    // A tone letter with no vowel to land on is kept literally, not dropped.
    assert_eq!(
        apply("ng", 's', InputMethod::Telex, ToneStyle::Traditional),
        TransformResult {
            kind: TransformKind::Pending,
            text: "ngs".into(),
        }
    );
    assert_eq!(
        apply("text", 's', InputMethod::Telex, ToneStyle::Traditional),
        TransformResult {
            kind: TransformKind::Pending,
            text: "texts".into(),
        }
    );
    // Leading `f`/`j` and English words keep every keystroke (engine restores).
    assert_eq!(type_keys("file"), "file");
    assert_eq!(type_keys("from"), "from");
    assert_eq!(type_keys("just"), "just");
    assert_eq!(
        apply("a", 'b', InputMethod::Telex, ToneStyle::Traditional),
        TransformResult {
            kind: TransformKind::Pending,
            text: "ab".into(),
        }
    );
}
