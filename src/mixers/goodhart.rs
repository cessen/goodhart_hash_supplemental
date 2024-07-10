pub const IN_SIZE_BYTES: usize = 128 / 8;
pub const OUT_SIZE_BYTES: usize = 128 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

// Play with this to see how the number of rounds affects diffusion.
const ROUNDS: usize = 12;

/// The mix function from "Hash Design and Goodhart's Law".
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    // Copy the state into the right layout.
    let mut state = [
        u64::from_le_bytes((&in_bytes[0..8]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[8..16]).try_into().unwrap()),
    ];

    const ROTATIONS: &[u32] = &[12, 39, 21, 13, 32, 11, 24, 53, 17, 27, 57, 13, 50, 8, 52, 8];

    for &rot in ROTATIONS.iter().take(ROUNDS) {
        state[0] = state[0].wrapping_add(state[1]).wrapping_add(1);
        state[1] = state[1].rotate_left(rot) ^ state[0];
    }

    // Copy the mixed state to the output.
    out_bytes[0..8].copy_from_slice(&u64::to_le_bytes(state[0]));
    out_bytes[8..16].copy_from_slice(&u64::to_le_bytes(state[1]));
}
