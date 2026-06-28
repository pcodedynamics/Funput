//! Engine fixture vectors — keep in sync with `funput-core/tests/fixtures/*`.
//!
//! Buffer cases assert final engine buffer parity with core `apply()`.
//! App-text cases assert full inject simulation via `app_text` (boundaries + restore).

use funput_core::InputMethod;
use funput_engine::Action;

pub struct ExpectedStep {
    pub action: Action,
    pub backspace: usize,
    pub output: &'static str,
}

pub struct StepCase {
    pub method: InputMethod,
    pub keys: &'static str,
    pub steps: &'static [ExpectedStep],
    pub final_buffer: &'static str,
    pub label: &'static str,
}

pub struct BufferCase {
    pub method: InputMethod,
    pub keys: &'static str,
    pub output: &'static str,
    pub label: &'static str,
}

pub struct AppTextCase {
    pub method: InputMethod,
    pub keys: &'static str,
    pub output: &'static str,
    pub label: &'static str,
}

const TELEX_NONE: ExpectedStep = ExpectedStep {
    action: Action::None,
    backspace: 0,
    output: "",
};

pub const STEP_CASES: &[StepCase] = &[
    StepCase {
        method: InputMethod::Telex,
        keys: "as",
        steps: &[TELEX_NONE, ExpectedStep {
            action: Action::Send,
            backspace: 1,
            output: "á",
        }],
        final_buffer: "á",
        label: "telex tone sac",
    },
    StepCase {
        method: InputMethod::Telex,
        keys: "dd",
        steps: &[TELEX_NONE, ExpectedStep {
            action: Action::Send,
            backspace: 1,
            output: "đ",
        }],
        final_buffer: "đ",
        label: "telex stroke dd",
    },
    StepCase {
        method: InputMethod::Telex,
        keys: "ass",
        steps: &[
            TELEX_NONE,
            ExpectedStep {
                action: Action::Send,
                backspace: 1,
                output: "á",
            },
            // Double tone key restores raw keystrokes: "á" → "as".
            ExpectedStep {
                action: Action::Send,
                backspace: 1,
                output: "as",
            },
        ],
        final_buffer: "as",
        label: "telex revert ass",
    },
    StepCase {
        method: InputMethod::Telex,
        keys: "ngs",
        steps: &[TELEX_NONE, TELEX_NONE, TELEX_NONE],
        final_buffer: "ngs",
        label: "telex literal ngs",
    },
    StepCase {
        method: InputMethod::Telex,
        keys: "card ",
        steps: &[
            TELEX_NONE,
            TELEX_NONE,
            ExpectedStep {
                action: Action::Send,
                backspace: 1,
                output: "ả",
            },
            // Eager restore: the invalid coda `d` flips "cảd" back to "card" at once.
            ExpectedStep {
                action: Action::Send,
                backspace: 1,
                output: "ard",
            },
            // Already restored, so the space just passes through.
            TELEX_NONE,
        ],
        final_buffer: "",
        label: "telex eager restore card",
    },
];

pub const TELEX_BUFFER_CASES: &[BufferCase] = &[
    BufferCase {
        method: InputMethod::Telex,
        keys: "dd",
        output: "đ",
        label: "stroke dd",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "as",
        output: "á",
        label: "tone sac",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "af",
        output: "à",
        label: "tone huyen",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "aa",
        output: "â",
        label: "shape aa",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "ows",
        output: "ớ",
        label: "shape ow + tone",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "uow",
        output: "uơ",
        label: "open rhyme uow",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "hoaf",
        output: "hòa",
        label: "reposition hoaf",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "ass",
        output: "as",
        label: "revert tone ass",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "aaa",
        output: "aa",
        label: "revert shape aaa",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "ddd",
        output: "dd",
        label: "revert stroke ddd",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "truowng",
        output: "trương",
        label: "complex truowng",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "nguwowif",
        output: "người",
        label: "complex nguwowif",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "vietj",
        output: "việt",
        label: "complex vietj",
    },
    BufferCase {
        method: InputMethod::Telex,
        keys: "truwownfg",
        output: "trường",
        label: "complex truwownfg",
    },
];

pub const VNI_BUFFER_CASES: &[BufferCase] = &[
    BufferCase {
        method: InputMethod::Vni,
        keys: "d9",
        output: "đ",
        label: "stroke d9",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "D9",
        output: "Đ",
        label: "stroke uppercase D9",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a1",
        output: "á",
        label: "tone sac",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a2",
        output: "à",
        label: "tone huyen",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a3",
        output: "ả",
        label: "tone hoi",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a4",
        output: "ã",
        label: "tone nga",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a5",
        output: "ạ",
        label: "tone nang",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "ma1",
        output: "má",
        label: "tone syllable ma1",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "trung1",
        output: "trúng",
        label: "tone trung1",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "thanh1",
        output: "thánh",
        label: "tone thanh1",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "d9i5nh",
        output: "định",
        label: "stroke + tone d9i5nh",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a6",
        output: "â",
        label: "shape a6",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "o71",
        output: "ớ",
        label: "shape o7 + tone",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a61",
        output: "ấ",
        label: "shape + tone a61",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "uo7",
        output: "uơ",
        label: "open rhyme uo7",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "tru7o7ng",
        output: "trương",
        label: "shape tru7o7ng",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "hoa2",
        output: "hòa",
        label: "reposition hoa2",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "chao2",
        output: "chào",
        label: "reposition chao2",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "thuy3",
        output: "thủy",
        label: "reposition thuy3",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "mua1",
        output: "múa",
        label: "reposition mua1",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "tru7o7n2g",
        output: "trường",
        label: "reposition tru7o7n2g",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a11",
        output: "a1",
        label: "revert tone a11",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "a66",
        output: "a6",
        label: "revert shape a66",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "d99",
        output: "d9",
        label: "revert stroke d99",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "hoa22",
        output: "hoa2",
        label: "revert reposition hoa22",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "tru7o7n2g2",
        output: "trương2",
        label: "revert tru7o7n2g2",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "ngu71",
        output: "ngứ",
        label: "complex ngu71",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "truo7ng",
        output: "trương",
        label: "complex truo7ng",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "ngu7o7i2",
        output: "người",
        label: "complex ngu7o7i2",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "vie5t",
        output: "việt",
        label: "complex vie5t",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "nghia4",
        output: "nghĩa",
        label: "complex nghia4",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "phuo7ng",
        output: "phương",
        label: "complex phuo7ng",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "nghie5ng",
        output: "nghiệng",
        label: "complex nghie5ng",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "nuoc71",
        output: "nước",
        label: "complex nuoc71",
    },
    BufferCase {
        method: InputMethod::Vni,
        keys: "ab",
        output: "ab",
        label: "normal append ab",
    },
];

pub const APP_TEXT_CASES: &[AppTextCase] = &[
    AppTextCase {
        method: InputMethod::Telex,
        keys: "xins chaof banj ",
        output: "xín chào bạn ",
        label: "telex multi-word greeting",
    },
    AppTextCase {
        method: InputMethod::Vni,
        keys: "xin1 dong2 ",
        output: "xín dòng ",
        label: "vni multi-word xin1 dong2",
    },
    AppTextCase {
        method: InputMethod::Vni,
        keys: "xin1 chao2 ban5 ",
        output: "xín chào bạn ",
        label: "vni multi-word xin1 chao2 ban5",
    },
    AppTextCase {
        method: InputMethod::Vni,
        keys: "trung1 quoc61 ban5 nuoc71 ",
        output: "trúng quốc bạn nước ",
        label: "vni multi-word real words",
    },
    AppTextCase {
        method: InputMethod::Telex,
        keys: "card ",
        output: "card ",
        label: "english restore card",
    },
    AppTextCase {
        method: InputMethod::Telex,
        keys: "cool ",
        output: "cool ",
        label: "english restore cool",
    },
    AppTextCase {
        method: InputMethod::Telex,
        keys: "mas ",
        output: "má ",
        label: "valid vn mas keeps composed",
    },
    AppTextCase {
        method: InputMethod::Telex,
        keys: "card mas ",
        output: "card má ",
        label: "mixed english and vietnamese",
    },
];
