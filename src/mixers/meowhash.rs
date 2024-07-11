// Substantial portions of this code are from https://github.com/bodil/meowhash-rs
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::sync::atomic::{fence, Ordering};

// Defined below.
use x86::*;

pub const IN_SIZE_BYTES: usize = 256 / 8;
pub const OUT_SIZE_BYTES: usize = 1024 / 8;
pub const DIGEST_SIZE_BYTES: usize = 128 / 8;

#[rustfmt::skip]
const SEED: [[u8; 16]; OUT_SIZE_BYTES / 16] = [
    [0x32, 0x43, 0xF6, 0xA8, 0x88, 0x5A, 0x30, 0x8D, 0x31, 0x31, 0x98, 0xA2, 0xE0, 0x37, 0x07, 0x34],
    [0x4A, 0x40, 0x93, 0x82, 0x22, 0x99, 0xF3, 0x1D, 0x00, 0x82, 0xEF, 0xA9, 0x8E, 0xC4, 0xE6, 0xC8],
    [0x94, 0x52, 0x82, 0x1E, 0x63, 0x8D, 0x01, 0x37, 0x7B, 0xE5, 0x46, 0x6C, 0xF3, 0x4E, 0x90, 0xC6],
    [0xCC, 0x0A, 0xC2, 0x9B, 0x7C, 0x97, 0xC5, 0x0D, 0xD3, 0xF8, 0x4D, 0x5B, 0x5B, 0x54, 0x70, 0x91],
    [0x79, 0x21, 0x6D, 0x5D, 0x98, 0x97, 0x9F, 0xB1, 0xBD, 0x13, 0x10, 0xBA, 0x69, 0x8D, 0xFB, 0x5A],
    [0xC2, 0xFF, 0xD7, 0x2D, 0xBD, 0x01, 0xAD, 0xFB, 0x7B, 0x8E, 0x1A, 0xFE, 0xD6, 0xA2, 0x67, 0xE9],
    [0x6B, 0xA7, 0xC9, 0x04, 0x5F, 0x12, 0xC7, 0xF9, 0x92, 0x4A, 0x19, 0x94, 0x7B, 0x39, 0x16, 0xCF],
    [0x70, 0x80, 0x1F, 0x2E, 0x28, 0x58, 0xEF, 0xC1, 0x66, 0x36, 0x92, 0x0D, 0x87, 0x15, 0x74, 0xE6],
];

macro_rules! mix_reg {
    ($r1:ident, $r2:ident, $r3:ident, $r4:ident, $r5:ident, $i1:expr, $i2:expr, $i3:expr, $i4:expr) => {
        $r1 = aesdec($r1, $r2);
        fence(Ordering::AcqRel);
        $r3 = paddq($r3, $i1);
        $r2 = pxor($r2, $i2);
        $r2 = aesdec($r2, $r4);
        fence(Ordering::AcqRel);
        $r5 = paddq($r5, $i3);
        $r4 = pxor($r4, $i4);
    };
}

macro_rules! mix {
    ($r1:ident, $r2:ident, $r3:ident, $r4:ident, $r5:ident, $ptr:expr) => {
        mix_reg!(
            $r1,
            $r2,
            $r3,
            $r4,
            $r5,
            movdqu(($ptr).add(15).cast()),
            movdqu(($ptr).add(0).cast()),
            movdqu(($ptr).add(1).cast()),
            movdqu(($ptr).add(16).cast())
        )
    };
}

/// The MeowHash v0.5 block absorber.
pub fn mix_input(in_bytes: &[u8], out_bytes: &mut [u8]) {
    assert!(in_bytes.len() == IN_SIZE_BYTES);
    assert!(out_bytes.len() == OUT_SIZE_BYTES);

    let zero_bytes = [0u8; IN_SIZE_BYTES];

    let mut xmm0: Simd128 = unsafe { std::mem::transmute(SEED[0]) };
    let mut xmm1: Simd128 = unsafe { std::mem::transmute(SEED[1]) };
    let mut xmm2: Simd128 = unsafe { std::mem::transmute(SEED[2]) };
    let mut xmm3: Simd128 = unsafe { std::mem::transmute(SEED[3]) };
    let mut xmm4: Simd128 = unsafe { std::mem::transmute(SEED[4]) };
    let mut xmm5: Simd128 = unsafe { std::mem::transmute(SEED[5]) };
    let mut xmm6: Simd128 = unsafe { std::mem::transmute(SEED[6]) };
    let mut xmm7: Simd128 = unsafe { std::mem::transmute(SEED[7]) };

    // The absorber.
    //
    // We're being a bit generous here by considering a round to be three
    // mix calls (actually incorporating three blocks) rather than one.  The
    // rationale is that due to the way MeowHash shuffles its use of the xmm
    // slots, this is the minimum number of mix calls needed to ensure that
    // every xmm slot will have another input block incorporated into it (it's
    // actually the fourth mix call that touches the final xmm slot, but it does
    // so before doing any mixing on that slot).  Since what we care about is
    // the complexity of inter-block bit relationships, that's what's needed
    // here to be conservative.
    //
    // Note that it's possible the situation is worse than this conservative
    // test indicates--but asserting that would require more analysis than I
    // have the energy for.
    //
    // Regardless, it's a bit moot since an input block doesn't reach even close
    // to 128 bits of min diffusion within three mix calls anyway.  That takes
    // six mix calls (and more for patterned inputs).
    //
    // You can uncomment subsequent rounds below to play with how much diffusion
    // you get per subsequent input block.
    unsafe {
        // Round 1.
        mix!(xmm0, xmm4, xmm6, xmm1, xmm2, in_bytes.as_ptr());
        mix!(xmm1, xmm5, xmm7, xmm2, xmm3, zero_bytes.as_ptr());
        mix!(xmm2, xmm6, xmm0, xmm3, xmm4, zero_bytes.as_ptr());

        // // Round 2.
        // mix!(xmm3, xmm7, xmm1, xmm4, xmm5, zero_bytes.as_ptr());
        // mix!(xmm4, xmm0, xmm2, xmm5, xmm6, zero_bytes.as_ptr());
        // mix!(xmm5, xmm1, xmm3, xmm6, xmm7, zero_bytes.as_ptr());

        // // Round 3.
        // mix!(xmm6, xmm2, xmm4, xmm7, xmm0, zero_bytes.as_ptr());
        // mix!(xmm7, xmm3, xmm5, xmm0, xmm1, zero_bytes.as_ptr());

        // Repeat.

        // // Round 1.
        // mix!(xmm0, xmm4, xmm6, xmm1, xmm2, zero_bytes.as_ptr());
        // mix!(xmm1, xmm5, xmm7, xmm2, xmm3, zero_bytes.as_ptr());
        // mix!(xmm2, xmm6, xmm0, xmm3, xmm4, zero_bytes.as_ptr());

        // // Round 2.
        // mix!(xmm3, xmm7, xmm1, xmm4, xmm5, zero_bytes.as_ptr());
        // mix!(xmm4, xmm0, xmm2, xmm5, xmm6, zero_bytes.as_ptr());
        // mix!(xmm5, xmm1, xmm3, xmm6, xmm7, zero_bytes.as_ptr());

        // // Round 3.
        // mix!(xmm6, xmm2, xmm4, xmm7, xmm0, zero_bytes.as_ptr());
        // mix!(xmm7, xmm3, xmm5, xmm0, xmm1, zero_bytes.as_ptr());
    }

    // Copy the mixed state to the output.
    out_bytes[0..16].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(xmm0) });
    out_bytes[16..32].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(xmm1) });
    out_bytes[32..48].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(xmm2) });
    out_bytes[48..64].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(xmm3) });
    out_bytes[64..80].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(xmm4) });
    out_bytes[80..96].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(xmm5) });
    out_bytes[96..112].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(xmm6) });
    out_bytes[112..128].copy_from_slice(&unsafe { std::mem::transmute::<_, [u8; 16]>(xmm7) });
}

mod x86 {
    #[cfg(target_arch = "x86")]
    use core::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use core::arch::x86_64::*;

    pub(crate) type Simd128 = __m128i;

    // pub(crate) const MEOW_PREFETCH: usize = 4096;
    // pub(crate) const MEOW_PREFETCH_LIMIT: usize = 0x3ff;

    // #[inline]
    // #[target_feature(enable = "sse")]
    // pub(crate) unsafe fn prefetcht0(p: *const u8) {
    //     _mm_prefetch(p as *const i8, _MM_HINT_T0)
    // }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub(crate) unsafe fn movdqu(addr: *const u8) -> Simd128 {
        _mm_loadu_si128(addr.cast())
    }

    // #[inline]
    // #[target_feature(enable = "sse2")]
    // pub(crate) unsafe fn movdqu_mem(addr: *mut Simd128, value: Simd128) {
    //     _mm_storeu_si128(addr, value)
    // }

    // #[inline]
    // #[target_feature(enable = "sse2")]
    // pub(crate) unsafe fn movq(value: i64) -> Simd128 {
    //     _mm_set_epi64x(0, value)
    // }

    #[inline]
    #[target_feature(enable = "aes")]
    pub(crate) unsafe fn aesdec(value: Simd128, key: Simd128) -> Simd128 {
        _mm_aesdec_si128(value, key)
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub(crate) unsafe fn pxor(a: Simd128, b: Simd128) -> Simd128 {
        _mm_xor_si128(a, b)
    }

    #[inline]
    #[target_feature(enable = "sse2")]
    pub(crate) unsafe fn paddq(a: Simd128, b: Simd128) -> Simd128 {
        _mm_add_epi64(a, b)
    }

    // #[inline]
    // #[target_feature(enable = "sse2")]
    // pub(crate) unsafe fn pand(a: Simd128, b: Simd128) -> Simd128 {
    //     _mm_and_si128(a, b)
    // }

    // #[inline]
    // #[target_feature(enable = "ssse3")]
    // pub(crate) unsafe fn palignr_1(a: Simd128, b: Simd128) -> Simd128 {
    //     _mm_alignr_epi8(a, b, 1)
    // }

    // #[inline]
    // #[target_feature(enable = "ssse3")]
    // pub(crate) unsafe fn palignr_15(a: Simd128, b: Simd128) -> Simd128 {
    //     _mm_alignr_epi8(a, b, 15)
    // }

    // #[inline]
    // #[target_feature(enable = "sse2")]
    // pub(crate) unsafe fn pxor_clear() -> Simd128 {
    //     _mm_setzero_si128()
    // }

    // #[inline]
    // #[target_feature(enable = "sse2")]
    // pub(crate) unsafe fn cmpeq(a: Simd128, b: Simd128) -> bool {
    //     _mm_movemask_epi8(_mm_cmpeq_epi8(a, b)) == 0xFFFF
    // }
}
