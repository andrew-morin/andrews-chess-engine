pub const MAILBOX: [Option<usize>; 120] = [
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    Some(0),
    Some(1),
    Some(2),
    Some(3),
    Some(4),
    Some(5),
    Some(6),
    Some(7),
    None,
    None,
    Some(8),
    Some(9),
    Some(10),
    Some(11),
    Some(12),
    Some(13),
    Some(14),
    Some(15),
    None,
    None,
    Some(16),
    Some(17),
    Some(18),
    Some(19),
    Some(20),
    Some(21),
    Some(22),
    Some(23),
    None,
    None,
    Some(24),
    Some(25),
    Some(26),
    Some(27),
    Some(28),
    Some(29),
    Some(30),
    Some(31),
    None,
    None,
    Some(32),
    Some(33),
    Some(34),
    Some(35),
    Some(36),
    Some(37),
    Some(38),
    Some(39),
    None,
    None,
    Some(40),
    Some(41),
    Some(42),
    Some(43),
    Some(44),
    Some(45),
    Some(46),
    Some(47),
    None,
    None,
    Some(48),
    Some(49),
    Some(50),
    Some(51),
    Some(52),
    Some(53),
    Some(54),
    Some(55),
    None,
    None,
    Some(56),
    Some(57),
    Some(58),
    Some(59),
    Some(60),
    Some(61),
    Some(62),
    Some(63),
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
    None,
];

pub const BOARD_INDEX_TO_MAILBOX_INDEX: [usize; 64] = [
    21, 22, 23, 24, 25, 26, 27, 28, 31, 32, 33, 34, 35, 36, 37, 38, 41, 42, 43, 44, 45, 46, 47, 48,
    51, 52, 53, 54, 55, 56, 57, 58, 61, 62, 63, 64, 65, 66, 67, 68, 71, 72, 73, 74, 75, 76, 77, 78,
    81, 82, 83, 84, 85, 86, 87, 88, 91, 92, 93, 94, 95, 96, 97, 98,
];

pub const CARDINAL_MAILBOX_DIRECTION_OFFSETS: [usize; 2] = [1, 10];
pub const DIAGONAL_MAILBOX_DIRECTION_OFFSETS: [usize; 2] = [9, 11];
pub const ALL_MAILBOX_DIRECTION_OFFSETS: [usize; 4] = [1, 9, 10, 11];
pub const KNIGHT_MAILBOX_DIRECTION_OFFSETS: [usize; 4] = [8, 12, 19, 21];

pub static KING_ATTACK_BITMASKS: [u64; 64] =
    build_hop_attack_bitmasks(ALL_MAILBOX_DIRECTION_OFFSETS);
pub static KNIGHT_ATTACK_BITMASKS: [u64; 64] =
    build_hop_attack_bitmasks(KNIGHT_MAILBOX_DIRECTION_OFFSETS);
pub static CARDINAL_ATTACK_BITMASKS: [[u64; 4]; 64] =
    build_slide_attack_bitmasks(CARDINAL_MAILBOX_DIRECTION_OFFSETS);
pub static DIAGONAL_ATTACK_BITMASKS: [[u64; 4]; 64] =
    build_slide_attack_bitmasks(DIAGONAL_MAILBOX_DIRECTION_OFFSETS);

const fn build_hop_attack_bitmasks(offset_directions: [usize; 4]) -> [u64; 64] {
    let mut bitmasks: [u64; 64] = [0; 64];
    let mut index = 0;
    while index < 64 {
        let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
        let mut bitmask: u64 = 0;
        let mut offset_index = 0;
        while offset_index < 4 {
            let offset = offset_directions[offset_index];
            let attack_index = MAILBOX[mailbox_index + offset];
            if let Some(attack_index) = attack_index {
                bitmask |= 1 << attack_index;
            }
            let attack_index = MAILBOX[mailbox_index - offset];
            if let Some(attack_index) = attack_index {
                bitmask |= 1 << attack_index;
            }
            offset_index += 1;
        }
        bitmasks[index] = bitmask;
        index += 1;
    }
    bitmasks
}

const fn build_slide_attack_bitmasks(offset_directions: [usize; 2]) -> [[u64; 4]; 64] {
    let mut bitmasks: [[u64; 4]; 64] = [[0; 4]; 64];
    let mut index = 0;
    while index < 64 {
        let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
        let mut direction_bitmasks: [u64; 4] = [0; 4];
        let mut offset_index = 0;
        while offset_index < 2 {
            let offset = offset_directions[offset_index];
            let mut attack_mailbox_index = mailbox_index - offset;
            loop {
                let attack_index = MAILBOX[attack_mailbox_index];
                if let Some(attack_index) = attack_index {
                    direction_bitmasks[offset_index] |= 1 << attack_index;
                    attack_mailbox_index -= offset;
                } else {
                    break;
                }
            }
            let mut attack_mailbox_index = mailbox_index + offset;
            loop {
                let attack_index = MAILBOX[attack_mailbox_index];
                if let Some(attack_index) = attack_index {
                    direction_bitmasks[offset_index + 2] |= 1 << attack_index;
                    attack_mailbox_index += offset;
                } else {
                    break;
                }
            }
            offset_index += 1;
        }
        bitmasks[index] = direction_bitmasks;
        index += 1;
    }
    bitmasks
}
