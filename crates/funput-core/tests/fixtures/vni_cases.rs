//! Canonical VNI fixture vectors (Rust const — single source of truth for regression).

pub struct VniCase {
    pub keys: &'static str,
    pub output: &'static str,
    pub label: &'static str,
}

pub struct VniWordCase {
    pub words: &'static str,
    pub output: &'static str,
    pub label: &'static str,
}

pub const CASES: &[VniCase] = &[
    // Stroke
    VniCase {
        keys: "d9",
        output: "đ",
        label: "stroke d9",
    },
    VniCase {
        keys: "D9",
        output: "Đ",
        label: "stroke uppercase D9",
    },
    // Tone — single vowel
    VniCase {
        keys: "a1",
        output: "á",
        label: "tone sac",
    },
    VniCase {
        keys: "a2",
        output: "à",
        label: "tone huyen",
    },
    VniCase {
        keys: "a3",
        output: "ả",
        label: "tone hoi",
    },
    VniCase {
        keys: "a4",
        output: "ã",
        label: "tone nga",
    },
    VniCase {
        keys: "a5",
        output: "ạ",
        label: "tone nang",
    },
    // Tone — syllables
    VniCase {
        keys: "ma1",
        output: "má",
        label: "tone syllable ma1",
    },
    VniCase {
        keys: "trung1",
        output: "trúng",
        label: "tone trung1",
    },
    VniCase {
        keys: "thanh1",
        output: "thánh",
        label: "tone thanh1",
    },
    VniCase {
        keys: "d9i5nh",
        output: "định",
        label: "stroke + tone d9i5nh",
    },
    // Shape
    VniCase {
        keys: "a6",
        output: "â",
        label: "shape a6",
    },
    VniCase {
        keys: "o71",
        output: "ớ",
        label: "shape o7 + tone",
    },
    VniCase {
        keys: "a61",
        output: "ấ",
        label: "shape + tone a61",
    },
    VniCase {
        keys: "uo7",
        output: "uơ",
        label: "open rhyme uo7",
    },
    VniCase {
        keys: "tru7o7ng",
        output: "trương",
        label: "shape tru7o7ng",
    },
    // Reposition
    VniCase {
        keys: "hoa2",
        output: "hòa",
        label: "reposition hoa2",
    },
    VniCase {
        keys: "chao2",
        output: "chào",
        label: "reposition chao2",
    },
    VniCase {
        keys: "thuy3",
        output: "thủy",
        label: "reposition thuy3",
    },
    VniCase {
        keys: "mua1",
        output: "múa",
        label: "reposition mua1",
    },
    VniCase {
        keys: "tru7o7n2g",
        output: "trường",
        label: "reposition tru7o7n2g",
    },
    // Revert — double modifier restores raw keystrokes (strip diacritic + append key)
    VniCase {
        keys: "a11",
        output: "a1",
        label: "revert tone a11",
    },
    VniCase {
        keys: "a66",
        output: "a6",
        label: "revert shape a66",
    },
    VniCase {
        keys: "d99",
        output: "d9",
        label: "revert stroke d99",
    },
    VniCase {
        keys: "hoa22",
        output: "hoa2",
        label: "revert reposition hoa22",
    },
    VniCase {
        keys: "tru7o7n2g2",
        output: "trương2",
        label: "revert tru7o7n2g2",
    },
    // Complex syllables (Phase 6)
    VniCase {
        keys: "ngu71",
        output: "ngứ",
        label: "complex ngu71",
    },
    VniCase {
        keys: "truo7ng",
        output: "trương",
        label: "complex truo7ng",
    },
    VniCase {
        keys: "ngu7o7i2",
        output: "người",
        label: "complex ngu7o7i2",
    },
    VniCase {
        keys: "vie5t",
        output: "việt",
        label: "complex vie5t",
    },
    VniCase {
        keys: "nghia4",
        output: "nghĩa",
        label: "complex nghia4",
    },
    VniCase {
        keys: "phuo7ng",
        output: "phương",
        label: "complex phuo7ng",
    },
    VniCase {
        keys: "nghie5ng",
        output: "nghiệng",
        label: "complex nghie5ng",
    },
    VniCase {
        keys: "nuoc71",
        output: "nước",
        label: "complex nuoc71",
    },
    // Validation / pass-through
    VniCase {
        keys: "ab",
        output: "ab",
        label: "normal append ab",
    },
];

pub const WORD_CASES: &[VniWordCase] = &[
    VniWordCase {
        words: "xin1 dong2",
        output: "xín dòng",
        label: "multi-word xin1 dong2",
    },
    VniWordCase {
        words: "xin1 chao2 ban5",
        output: "xín chào bạn",
        label: "multi-word xin1 chao2 ban5",
    },
    VniWordCase {
        words: "trung1 quoc1 ban5 nuoc1",
        output: "trúng quóc bạn nuóc",
        label: "multi-word trung1 quoc1",
    },
];
