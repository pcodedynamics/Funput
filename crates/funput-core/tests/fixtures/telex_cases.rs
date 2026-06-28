//! Canonical Telex fixture vectors (Rust const — single source of truth for regression).

pub struct TelexCase {
    pub keys: &'static str,
    pub output: &'static str,
    pub label: &'static str,
}

pub struct TelexWordCase {
    pub words: &'static str,
    pub output: &'static str,
    pub label: &'static str,
}

pub const CASES: &[TelexCase] = &[
    TelexCase {
        keys: "dd",
        output: "đ",
        label: "stroke dd",
    },
    TelexCase {
        keys: "as",
        output: "á",
        label: "tone sac",
    },
    TelexCase {
        keys: "af",
        output: "à",
        label: "tone huyen",
    },
    TelexCase {
        keys: "aa",
        output: "â",
        label: "shape aa",
    },
    TelexCase {
        keys: "ows",
        output: "ớ",
        label: "shape ow + tone",
    },
    TelexCase {
        keys: "uow",
        output: "uơ",
        label: "open rhyme uow",
    },
    TelexCase {
        keys: "hoaf",
        output: "hòa",
        label: "reposition hoaf",
    },
    TelexCase {
        keys: "ass",
        output: "as",
        label: "revert tone ass",
    },
    TelexCase {
        keys: "aaa",
        output: "aa",
        label: "revert shape aaa",
    },
    TelexCase {
        keys: "ddd",
        output: "dd",
        label: "revert stroke ddd",
    },
    TelexCase {
        keys: "truowng",
        output: "trương",
        label: "complex truowng",
    },
    TelexCase {
        keys: "nguwowif",
        output: "người",
        label: "complex nguwowif",
    },
    TelexCase {
        keys: "vietj",
        output: "việt",
        label: "complex vietj",
    },
    TelexCase {
        keys: "truwownfg",
        output: "trường",
        label: "complex truwownfg",
    },
];

pub const WORD_CASES: &[TelexWordCase] = &[
    TelexWordCase {
        words: "xins chaof banj",
        output: "xín chào bạn",
        label: "multi-word greeting",
    },
];
