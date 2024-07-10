pub const IN_SIZE_BYTES: usize = 128 / 8;
pub const OUT_SIZE_BYTES: usize = 128 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

/// The FNV1a hash, 128-bit variant.
///
/// Note that FNV isn't a block-based hash, so measuring the mixing between
/// "blocks" doesn't have the same meaning as usual.  However, it can still give
/// us a good idea of how quickly diffusion happens.  For this we just assume a
/// "block" size of 128 bits.
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    // For rounds after the first one.
    let blank = &[0u8; 16][..];

    let rounds = 1; // Set higher to play with how much diffusion you get per block.

    let mut state: u128 = 0x6c62272e07bb014262b821756295c58d;
    for block in [in_bytes].iter().chain([blank].iter().cycle()).take(rounds) {
        for &byte in block.iter() {
            state ^= byte as u128;
            state *= 0x1000000000000000000013b;
        }
    }

    // Copy the mixed state to the output.
    out_bytes[0..16].copy_from_slice(&u128::to_le_bytes(state));
}
