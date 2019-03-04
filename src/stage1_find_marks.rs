#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use super::parsed_json::ParsedJson;

pub fn cmp_mask_against_input(input_lo: __m256i, input_hi: __m256i, mask: __m256i) -> u64 {
    unsafe {
        let cmp_res_0 = _mm256_cmpeq_epi8(input_lo, mask);
        let res_0 = _mm256_movemask_epi8(cmp_res_0) as u64;
        let cmp_res_1 = _mm256_cmpeq_epi8(input_hi, mask);
        let res_1 = _mm256_movemask_epi8(cmp_res_1) as u64;
        res_0 | res_1 << 32
    }
}

pub fn find_structural_bits(buf: &[u8], len: usize, pj: &ParsedJson) -> bool {
    if len > pj.byte_capacity() {
        eprintln!("Your ParsedJson object only supports documents up to {} bytes but you are trying to process {} bytes", 
            pj.byte_capacity(),
            len);
        return false;
    }

    let base_ptr = &pj.structural_indexes;
    let mut base = 0;

    // utf8 validate

    const EVEN_BITS: u64 = 0x5555555555555555;
    const ODD_BITS: u64 = !EVEN_BITS;

    let prev_iter_ends_odd_blackslash = 0u64;
    let prev_iter_inside_quote = 0u64;

    let prev_iter_ends_pseudo_pred = 1u64;
    let len_minus_64 = if len < 64 { 0 } else { len - 64 };
    let structurals = 0u64;

    for idx in (0..len_minus_64).step_by(64) {
        // [TODO] prefetch
    }

    true
}
