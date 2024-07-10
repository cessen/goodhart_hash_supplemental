pub const IN_SIZE_BYTES: usize = 512 / 8;
pub const OUT_SIZE_BYTES: usize = 512 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

const PRIME32_1: u32 = 0x9E3779B1;
const PRIME32_2: u32 = 0x85EBCA77;
const PRIME32_3: u32 = 0xC2B2AE3D;
// const PRIME32_4: u32 = 0x27D4EB2F;
// const PRIME32_5: u32 = 0x165667B1;
const PRIME64_1: u64 = 0x9E3779B185EBCA87;
const PRIME64_2: u64 = 0xC2B2AE3D27D4EB4F;
const PRIME64_3: u64 = 0x165667B19E3779F9;
const PRIME64_4: u64 = 0x85EBCA77C2B2AE63;
const PRIME64_5: u64 = 0x27D4EB2F165667C5;

// Unlike the standard implementation, we store the secrets as u64s rather than
// as a byte array, since that's how they're used anyway.
const SECRET: [u64; 24] = [
    0xb8fe6c3923a44bbe,
    0x7c01812cf721ad1c,
    0xded46de9839097db,
    0x7240a4a4b7b3671f,
    0xcb79e64eccc0e578,
    0x825ad07dccff7221,
    0xb8084674f743248e,
    0xe03590e6813a264c,
    0x3c2852bb91c300cb,
    0x88d0658b1b532ea3,
    0x71644897a20df94e,
    0x3819ef46a9deacd8,
    0xa8fa763fe39c343f,
    0xf9dcbbc7c70b4f1d,
    0x8a51e04bcdb45931,
    0xc89f7ec9d9787364,
    0xeac5ac8334d3ebc3,
    0xc581a0fffa1363eb,
    0x170ddd51b7f0da49,
    0xd316552629d4689e,
    0x2b16be587d47a1fc,
    0x8ff8b8d17ad031ce,
    0x45cb3a8f95160428,
    0xafd7fbcabb4b407e,
];

/// The large-size xxhash3 accumulator, which runs in the inner
/// loop of xxhash3 as the primary way to incorporate input
/// blocks into the hash.
///
/// Note that in xxhash's terminology "block" is used to mean
/// something else, more akin to a set of blocks, and uses the
/// term "stripe" for what would normally be considered a block.
/// Normally a "stripe" would indicate striping the input for
/// multiple parallel accumulators, but xxhash3 uses it to refer
/// to chunks of data that are simply accumulated sequentially.
///
/// We use xxhash3's terminology in the code below for consistency
/// with its specification.
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    // Copy input into the right layout.
    let stripe = [
        u64::from_le_bytes((&in_bytes[0..8]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[8..16]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[16..24]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[24..32]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[32..40]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[40..48]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[48..56]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[56..64]).try_into().unwrap()),
    ];

    // Set accumulator state to the default.
    let mut accum_state = [
        PRIME32_3 as u64,
        PRIME64_1,
        PRIME64_2,
        PRIME64_3,
        PRIME64_4,
        PRIME32_2 as u64,
        PRIME64_5,
        PRIME32_1 as u64,
    ];

    let rounds = 1; // Set higher to play with how much diffusion you get per block.
    for i in 0..rounds {
        let secret_offset = (i * 8) % SECRET.len();

        // From the xxhash3 spec:
        // ```
        // accumulate(u64 stripe[8], size secretOffset):
        //   u64 secretWords[8] = secret[secretOffset:secretOffset+64];
        //   for (i = 0; i < 8; i++) {
        //     u64 value = stripe[i] xor secretWords[i];
        //     acc[i xor 1] = acc[i xor 1] + stripe[i];
        //     acc[i] = acc[i] + (u64)lowerHalf(value) * (u64)higherHalf(value);
        //                       // (value and 0xFFFFFFFF) * (value >> 32)
        //   }
        // ```
        let secret_words = &SECRET[secret_offset..];
        for i in 0..8 {
            let chunk = stripe.get(i).map(|c| *c).unwrap_or(0); // Assume off-the-end data is a stream of zeros, for rounds testing.
            let value = chunk ^ secret_words[i];
            accum_state[i ^ 1] += chunk;
            accum_state[i] += (value & 0xffffffff) * (value >> 32);
        }
    }

    // Copy the mixed state back to the byte buffer.
    out_bytes[0..8].copy_from_slice(&u64::to_le_bytes(accum_state[0]));
    out_bytes[8..16].copy_from_slice(&u64::to_le_bytes(accum_state[1]));
    out_bytes[16..24].copy_from_slice(&u64::to_le_bytes(accum_state[2]));
    out_bytes[24..32].copy_from_slice(&u64::to_le_bytes(accum_state[3]));
    out_bytes[32..40].copy_from_slice(&u64::to_le_bytes(accum_state[4]));
    out_bytes[40..48].copy_from_slice(&u64::to_le_bytes(accum_state[5]));
    out_bytes[48..56].copy_from_slice(&u64::to_le_bytes(accum_state[6]));
    out_bytes[56..64].copy_from_slice(&u64::to_le_bytes(accum_state[7]));
}
