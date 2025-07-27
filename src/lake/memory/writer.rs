use std::arch::x86_64::{__m128i, __m256i, _mm256_loadu_si256, _mm256_storeu_si256, _mm_loadu_si128, _mm_prefetch, _mm_storeu_si128, _MM_HINT_T0};
use crate::lake::memory::LakeTools;

impl LakeTools {

    //write_to(dst, src, N) finished in 9 ns.
    // -------------------------------------------
    // copy_nonoverlapping(dst, src, N) finished in 10 ns.
    #[target_feature(enable = "avx2")]
    pub unsafe fn write_to(mut dst: *mut u8, mut src: *const u8, mut len: usize) {
        if len == 1024 {
            Self::write_1024(dst, src);
        } else {
            Self::_write_to(dst, src, len);
        }
    }

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn write_1024(dst: *mut u8, src: *const u8) {
        macro_rules! do32 {
        ($i:expr) => {
            let v = _mm256_loadu_si256(src.add($i) as *const __m256i);
            _mm256_storeu_si256(dst.add($i) as *mut __m256i, v);
        };
    }

        do32!(0);
        do32!(32);
        do32!(64);
        do32!(96);
        do32!(128);
        do32!(160);
        do32!(192);
        do32!(224);
        do32!(256);
        do32!(288);
        do32!(320);
        do32!(352);
        do32!(384);
        do32!(416);
        do32!(448);
        do32!(480);
        do32!(512);
        do32!(544);
        do32!(576);
        do32!(608);
        do32!(640);
        do32!(672);
        do32!(704);
        do32!(736);
        do32!(768);
        do32!(800);
        do32!(832);
        do32!(864);
        do32!(896);
        do32!(928);
        do32!(960);
        do32!(992);
    }

    #[target_feature(enable = "avx2")]
    pub(super) unsafe fn _write_to(mut dst: *mut u8, mut src: *const u8, mut len: usize) {
        while len >= 32 {
            let v: __m256i = _mm256_loadu_si256(src as *const __m256i);
            _mm256_storeu_si256(dst as *mut __m256i, v);
            src = src.add(32);
            dst = dst.add(32);
            len -= 32;
        }
        while len >= 16 {
            let v: __m128i = _mm_loadu_si128(src as *const __m128i);
            _mm_storeu_si128(dst as *mut __m128i, v);
            src = src.add(16);
            dst = dst.add(16);
            len -= 16;
        }
        while len >= 8 {
            let val: u64 = core::ptr::read_unaligned(src as *const u64);
            core::ptr::write_unaligned(dst as *mut u64, val);
            src = src.add(8);
            dst = dst.add(8);
            len -= 8;
        }
        if len >= 4 {
            let val: u32 = core::ptr::read_unaligned(src as *const u32);
            core::ptr::write_unaligned(dst as *mut u32, val);
            src = src.add(4);
            dst = dst.add(4);
            len -= 4;
        }
        while len > 0 {
            *dst = *src;
            src = src.add(1);
            dst = dst.add(1);
            len -= 1;
        }
    }
}
