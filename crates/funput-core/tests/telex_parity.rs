mod fixtures {
    pub mod telex_parity;
}
mod support;

use funput_core::InputMethod;

#[test]
fn telex_matches_vni_output() {
    for case in fixtures::telex_parity::CASES {
        let telex = support::type_keys(InputMethod::Telex, case.telex_keys);
        let vni = support::type_keys(InputMethod::Vni, case.vni_keys);
        assert_eq!(telex, vni, "{}: telex vs vni output", case.label);
    }
}

#[test]
fn telex_word_parity() {
    for case in fixtures::telex_parity::WORD_CASES {
        let telex = support::type_words(InputMethod::Telex, case.telex_words);
        let vni = support::type_words(InputMethod::Vni, case.vni_words);
        assert_eq!(telex, vni, "{}: telex vs vni words", case.label);
        assert_eq!(telex, case.output, "{}: telex output", case.label);
    }
}
