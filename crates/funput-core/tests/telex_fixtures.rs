mod fixtures {
    pub mod telex_cases;
}
mod support;

use funput_core::InputMethod;

#[test]
fn telex_fixture_cases() {
    for case in fixtures::telex_cases::CASES {
        assert_eq!(
            support::type_keys(InputMethod::Telex, case.keys),
            case.output,
            "{}",
            case.label
        );
    }
}

#[test]
fn telex_fixture_word_cases() {
    for case in fixtures::telex_cases::WORD_CASES {
        assert_eq!(
            support::type_words(InputMethod::Telex, case.words),
            case.output,
            "{}",
            case.label
        );
    }
}

#[test]
fn telex_full_regression() {
    telex_fixture_cases();
    telex_fixture_word_cases();
}
