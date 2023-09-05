pub struct MagicBitboard {
    // masks
    pub bishop_masks: [u64; 64],
    pub rook_masks: [u64; 64],
    // attacks
    pub bishop_attacks: Vec<[u64; 512]>,
    pub rook_attacks: Vec<[u64; 4096]>,
}
impl MagicBitboard {
    pub fn get_bishop_attacks(&self, square_index: usize, occupancy: u64) -> u64 {
        let mut occupancy = occupancy;
        occupancy &= self.bishop_masks[square_index];
        occupancy = occupancy.wrapping_mul(BISHOP_MAGIC_NUMBERS[square_index]);
        occupancy >>= 64 - BISHOP_RELEVANT_BITS[square_index];

        self.bishop_attacks[square_index][occupancy as usize]
    }

    pub fn get_rook_attacks(&self, square_index: usize, occupancy: u64) -> u64 {
        let mut occupancy = occupancy;
        occupancy &= self.rook_masks[square_index];
        occupancy = occupancy.wrapping_mul(ROOK_MAGIC_NUMBERS[square_index]);
        occupancy >>= 64 - ROOK_RELEVANT_BITS[square_index];

        self.rook_attacks[square_index][occupancy as usize]
    }

    pub fn init() -> MagicBitboard {
        let mut bishop_masks: [u64; 64] = [0u64; 64];
        let mut rook_masks: [u64; 64] = [0u64; 64];
        let mut bishop_attacks = vec![[0u64; 512]; 64];
        let mut rook_attacks = vec![[0u64; 4096]; 64];
        for square in 0..64 {
            let bishop_mask = mask_bishop_attacks(square);
            let rook_mask = mask_rook_attacks(square);

            bishop_masks[square] = bishop_mask;
            rook_masks[square] = rook_mask;

            let bit_count = bishop_mask.count_ones();

            let occupancy_variations = 1 << bit_count;

            for count in 0..occupancy_variations {
                let occupancy: u64 = set_occupancy(count, bit_count, bishop_mask);
                let magic_index: u64 = occupancy.wrapping_mul(BISHOP_MAGIC_NUMBERS[square])
                    >> 64 - BISHOP_RELEVANT_BITS[square];
                bishop_attacks[square as usize][magic_index as usize] =
                    bishop_attacks_on_the_fly(square, occupancy);
            }

            let bit_count = rook_mask.count_ones();

            let occupancy_variations = 1 << bit_count;

            for count in 0..occupancy_variations {
                let occupancy = set_occupancy(count, bit_count, rook_mask);
                let magic_index = occupancy.wrapping_mul(ROOK_MAGIC_NUMBERS[square])
                    >> 64 - ROOK_RELEVANT_BITS[square];
                rook_attacks[square as usize][magic_index as usize] =
                    rook_attacks_on_the_fly(square, occupancy);
            }
        }
        MagicBitboard {
            bishop_masks,
            rook_masks,
            bishop_attacks,
            rook_attacks,
        }
    }
}

fn bishop_attacks_on_the_fly(square: usize, block: u64) -> u64 {
    let mut attacks = 0u64;

    let mut f;
    let mut r;

    let tr = square / 8;
    let tf = square % 8;

    r = tr + 1;
    f = tf + 1;
    while r <= 7 && f <= 7 {
        attacks |= 1u64 << (r * 8 + f);
        if block & (1u64 << (r * 8 + f)) > 0 {
            break;
        }
        r += 1;
        f += 1;
    }

    if tf > 0 {
        r = tr + 1;
        f = tf - 1;
        while r <= 7 {
            attacks |= 1u64 << (r * 8 + f);
            if block & (1u64 << (r * 8 + f)) > 0 {
                break;
            }
            r += 1;
            if f == 0 {
                break;
            }
            f -= 1;
        }
    }

    if tr > 0 {
        r = tr - 1;
        f = tf + 1;
        while f <= 7 {
            attacks |= 1u64 << (r * 8 + f);
            if block & (1u64 << (r * 8 + f)) > 0 {
                break;
            }
            if r == 0 {
                break;
            }
            r -= 1;
            f += 1;
        }
    }

    if tr > 0 && tf > 0 {
        r = tr - 1;
        f = tf - 1;
        loop {
            attacks |= 1u64 << (r * 8 + f);
            if block & (1u64 << (r * 8 + f)) > 0 {
                break;
            }
            if r == 0 || f == 0 {
                break;
            }
            r -= 1;
            f -= 1;
        }
    }

    attacks
}

fn mask_bishop_attacks(square: usize) -> u64 {
    let mut attacks = 0u64;

    let mut f;
    let mut r;

    let tr = square / 8;
    let tf = square % 8;

    r = tr + 1;
    f = tf + 1;
    while r <= 6 && f <= 6 {
        attacks |= 1u64 << (r * 8 + f);
        r += 1;
        f += 1;
    }

    if tf > 0 {
        r = tr + 1;
        f = tf - 1;
        while r <= 6 && f >= 1 {
            attacks |= 1u64 << (r * 8 + f);
            r += 1;
            f -= 1;
        }
    }

    if tr > 0 {
        r = tr - 1;
        f = tf + 1;
        while r >= 1 && f <= 6 {
            attacks |= 1u64 << (r * 8 + f);
            r -= 1;
            f += 1;
        }
    }

    if tr > 0 && tf > 0 {
        r = tr - 1;
        f = tf - 1;
        while r >= 1 && f >= 1 {
            attacks |= 1u64 << (r * 8 + f);
            r -= 1;
            f -= 1;
        }
    }

    attacks
}

fn rook_attacks_on_the_fly(square: usize, block: u64) -> u64 {
    let mut attacks = 0u64;

    let mut f;
    let mut r;

    let tr = square / 8;
    let tf = square % 8;

    r = tr + 1;
    while r <= 7 {
        attacks |= 1u64 << (r * 8 + tf);
        if (block & (1u64 << (r * 8 + tf))) > 0 {
            break;
        }
        r += 1;
    }

    if tr > 0 {
        r = tr - 1;
        loop {
            attacks |= 1u64 << (r * 8 + tf);
            if (block & (1u64 << (r * 8 + tf))) > 0 {
                break;
            }
            if r == 0 {
                break;
            }
            r -= 1;
        }
    }

    f = tf + 1;
    while f <= 7 {
        attacks |= 1u64 << (tr * 8 + f);
        if (block & (1u64 << (tr * 8 + f))) > 0 {
            break;
        }
        f += 1;
    }

    if tf > 0 {
        f = tf - 1;
        loop {
            attacks |= 1u64 << (tr * 8 + f);
            if (block & (1u64 << (tr * 8 + f))) > 0 {
                break;
            }
            if f == 0 {
                break;
            }
            f -= 1;
        }
    }

    attacks
}

fn mask_rook_attacks(square: usize) -> u64 {
    let mut attacks = 0u64;

    let mut f;
    let mut r;

    let tr = square / 8;
    let tf = square % 8;

    r = tr + 1;
    while r <= 6 {
        attacks |= 1u64 << (r * 8 + tf);
        r += 1;
    }

    if tr > 0 {
        r = tr - 1;
        while r >= 1 {
            attacks |= 1u64 << (r * 8 + tf);
            r -= 1;
        }
    }

    f = tf + 1;
    while f <= 6 {
        attacks |= 1u64 << (tr * 8 + f);
        f += 1;
    }

    if tf > 0 {
        f = tf - 1;
        while f >= 1 {
            attacks |= 1u64 << (tr * 8 + f);
            f -= 1;
        }
    }
    attacks
}

fn set_occupancy(index: usize, bits_in_mask: u32, attack_mask: u64) -> u64 {
    let mut occupancy = 0u64;
    let mut attack_mask = attack_mask;

    for count in 0..bits_in_mask {
        let square = attack_mask.trailing_zeros();

        if (attack_mask & (1u64 << square)) > 0 {
            attack_mask ^= 1u64 << square
        }

        if (index & (1 << count)) > 0 {
            occupancy |= 1u64 << square;
        }
    }

    occupancy
}

// rook relevant occupancy bits
#[rustfmt::skip]
const ROOK_RELEVANT_BITS: [u8; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

// bishop relevant occupancy bits
#[rustfmt::skip]
const BISHOP_RELEVANT_BITS: [u8; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

const ROOK_MAGIC_NUMBERS: [u64; 64] = [
    0x8a80104000800020,
    0x140002000100040,
    0x2801880a0017001,
    0x100081001000420,
    0x200020010080420,
    0x3001c0002010008,
    0x8480008002000100,
    0x2080088004402900,
    0x800098204000,
    0x2024401000200040,
    0x100802000801000,
    0x120800800801000,
    0x208808088000400,
    0x2802200800400,
    0x2200800100020080,
    0x801000060821100,
    0x80044006422000,
    0x100808020004000,
    0x12108a0010204200,
    0x140848010000802,
    0x481828014002800,
    0x8094004002004100,
    0x4010040010010802,
    0x20008806104,
    0x100400080208000,
    0x2040002120081000,
    0x21200680100081,
    0x20100080080080,
    0x2000a00200410,
    0x20080800400,
    0x80088400100102,
    0x80004600042881,
    0x4040008040800020,
    0x440003000200801,
    0x4200011004500,
    0x188020010100100,
    0x14800401802800,
    0x2080040080800200,
    0x124080204001001,
    0x200046502000484,
    0x480400080088020,
    0x1000422010034000,
    0x30200100110040,
    0x100021010009,
    0x2002080100110004,
    0x202008004008002,
    0x20020004010100,
    0x2048440040820001,
    0x101002200408200,
    0x40802000401080,
    0x4008142004410100,
    0x2060820c0120200,
    0x1001004080100,
    0x20c020080040080,
    0x2935610830022400,
    0x44440041009200,
    0x280001040802101,
    0x2100190040002085,
    0x80c0084100102001,
    0x4024081001000421,
    0x20030a0244872,
    0x12001008414402,
    0x2006104900a0804,
    0x1004081002402,
];

const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
    0x40040844404084,
    0x2004208a004208,
    0x10190041080202,
    0x108060845042010,
    0x581104180800210,
    0x2112080446200010,
    0x1080820820060210,
    0x3c0808410220200,
    0x4050404440404,
    0x21001420088,
    0x24d0080801082102,
    0x1020a0a020400,
    0x40308200402,
    0x4011002100800,
    0x401484104104005,
    0x801010402020200,
    0x400210c3880100,
    0x404022024108200,
    0x810018200204102,
    0x4002801a02003,
    0x85040820080400,
    0x810102c808880400,
    0xe900410884800,
    0x8002020480840102,
    0x220200865090201,
    0x2010100a02021202,
    0x152048408022401,
    0x20080002081110,
    0x4001001021004000,
    0x800040400a011002,
    0xe4004081011002,
    0x1c004001012080,
    0x8004200962a00220,
    0x8422100208500202,
    0x2000402200300c08,
    0x8646020080080080,
    0x80020a0200100808,
    0x2010004880111000,
    0x623000a080011400,
    0x42008c0340209202,
    0x209188240001000,
    0x400408a884001800,
    0x110400a6080400,
    0x1840060a44020800,
    0x90080104000041,
    0x201011000808101,
    0x1a2208080504f080,
    0x8012020600211212,
    0x500861011240000,
    0x180806108200800,
    0x4000020e01040044,
    0x300000261044000a,
    0x802241102020002,
    0x20906061210001,
    0x5a84841004010310,
    0x4010801011c04,
    0xa010109502200,
    0x4a02012000,
    0x500201010098b028,
    0x8040002811040900,
    0x28000010020204,
    0x6000020202d0240,
    0x8918844842082200,
    0x4010011029020020,
];

// Lookup table for knight moves
#[rustfmt::skip]
pub const KNIGHTS_LOOKUP: [u64; 64] = [
    132096,329728,659712,1319424,2638848,5277696,10489856,4202496,33816580,84410376,168886289,
    337772578,675545156,1351090312,2685403152,1075839008,8657044482,21609056261,43234889994,
    86469779988,172939559976,345879119952,687463207072,275414786112,2216203387392,5531918402816,
    11068131838464,22136263676928,44272527353856,88545054707712,175990581010432,70506185244672,
    567348067172352,1416171111120896,2833441750646784,5666883501293568,11333767002587136,22667534005174272,
    45053588738670592,18049583422636032,145241105196122112,362539804446949376,725361088165576704,1450722176331153408,
    2901444352662306816,5802888705324613632,11533718717099671552,4620693356194824192,288234782788157440,576469569871282176,
    1224997833292120064,2449995666584240128,4899991333168480256,9799982666336960512,1152939783987658752,2305878468463689728,
    1128098930098176,2257297371824128,4796069720358912,9592139440717824,19184278881435648,38368557762871296,4679521487814656,9077567998918656,
];

// Lookup table for king moves
#[rustfmt::skip]
pub const KING_LOOKUP: [u64; 64] = [ 
    770,1797,3594,7188,14376,28752,57504,49216,197123,460039,920078,1840156,3680312,7360624,
    14721248,12599488,50463488,117769984,235539968,471079936,942159872,1884319744,3768639488,
    3225468928,12918652928,30149115904,60298231808,120596463616,241192927232,482385854464,964771708928,
    825720045568,3307175149568,7718173671424,15436347342848,30872694685696,61745389371392,123490778742784,
    246981557485568,211384331665408,846636838289408,1975852459884544,3951704919769088,7903409839538176,15806819679076352,
    31613639358152704,63227278716305408,54114388906344448,216739030602088448,505818229730443264,1011636459460886528,
    2023272918921773056,4046545837843546112,8093091675687092224,16186183351374184448,13853283560024178688,144959613005987840,
    362258295026614272,724516590053228544,1449033180106457088,2898066360212914176,5796132720425828352,
    11592265440851656704,4665729213955833856,
];
