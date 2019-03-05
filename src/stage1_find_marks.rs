#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use super::parsed_json::ParsedJson;
use super::utils::{hamming, trailing_zeroes};

pub fn cmp_mask_against_input(input_lo: __m256i, input_hi: __m256i, mask: __m256i) -> u64 {
    unsafe {
        let cmp_res_0 = _mm256_cmpeq_epi8(input_lo, mask);
        let res_0 = _mm256_movemask_epi8(cmp_res_0) as u64;
        let cmp_res_1 = _mm256_cmpeq_epi8(input_hi, mask);
        let res_1 = _mm256_movemask_epi8(cmp_res_1) as u64;
        res_0 | res_1 << 32
    }
}

pub fn find_structural_bits(buf: *const u8, len: usize, pj: &mut ParsedJson) -> bool {
    if len > pj.byte_capacity() {
        eprintln!("Your ParsedJson object only supports documents up to {} bytes but you are trying to process {} bytes", 
            pj.byte_capacity(),
            len);
        return false;
    }

    let base_ptr = &mut pj.structural_indexes;
    let mut base = 0;

    // utf8 validate

    const EVEN_BITS: u64 = 0x5555555555555555;
    const ODD_BITS: u64 = !EVEN_BITS;

    let mut prev_iter_ends_odd_blackslash = 0u64;
    let prev_iter_inside_quote = 0u64;

    let prev_iter_ends_pseudo_pred = 1u64;
    let len_minus_64 = if len < 64 { 0 } else { len - 64 };
    let structurals = 0u64;

    for idx in (0..len_minus_64).step_by(64) {
        // [TODO] prefetch

        let input_lo = unsafe { _mm256_loadu_si256(buf.add(idx + 0) as *const _) };
        let input_hi = unsafe { _mm256_loadu_si256(buf.add(idx + 32) as *const _) };

        // [TODO] validate

        ////////////////////////////////////////////////////////////////////////////////////////////
        //     Step 1: detect odd sequences of backslashes
        ////////////////////////////////////////////////////////////////////////////////////////////

        let bs_bits =
            cmp_mask_against_input(input_lo, input_hi, unsafe { _mm256_set1_epi8('\\' as i8) });
        let start_edges = bs_bits & !(bs_bits << 1);
        // flip lowest if we have an odd-length run at the end of the prior
        // iteration
        let even_start_mask = EVEN_BITS ^ prev_iter_ends_odd_blackslash;
        let even_starts = start_edges & even_start_mask;
        let odd_starts = start_edges & !even_start_mask;
        let even_carries = bs_bits + even_starts;

        let (mut odd_carries, iter_ends_odd_blackslash) = bs_bits.overflowing_add(odd_starts);
        // push in bit zero as a potential end
        // if we had an odd-numbered run at the
        // end of the previous iteration
        odd_carries |= prev_iter_ends_odd_blackslash;

        prev_iter_ends_odd_blackslash = if iter_ends_odd_blackslash { 0x1 } else { 0x0 };
        let even_carry_ends = even_carries & !bs_bits;
        let odd_carry_ends = odd_carries & !bs_bits;
        let even_start_odd_end = even_carry_ends & ODD_BITS;
        let odd_start_even_end = odd_carry_ends & EVEN_BITS;
        let odd_ends = even_start_odd_end | odd_start_even_end;

        ////////////////////////////////////////////////////////////////////////////////////////////
        //     Step 2: detect insides of quote pairs
        ////////////////////////////////////////////////////////////////////////////////////////////

        let mut quote_bits =
            cmp_mask_against_input(input_lo, input_hi, unsafe { _mm256_set1_epi8('"' as i8) });

        quote_bits = quote_bits & !odd_ends;
        let quote_mask = unsafe {
            _mm_cvtsi128_si64(_mm_clmulepi64_si128(
                _mm_set_epi64x(0, quote_bits as i64),
                // _mm_set1_epi8(0xFF),
                _mm_set1_epi8(-1), // 0xFF overflow to -1
                0,
            ))
        };

        let cnt = hamming(structurals);
        let next_base = base + cnt;

        while structurals != 0u64 {
            let base_usize = base as usize;
            base_ptr[base_usize + 0] = idx as u32 - 64 + trailing_zeroes(structurals);
        }
    }

    true
}
