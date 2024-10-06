pub const IN_SIZE_BYTES: usize = 768 / 8;
pub const OUT_SIZE_BYTES: usize = 768 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

// The absorber from SpookyHash 2.
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    // Fetches the 64-bit chunk of input data at byte offset `i`. Returns 0 if
    // it's out-of-bounds, which serves to pretend like there is an infinite
    // stream of zeroed out bytes after the initial data.  This is useful for
    // testing additional rounds without interference from other data.
    let fetch64 = |i: usize| -> u64 {
        if (i + 8) <= IN_SIZE_BYTES {
            u64::from_le_bytes((&in_bytes[i..(i + 8)]).try_into().unwrap())
        } else {
            0
        }
    };

    let mut data_offset = 0;
    let mut state = [0u64; 12];

    let rounds = 1;

    #[cfg_attr(rustfmt, rustfmt_skip)]
    for _ in 0..rounds {
        state[0]  += fetch64(data_offset + (8 * 0));  state[2]  ^= state[10]; state[11] ^= state[0];  state[0]  = state[0].rotate_left(11);  state[11] += state[1];
        state[1]  += fetch64(data_offset + (8 * 1));  state[3]  ^= state[11]; state[0]  ^= state[1];  state[1]  = state[1].rotate_left(32);  state[0]  += state[2];
        state[2]  += fetch64(data_offset + (8 * 2));  state[4]  ^= state[0];  state[1]  ^= state[2];  state[2]  = state[2].rotate_left(43);  state[1]  += state[3];
        state[3]  += fetch64(data_offset + (8 * 3));  state[5]  ^= state[1];  state[2]  ^= state[3];  state[3]  = state[3].rotate_left(31);  state[2]  += state[4];
        state[4]  += fetch64(data_offset + (8 * 4));  state[6]  ^= state[2];  state[3]  ^= state[4];  state[4]  = state[4].rotate_left(17);  state[3]  += state[5];
        state[5]  += fetch64(data_offset + (8 * 5));  state[7]  ^= state[3];  state[4]  ^= state[5];  state[5]  = state[5].rotate_left(28);  state[4]  += state[6];
        state[6]  += fetch64(data_offset + (8 * 6));  state[8]  ^= state[4];  state[5]  ^= state[6];  state[6]  = state[6].rotate_left(39);  state[5]  += state[7];
        state[7]  += fetch64(data_offset + (8 * 7));  state[9]  ^= state[5];  state[6]  ^= state[7];  state[7]  = state[7].rotate_left(57);  state[6]  += state[8];
        state[8]  += fetch64(data_offset + (8 * 8));  state[10] ^= state[6];  state[7]  ^= state[8];  state[8]  = state[8].rotate_left(55);  state[7]  += state[9];
        state[9]  += fetch64(data_offset + (8 * 9));  state[11] ^= state[7];  state[8]  ^= state[9];  state[9]  = state[9].rotate_left(54);  state[8]  += state[10];
        state[10] += fetch64(data_offset + (8 * 10)); state[0]  ^= state[8];  state[9]  ^= state[10]; state[10] = state[10].rotate_left(22); state[9]  += state[11];
        state[11] += fetch64(data_offset + (8 * 11)); state[1]  ^= state[9];  state[10] ^= state[11]; state[11] = state[11].rotate_left(46); state[10] += state[0];

        data_offset += IN_SIZE_BYTES;
    }

    // Copy the mixed state to the output.
    out_bytes[0..8].copy_from_slice(&u64::to_le_bytes(state[0]));
    out_bytes[8..16].copy_from_slice(&u64::to_le_bytes(state[1]));
    out_bytes[16..24].copy_from_slice(&u64::to_le_bytes(state[2]));
    out_bytes[24..32].copy_from_slice(&u64::to_le_bytes(state[3]));
    out_bytes[32..40].copy_from_slice(&u64::to_le_bytes(state[4]));
    out_bytes[40..48].copy_from_slice(&u64::to_le_bytes(state[5]));
    out_bytes[48..56].copy_from_slice(&u64::to_le_bytes(state[6]));
    out_bytes[56..64].copy_from_slice(&u64::to_le_bytes(state[7]));
    out_bytes[64..72].copy_from_slice(&u64::to_le_bytes(state[8]));
    out_bytes[72..80].copy_from_slice(&u64::to_le_bytes(state[9]));
    out_bytes[80..88].copy_from_slice(&u64::to_le_bytes(state[10]));
    out_bytes[88..96].copy_from_slice(&u64::to_le_bytes(state[11]));
}
