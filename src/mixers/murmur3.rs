pub const IN_SIZE_BYTES: usize = 128 / 8;
pub const OUT_SIZE_BYTES: usize = 128 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

/// The Murmur3 accumulator.
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    const C1: u64 = 0x87c37b91114253d5;
    const C2: u64 = 0x4cf5ad432745937f;
    const SEED: u64 = 0xe9e58282f1c2287e;

    let mut h1 = SEED;
    let mut h2 = SEED;
    let rounds = 1; // Set higher to play with how much diffusion you get per block.
    for i in 0..rounds {
        let [mut k1, mut k2] = if i == 0 {
            // Copy the input into the right layout.
            [
                u64::from_le_bytes((&in_bytes[0..8]).try_into().unwrap()),
                u64::from_le_bytes((&in_bytes[8..16]).try_into().unwrap()),
            ]
        } else {
            // We use zeros after the first round so that we're just
            // tracking how well that first block diffuses.  Normally
            // in Murmur3 additional blocks would be accumulated here.
            [0, 0]
        };

        k1 *= C1;
        k1 = k1.rotate_left(31);
        k1 *= C2;
        h1 ^= k1;

        h1 = h1.rotate_left(27);
        h1 += h2;
        h1 = h1 * 5 + 0x52dce729;

        k2 *= C2;
        k2 = k2.rotate_left(33);
        k2 *= C1;
        h2 ^= k2;

        h2 = h2.rotate_left(31);
        h2 += h1;
        h2 = h2 * 5 + 0x38495ab5;
    }

    // Copy the mixed state to the output.
    out_bytes[0..8].copy_from_slice(&u64::to_le_bytes(h2));
    out_bytes[8..16].copy_from_slice(&u64::to_le_bytes(h1));
}
