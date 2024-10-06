#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

pub const IN_SIZE_BYTES: usize = 128 / 8;
pub const OUT_SIZE_BYTES: usize = 128 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

// Some random numbers, for playing with the number of rounds below.
const KEYS: &[u128] = &[
    0x5aee66ffbc9d5f254dd19917b03fb552,
    0xb625245574d76546f7007e2431b3c833,
    0x1896c105404b2ea0d25e2a5875e2debc,
    0x06805b053507e7f3eae6bbd2d5eec448,
    0x10a7c61c8f5cc2a05c1aeb42b567cd13,
    0x06b6c1596f23e0f0c8efabd49aa8ee35,
    0x864b58dfc6b7d377605384a744a6c1fa,
    0x14ed774d389a2665d9e32ec187f79cf0,
];

// Two rounds of AES as a mixer.
//
// This is included because there are an increasing number of hashes that use
// AES intrinsics as their core bit mixing compontent.  There's nothing wrong
// with that, but it's important to be aware that it's not magic, and a minimium
// of three full rounds is needed to achieve full 128-bit diffusion.
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    let mut state: __m128i = unsafe { _mm_loadu_si128(in_bytes.as_ptr().cast()) };
    unsafe {
        for i in 0..2 {
            state = _mm_aesenc_si128(state, std::mem::transmute(KEYS[i]));
        }

        // Note: `_mm_aesenclast_si128()` doesn't do as much mixing as
        // `_mm_aesenc_si128()`.  Therfore uncommenting the line below dosen't
        // fully diffuse the hash state after the two full rounds above, whereas
        // simply doing another round of `_mm_aesenc_si128()` does.

        // state = _mm_aesenclast_si128(state, std::mem::transmute(KEYS[7]));
    }

    // Copy the mixed state to the output.
    out_bytes[0..16].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(state) });
}
