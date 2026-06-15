mod fixtures {
    pub mod step_cases;
}
mod support;

use fixtures::step_cases::{
    AppTextCase, BufferCase, StepCase, APP_TEXT_CASES, STEP_CASES, TELEX_BUFFER_CASES,
    VNI_BUFFER_CASES,
};
use support::{app_text, type_keys_buffer, type_keys_with_results};

fn assert_steps(case: &StepCase) {
    let (buffer, results) = type_keys_with_results(case.method, case.keys);
    assert_eq!(
        buffer, case.final_buffer,
        "{}: final buffer",
        case.label
    );
    assert_eq!(
        results.len(),
        case.steps.len(),
        "{}: step count",
        case.label
    );
    for (i, (got, expected)) in results.iter().zip(case.steps.iter()).enumerate() {
        assert_eq!(got.action, expected.action, "{}: step {i} action", case.label);
        assert_eq!(
            got.backspace, expected.backspace,
            "{}: step {i} backspace",
            case.label
        );
        assert_eq!(got.output, expected.output, "{}: step {i} output", case.label);
    }
}

fn assert_buffer(case: &BufferCase) {
    assert_eq!(
        type_keys_buffer(case.method, case.keys),
        case.output,
        "{}",
        case.label
    );
}

fn assert_app_text(case: &AppTextCase) {
    assert_eq!(
        app_text(case.method, case.keys),
        case.output,
        "{}",
        case.label
    );
}

#[test]
fn step_fixture_cases() {
    for case in STEP_CASES {
        assert_steps(case);
    }
}

#[test]
fn telex_buffer_fixture_cases() {
    for case in TELEX_BUFFER_CASES {
        assert_buffer(case);
    }
}

#[test]
fn vni_buffer_fixture_cases() {
    for case in VNI_BUFFER_CASES {
        assert_buffer(case);
    }
}

#[test]
fn app_text_fixture_cases() {
    for case in APP_TEXT_CASES {
        assert_app_text(case);
    }
}

#[test]
fn engine_full_regression() {
    step_fixture_cases();
    telex_buffer_fixture_cases();
    vni_buffer_fixture_cases();
    app_text_fixture_cases();
}
