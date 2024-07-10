#[cfg(target_arch = "x86")]
use std::arch::x86::{__m128i, _mm_aesenc_si128};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{__m128i, _mm_aesenc_si128};

pub const IN_SIZE_BYTES: usize = 512 / 8;
pub const OUT_SIZE_BYTES: usize = 512 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

/// The AquaHash accumulator.
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    // Fetches the 128-bit chunk of input data at byte offset `i`. Returns 0 if
    // it's out-of-bounds, which serves to pretend like there is an infinite
    // stream of zeroed out bytes after the initial data.  This is useful for
    // testing additional rounds without interference from other data.
    let fetch128 = |i: usize| -> __m128i {
        if (i + 16) <= IN_SIZE_BYTES {
            unsafe {
                std::mem::transmute(u128::from_le_bytes(
                    (&in_bytes[i..(i + 16)]).try_into().unwrap(),
                ))
            }
        } else {
            unsafe { std::mem::transmute(0u128) }
        }
    };

    // Initial state.
    let mut state: [__m128i; 4] = [
        unsafe { std::mem::transmute([0xa11202c9b468bea1u64, 0xd75157a01452495bu64]) },
        unsafe { std::mem::transmute([0xb1293b3305418592u64, 0xd210d232c6429b69u64]) },
        unsafe { std::mem::transmute([0xbd3dc2b7b87c4715u64, 0x6a6c9527ac2e0e4eu64]) },
        unsafe { std::mem::transmute([0xcc96ed1674eaaa03u64, 0x1e863f24b2a8316au64]) },
    ];

    // Accumulate block.
    // Note we use 2 rounds for the default test here because of how AES rounds
    // work.  The key (in this case the input data) isn't incorporated until
    // the end, and that's only by a xor which doesn't mix anything.  So in
    // practice, the mixing of a block actually happens at the start of the next
    // AES round.
    let rounds = 2; // Set higher to play with how diffussion progresses with more blocks.
    let mut data_offset = 0;
    for _ in 0..rounds {
        state[0] = unsafe { _mm_aesenc_si128(state[0], fetch128(data_offset)) };
        data_offset += 16;
        state[1] = unsafe { _mm_aesenc_si128(state[1], fetch128(data_offset)) };
        data_offset += 16;
        state[2] = unsafe { _mm_aesenc_si128(state[2], fetch128(data_offset)) };
        data_offset += 16;
        state[3] = unsafe { _mm_aesenc_si128(state[3], fetch128(data_offset)) };
        data_offset += 16;
    }

    // Copy the mixed state to the output.
    out_bytes[0..16].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(state[0]) });
    out_bytes[16..32].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(state[1]) });
    out_bytes[32..48].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(state[2]) });
    out_bytes[48..64].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(state[3]) });
}
