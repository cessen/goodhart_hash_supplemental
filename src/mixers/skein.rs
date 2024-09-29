//! Skein's 256-bit mixing function, at only 7 rounds.
//!
//! NOTE: this is not reflective of Skein's quality.  Actual Skein uses a much
//! larger number of rounds, which fully diffuses the hash state many times
//! over.  This reduced-round version is included here purely for comparison
//! with TentHash, which uses an effectively identical construction, but with
//! constants optimized specifically for fewer rounds.

pub const IN_SIZE_BYTES: usize = 256 / 8;
pub const OUT_SIZE_BYTES: usize = 256 / 8;
pub const DIGEST_SIZE_BYTES: usize = 256 / 8;

pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    // Copy the state into the right layout.
    let mut state = [
        u64::from_le_bytes((&in_bytes[0..8]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[8..16]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[16..24]).try_into().unwrap()),
        u64::from_le_bytes((&in_bytes[24..32]).try_into().unwrap()),
    ];

    mix_state(&mut state);

    // Copy the mixed state to the output.
    out_bytes[0..8].copy_from_slice(&u64::to_le_bytes(state[0]));
    out_bytes[8..16].copy_from_slice(&u64::to_le_bytes(state[1]));
    out_bytes[16..24].copy_from_slice(&u64::to_le_bytes(state[2]));
    out_bytes[24..32].copy_from_slice(&u64::to_le_bytes(state[3]));
}

fn mix_state(state: &mut [u64; 4]) {
    const ROTATIONS: &[[u32; 2]] = &[
        [14, 16],
        [52, 57],
        [23, 40],
        [5, 37],
        [25, 33],
        [46, 12],
        [58, 22],
        [32, 32],
    ];

    for rot_pair in ROTATIONS.iter().take(7) {
        state[0] = state[0].wrapping_add(state[1]);
        state[1] = state[1].rotate_left(rot_pair[0]) ^ state[0];
        state[2] = state[2].wrapping_add(state[3]);
        state[3] = state[3].rotate_left(rot_pair[1]) ^ state[2];

        state.swap(1, 3);
    }
}
