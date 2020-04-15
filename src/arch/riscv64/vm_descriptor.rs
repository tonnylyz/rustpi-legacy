use register::*;

register_bitfields! {u64,
    pub TABLE_DESCRIPTOR [
        NEXT_LEVEL_TABLE_PPN OFFSET(10) NUMBITS(44) [], // [53:10]

        DIRTY  OFFSET(7) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        ACCESSED  OFFSET(6) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        GLOBAL  OFFSET(5) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        USER  OFFSET(4) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        VALID OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}

register_bitfields! {u64,
    pub PAGE_DESCRIPTOR [
        OUTPUT_PPN OFFSET(10) NUMBITS(44) [], // [53:10]
    // Note: LIB and COW are software-defined bits
        LIB      OFFSET(9) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        COW      OFFSET(8) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        DIRTY  OFFSET(7) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        ACCESSED  OFFSET(6) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        GLOBAL  OFFSET(5) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        USER  OFFSET(4) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        X    OFFSET(3) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        W    OFFSET(2) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        R    OFFSET(1) NUMBITS(1) [
            False = 0,
            True = 1
        ],
        VALID    OFFSET(0) NUMBITS(1) [
            False = 0,
            True = 1
        ]
    ]
}
