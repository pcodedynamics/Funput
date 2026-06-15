//! VNI ↔ Telex parity vectors — both key sequences must produce the same output.

pub struct ParityCase {
    pub telex_keys: &'static str,
    pub vni_keys: &'static str,
    pub label: &'static str,
}

pub struct ParityWordCase {
    pub telex_words: &'static str,
    pub vni_words: &'static str,
    pub output: &'static str,
    pub label: &'static str,
}

pub const CASES: &[ParityCase] = &[
    ParityCase {
        telex_keys: "dd",
        vni_keys: "d9",
        label: "stroke",
    },
    ParityCase {
        telex_keys: "DD",
        vni_keys: "D9",
        label: "stroke uppercase",
    },
    ParityCase {
        telex_keys: "as",
        vni_keys: "a1",
        label: "tone sac",
    },
    ParityCase {
        telex_keys: "af",
        vni_keys: "a2",
        label: "tone huyen",
    },
    ParityCase {
        telex_keys: "ar",
        vni_keys: "a3",
        label: "tone hoi",
    },
    ParityCase {
        telex_keys: "ax",
        vni_keys: "a4",
        label: "tone nga",
    },
    ParityCase {
        telex_keys: "aj",
        vni_keys: "a5",
        label: "tone nang",
    },
    ParityCase {
        telex_keys: "mas",
        vni_keys: "ma1",
        label: "tone syllable",
    },
    ParityCase {
        telex_keys: "trungs",
        vni_keys: "trung1",
        label: "tone trung",
    },
    ParityCase {
        telex_keys: "thanhs",
        vni_keys: "thanh1",
        label: "tone thanh",
    },
    ParityCase {
        telex_keys: "ddinjh",
        vni_keys: "d9i5nh",
        label: "stroke + tone",
    },
    ParityCase {
        telex_keys: "aa",
        vni_keys: "a6",
        label: "shape aa",
    },
    ParityCase {
        telex_keys: "ows",
        vni_keys: "o71",
        label: "shape ow + tone",
    },
    ParityCase {
        telex_keys: "aas",
        vni_keys: "a61",
        label: "shape + tone",
    },
    ParityCase {
        telex_keys: "uow",
        vni_keys: "uo7",
        label: "shape uow",
    },
    ParityCase {
        telex_keys: "truwowng",
        vni_keys: "tru7o7ng",
        label: "shape truwowng",
    },
    ParityCase {
        telex_keys: "hoaf",
        vni_keys: "hoa2",
        label: "reposition hoaf",
    },
    ParityCase {
        telex_keys: "chaof",
        vni_keys: "chao2",
        label: "reposition chaof",
    },
    ParityCase {
        telex_keys: "thuyr",
        vni_keys: "thuy3",
        label: "reposition thuyr",
    },
    ParityCase {
        telex_keys: "muas",
        vni_keys: "mua1",
        label: "reposition muas",
    },
    ParityCase {
        telex_keys: "truwownfg",
        vni_keys: "tru7o7n2g",
        label: "reposition truwownfg",
    },
    ParityCase {
        telex_keys: "ass",
        vni_keys: "a11",
        label: "revert tone",
    },
    ParityCase {
        telex_keys: "aaa",
        vni_keys: "a66",
        label: "revert shape",
    },
    ParityCase {
        telex_keys: "ddd",
        vni_keys: "d99",
        label: "revert stroke",
    },
    ParityCase {
        telex_keys: "hoaff",
        vni_keys: "hoa22",
        label: "revert reposition",
    },
    ParityCase {
        telex_keys: "truwownfgf",
        vni_keys: "tru7o7n2g2",
        label: "revert truwownfg",
    },
    ParityCase {
        telex_keys: "nguws",
        vni_keys: "ngu71",
        label: "complex nguws",
    },
    ParityCase {
        telex_keys: "truowng",
        vni_keys: "truo7ng",
        label: "complex truowng",
    },
    ParityCase {
        telex_keys: "nguwowif",
        vni_keys: "ngu7o7i2",
        label: "complex nguwowif",
    },
    ParityCase {
        telex_keys: "vietj",
        vni_keys: "vie5t",
        label: "complex vietj",
    },
    ParityCase {
        telex_keys: "nghiax",
        vni_keys: "nghia4",
        label: "complex nghiax",
    },
    ParityCase {
        telex_keys: "phuwowng",
        vni_keys: "phuo7ng",
        label: "complex phuwowng",
    },
    ParityCase {
        telex_keys: "nghiejng",
        vni_keys: "nghie5ng",
        label: "complex nghiejng",
    },
    ParityCase {
        telex_keys: "nuocws",
        vni_keys: "nuoc71",
        label: "complex nuocws",
    },
    ParityCase {
        telex_keys: "ab",
        vni_keys: "ab",
        label: "normal append",
    },
];

pub const WORD_CASES: &[ParityWordCase] = &[
    ParityWordCase {
        telex_words: "xins dongf",
        vni_words: "xin1 dong2",
        output: "xín dòng",
        label: "multi-word xins dongf",
    },
    ParityWordCase {
        telex_words: "xins chaof banj",
        vni_words: "xin1 chao2 ban5",
        output: "xín chào bạn",
        label: "multi-word greeting",
    },
    ParityWordCase {
        telex_words: "trungs quocs banj nuocs",
        vni_words: "trung1 quoc1 ban5 nuoc1",
        output: "trúng quóc bạn nuóc",
        label: "multi-word trungs quocs",
    },
];
