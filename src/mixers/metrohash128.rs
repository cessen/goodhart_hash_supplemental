pub const IN_SIZE_BYTES: usize = 256 / 8;
pub const OUT_SIZE_BYTES: usize = 256 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

/// The MetroHash128 accumulator.
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    const K0: u64 = 0xC83A91E1;
    const K1: u64 = 0x8648DBDB;
    const K2: u64 = 0x7BDEC03B;
    const K3: u64 = 0x2F5870A5;

    // Fetches the 64-bit chunk of input data at byte offset `i`. Returns 0 if
    // it's out-of-bounds, which serves to pretend like there is an infinite
    // stream of zeroed out bytes after the initial data.  This is useful for
    // testing additional rounds without interference from other data.
    let read_u64 = |i: usize| -> u64 {
        if (i + 8) <= IN_SIZE_BYTES {
            u64::from_le_bytes((&in_bytes[i..(i + 8)]).try_into().unwrap())
        } else {
            0
        }
    };

    let mut state = [0u64; 4];
    let mut offset = 0;
    let rounds = 1; // Set higher to play with how much diffusion you get per block.
    for _ in 0..rounds {
        state[0] += read_u64(offset) * K0;
        offset += 8;
        state[0] = state[0].rotate_right(29) + state[2];
        state[1] += read_u64(offset) * K1;
        offset += 8;
        state[1] = state[1].rotate_right(29) + state[3];
        state[2] += read_u64(offset) * K2;
        offset += 8;
        state[2] = state[2].rotate_right(29) + state[0];
        state[3] += read_u64(offset) * K3;
        offset += 8;
        state[3] = state[3].rotate_right(29) + state[1];
    }

    // Copy the mixed state to the output.
    out_bytes[0..8].copy_from_slice(&u64::to_le_bytes(state[0]));
    out_bytes[8..16].copy_from_slice(&u64::to_le_bytes(state[1]));
    out_bytes[16..24].copy_from_slice(&u64::to_le_bytes(state[2]));
    out_bytes[24..32].copy_from_slice(&u64::to_le_bytes(state[3]));
}
