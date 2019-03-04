#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

fn push_last_byte_of_a_to_b(a: __m256i, b: __m256i) -> __m256i {
    unsafe { _mm256_alignr_epi8(b, _mm256_permute2x128_si256(a, b, 0x21), 15) }
}
