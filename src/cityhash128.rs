pub const IN_SIZE_BYTES: usize = 512 / 8;
pub const OUT_SIZE_BYTES: usize = 448 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

/// The CityHash128/FarmHash128 accumulator.  (Yes, they are identical.)
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    const K1: u64 = 0xb492b66fbe98f273;

    const SEED1: u64 = 0x6cfd5fc33eb025ed;
    const SEED2: u64 = 0x22db8460f81d5fea;

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

    // Return a 16-byte hash for input_bytes[offset..(offset+32)], a, and b.
    let weak_hash_len_32_with_seeds = |offset: usize, mut a: u64, mut b: u64| -> (u64, u64) {
        let w = fetch64(offset);
        let x = fetch64(offset + 8);
        let y = fetch64(offset + 16);
        let z = fetch64(offset + 24);

        a += w;
        b = (b + a + z).rotate_right(21);
        let c = a;
        a += x;
        a += y;
        b += a.rotate_right(44);

        (a + z, b + c)
    };

    let rounds = 1; // Set higher to play with how diffussion progresses with more blocks.

    let mut v = (0u64, 0u64);
    let mut w = (0u64, 0u64);
    let mut x = SEED1;
    let mut y = SEED2;
    let mut z = (rounds as u64 * 128 / 8).wrapping_mul(K1);
    v.0 = (y ^ K1).rotate_right(49) * K1 + fetch64(0);
    v.1 = (v.0).rotate_right(42) * K1 + fetch64(8);
    w.0 = (y + z).rotate_right(35) * K1 + x;
    w.1 = (x + fetch64(88)).rotate_right(53) * K1;

    let mut data_offset = 0;
    for _ in 0..rounds {
        // Note: in the reference CityHash128 implementation, this
        // loop is unrolled.  However, each loop processes a *different*
        // block of data, so for the purpose of analyzing diffusion
        // between blocks we only want a single loop of this.  Hence why
        // the second unroll has been removed here.
        x = (x + y + v.0 + fetch64(data_offset + 8)).rotate_right(37) * K1;
        y = (y + v.1 + fetch64(data_offset + 48)).rotate_right(42) * K1;
        x ^= w.1;
        y += v.0 + fetch64(data_offset + 40);
        z = (z + w.0).rotate_right(33) * K1;
        v = weak_hash_len_32_with_seeds(data_offset, v.1 * K1, x + w.0);
        w = weak_hash_len_32_with_seeds(data_offset + 32, z + w.1, y + fetch64(data_offset + 16));
        std::mem::swap(&mut z, &mut x);
        data_offset += 64;
    }

    // Copy the mixed state to the output.
    out_bytes[0..8].copy_from_slice(&u64::to_le_bytes(v.0));
    out_bytes[8..16].copy_from_slice(&u64::to_le_bytes(v.1));
    out_bytes[16..24].copy_from_slice(&u64::to_le_bytes(w.0));
    out_bytes[24..32].copy_from_slice(&u64::to_le_bytes(w.1));
    out_bytes[32..40].copy_from_slice(&u64::to_le_bytes(x));
    out_bytes[40..48].copy_from_slice(&u64::to_le_bytes(y));
    out_bytes[48..56].copy_from_slice(&u64::to_le_bytes(z));
}
