#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use super::parsed_json::ParsedJson;
use super::utils::{hamming, trailing_zeroes};
use std::ptr;

pub fn cmp_mask_against_input(input_lo: __m256i, input_hi: __m256i, mask: __m256i) -> i64 {
    unsafe {
        let cmp_res_0 = _mm256_cmpeq_epi8(input_lo, mask);
        let res_0 = _mm256_movemask_epi8(cmp_res_0) as i64;
        let cmp_res_1 = _mm256_cmpeq_epi8(input_hi, mask);
        let res_1 = _mm256_movemask_epi8(cmp_res_1) as i64;
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

    let base_ptr = pj.structural_indexes.as_mut_ptr();
    let mut base = 0u32;

    // utf8 validate

    let even_bits: i64 = 0x5555555555555555;
    let odd_bits = !even_bits;

    let mut prev_iter_ends_odd_blackslash = 0i64;
    let mut prev_iter_inside_quote = 0i64;

    let mut prev_iter_ends_pseudo_pred = 1i64;
    let len_minus_64 = if len < 64 { 0 } else { len - 64 };
    let mut idx = 0usize;
    let mut structurals = 0i64;

    while idx < len_minus_64 {
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
        let even_start_mask = even_bits ^ prev_iter_ends_odd_blackslash;
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
        let even_start_odd_end = even_carry_ends & odd_bits;
        let odd_start_even_end = odd_carry_ends & even_bits;
        let odd_ends = even_start_odd_end | odd_start_even_end;

        ////////////////////////////////////////////////////////////////////////////////////////////
        //     Step 2: detect insides of quote pairs
        ////////////////////////////////////////////////////////////////////////////////////////////

        let mut quote_bits =
            cmp_mask_against_input(input_lo, input_hi, unsafe { _mm256_set1_epi8('"' as i8) });

        quote_bits = quote_bits & !odd_ends;
        let mut quote_mask = unsafe {
            _mm_cvtsi128_si64(_mm_clmulepi64_si128(
                _mm_set_epi64x(0, quote_bits as i64),
                // _mm_set1_epi8(0xFF),
                _mm_set1_epi8(-1), // 0xFF overflow to -1
                0,
            ))
        } as i64;

        let cnt = hamming(structurals);
        let next_base = base + cnt;

        while structurals != 0i64 {
            let base_isize = base as isize;
            unsafe {
                *base_ptr.offset(base_isize + 0) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 1) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 2) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 3) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 4) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 5) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 6) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 7) = idx as u32 - 64 + trailing_zeroes(structurals);
            }
            structurals = structurals & (structurals - 1);
            base += 8;
        }

        base = next_base;

        quote_mask ^= prev_iter_inside_quote;
        prev_iter_inside_quote = quote_mask as i64 >> 63;

        let low_nibble_mask: __m256i = unsafe {
            _mm256_setr_epi8(
                //  0                           9  a   b  c  d
                16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2, 9, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12,
                1, 2, 9, 0, 0,
            )
        };

        let high_nibble_mask: __m256i = unsafe {
            _mm256_setr_epi8(
                //  0     2   3     5     7
                8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1, 0, 0, 8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0,
                3, 2, 1, 0, 0,
            )
        };

        let structural_shufti_mask = unsafe { _mm256_set1_epi8(0x7) };
        let whitespace_shufti_mask = unsafe { _mm256_set1_epi8(0x18) };

        let v_lo = unsafe {
            _mm256_and_si256(
                _mm256_shuffle_epi8(low_nibble_mask, input_lo),
                _mm256_shuffle_epi8(
                    high_nibble_mask,
                    _mm256_and_si256(_mm256_srli_epi32(input_lo, 4), _mm256_set1_epi8(0x7f)),
                ),
            )
        };

        let v_hi = unsafe {
            _mm256_and_si256(
                _mm256_shuffle_epi8(low_nibble_mask, input_hi),
                _mm256_shuffle_epi8(
                    high_nibble_mask,
                    _mm256_and_si256(_mm256_srli_epi32(input_hi, 4), _mm256_set1_epi8(0x7f)),
                ),
            )
        };

        let tmp_lo = unsafe {
            _mm256_cmpeq_epi8(
                _mm256_and_si256(v_lo, structural_shufti_mask),
                _mm256_set1_epi8(0),
            )
        };

        let tmp_hi = unsafe {
            _mm256_cmpeq_epi8(
                _mm256_and_si256(v_hi, structural_shufti_mask),
                _mm256_set1_epi8(0),
            )
        };

        let structural_res_0 = unsafe { _mm256_movemask_epi8(tmp_lo) } as u64;
        let structural_res_1 = unsafe { _mm256_movemask_epi8(tmp_hi) } as u64;

        let tmp_ws_lo = unsafe {
            _mm256_cmpeq_epi8(
                _mm256_and_si256(v_lo, whitespace_shufti_mask),
                _mm256_set1_epi8(0),
            )
        };
        let tmp_ws_hi = unsafe {
            _mm256_cmpeq_epi8(
                _mm256_and_si256(v_hi, whitespace_shufti_mask),
                _mm256_set1_epi8(0),
            )
        };

        let ws_res_0 = unsafe { _mm256_movemask_epi8(tmp_ws_lo) } as i64;
        let ws_res_1 = unsafe { _mm256_movemask_epi8(tmp_ws_hi) } as i64;
        let whitespace = !(ws_res_0 | (ws_res_1 << 32));

        structurals &= !quote_mask;
        structurals |= quote_bits;

        let pseudo_pred = structurals | whitespace;
        let shifted_pseudo_pred = (pseudo_pred << 1) | prev_iter_ends_pseudo_pred;
        prev_iter_ends_pseudo_pred = pseudo_pred >> 63;
        let pseudo_structurals = shifted_pseudo_pred & (!whitespace) & (!quote_mask);
        structurals |= pseudo_structurals;

        structurals &= !(quote_bits & !quote_mask);

        idx += 64;
    }

    if idx < len {
        let tmpbuf = [0x20u8; 64].as_mut_ptr();
        unsafe {
            ptr::copy_nonoverlapping(buf.add(idx), tmpbuf, len - idx);
        }

        let input_lo = unsafe { _mm256_loadu_si256(tmpbuf.add(0) as *const __m256i) };
        let input_hi = unsafe { _mm256_loadu_si256(tmpbuf.add(32) as *const __m256i) };

        // [TODO] validate

        ////////////////////////////////////////////////////////////////////////////////////////////
        //     Step 1: detect odd sequences of backslashes
        ////////////////////////////////////////////////////////////////////////////////////////////

        let bs_bits =
            cmp_mask_against_input(input_lo, input_hi, unsafe { _mm256_set1_epi8('\\' as i8) });
        let start_edges = bs_bits & !(bs_bits << 1);
        // flip lowest if we have an odd-length run at the end of the prior
        // iteration
        let even_start_mask = even_bits ^ prev_iter_ends_odd_blackslash;
        let even_starts = start_edges & even_start_mask;
        let odd_starts = start_edges & !even_start_mask;
        let even_carries = bs_bits + even_starts;

        let (mut odd_carries, _) = bs_bits.overflowing_add(odd_starts);
        // push in bit zero as a potential end
        // if we had an odd-numbered run at the
        // end of the previous iteration
        odd_carries |= prev_iter_ends_odd_blackslash;

        // prev_iter_ends_odd_blackslash = if iter_ends_odd_blackslash { 0x1 } else { 0x0 };
        let even_carry_ends = even_carries & !bs_bits;
        let odd_carry_ends = odd_carries & !bs_bits;
        let even_start_odd_end = even_carry_ends & odd_bits;
        let odd_start_even_end = odd_carry_ends & even_bits;
        let odd_ends = even_start_odd_end | odd_start_even_end;

        ////////////////////////////////////////////////////////////////////////////////////////////
        //     Step 2: detect insides of quote pairs
        ////////////////////////////////////////////////////////////////////////////////////////////

        let mut quote_bits =
            cmp_mask_against_input(input_lo, input_hi, unsafe { _mm256_set1_epi8('"' as i8) });

        quote_bits = quote_bits & !odd_ends;
        let mut quote_mask = unsafe {
            _mm_cvtsi128_si64(_mm_clmulepi64_si128(
                _mm_set_epi64x(0, quote_bits as i64),
                // _mm_set1_epi8(0xFF),
                _mm_set1_epi8(-1), // 0xFF overflow to -1
                0,
            ))
        } as i64;

        quote_mask ^= prev_iter_inside_quote;

        let cnt = hamming(structurals);
        let next_base = base + cnt;

        while structurals != 0i64 {
            let base_isize = base as isize;
            unsafe {
                *base_ptr.offset(base_isize + 0) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 1) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 2) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 3) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 4) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 5) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 6) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                *base_ptr.offset(base_isize + 7) = idx as u32 - 64 + trailing_zeroes(structurals);
                structurals = structurals & (structurals - 1);
                base += 8;
            }
        }

        base = next_base;

        quote_mask ^= prev_iter_inside_quote;
        prev_iter_inside_quote = quote_mask as i64 >> 63;

        let low_nibble_mask: __m256i = unsafe {
            _mm256_setr_epi8(
                //  0                           9  a   b  c  d
                16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12, 1, 2, 9, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 8, 12,
                1, 2, 9, 0, 0,
            )
        };

        let high_nibble_mask: __m256i = unsafe {
            _mm256_setr_epi8(
                //  0     2   3     5     7
                8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0, 3, 2, 1, 0, 0, 8, 0, 18, 4, 0, 1, 0, 1, 0, 0, 0,
                3, 2, 1, 0, 0,
            )
        };

        let structural_shufti_mask = unsafe { _mm256_set1_epi8(0x7) };
        let whitespace_shufti_mask = unsafe { _mm256_set1_epi8(0x18) };

        let v_lo = unsafe {
            _mm256_and_si256(
                _mm256_shuffle_epi8(low_nibble_mask, input_lo),
                _mm256_shuffle_epi8(
                    high_nibble_mask,
                    _mm256_and_si256(_mm256_srli_epi32(input_lo, 4), _mm256_set1_epi8(0x7f)),
                ),
            )
        };

        let v_hi = unsafe {
            _mm256_and_si256(
                _mm256_shuffle_epi8(low_nibble_mask, input_hi),
                _mm256_shuffle_epi8(
                    high_nibble_mask,
                    _mm256_and_si256(_mm256_srli_epi32(input_hi, 4), _mm256_set1_epi8(0x7f)),
                ),
            )
        };

        let tmp_lo = unsafe {
            _mm256_cmpeq_epi8(
                _mm256_and_si256(v_lo, structural_shufti_mask),
                _mm256_set1_epi8(0),
            )
        };

        let tmp_hi = unsafe {
            _mm256_cmpeq_epi8(
                _mm256_and_si256(v_hi, structural_shufti_mask),
                _mm256_set1_epi8(0),
            )
        };

        let structural_res_0 = unsafe { _mm256_movemask_epi8(tmp_lo) } as i64;
        let structural_res_1 = unsafe { _mm256_movemask_epi8(tmp_hi) } as i64;
        structurals = !(structural_res_0 | (structural_res_1 << 32));

        // [TODO] whitespace
        let tmp_ws_lo = unsafe {
            _mm256_cmpeq_epi8(
                _mm256_and_si256(v_lo, whitespace_shufti_mask),
                _mm256_set1_epi8(0),
            )
        };
        let tmp_ws_hi = unsafe {
            _mm256_cmpeq_epi8(
                _mm256_and_si256(v_hi, whitespace_shufti_mask),
                _mm256_set1_epi8(0),
            )
        };

        let ws_res_0 = unsafe { _mm256_movemask_epi8(tmp_ws_lo) } as i64;
        let ws_res_1 = unsafe { _mm256_movemask_epi8(tmp_ws_hi) } as i64;
        let whitespace = !(ws_res_0 | (ws_res_1 << 32));

        structurals &= !quote_mask;
        structurals |= quote_bits;

        let pseudo_pred = structurals | whitespace;
        let shifted_pseudo_pred = (pseudo_pred << 1) | prev_iter_ends_pseudo_pred;
        prev_iter_ends_pseudo_pred = pseudo_pred >> 63;
        let pseudo_structurals = shifted_pseudo_pred & (!whitespace) & (!quote_mask);
        structurals |= pseudo_structurals;

        structurals &= !(quote_bits & !quote_mask);

        idx += 64;
    }

    let cnt = hamming(structurals);
    let next_base = base + cnt;

    while structurals != 0i64 {
        let base_isize = base as isize;
        unsafe {
            *base_ptr.offset(base_isize + 0) = idx as u32 - 64 + trailing_zeroes(structurals);
            structurals = structurals & (structurals - 1);
            *base_ptr.offset(base_isize + 1) = idx as u32 - 64 + trailing_zeroes(structurals);
            structurals = structurals & (structurals - 1);
            *base_ptr.offset(base_isize + 2) = idx as u32 - 64 + trailing_zeroes(structurals);
            structurals = structurals & (structurals - 1);
            *base_ptr.offset(base_isize + 3) = idx as u32 - 64 + trailing_zeroes(structurals);
            structurals = structurals & (structurals - 1);
            *base_ptr.offset(base_isize + 4) = idx as u32 - 64 + trailing_zeroes(structurals);
            structurals = structurals & (structurals - 1);
            *base_ptr.offset(base_isize + 5) = idx as u32 - 64 + trailing_zeroes(structurals);
            structurals = structurals & (structurals - 1);
            *base_ptr.offset(base_isize + 6) = idx as u32 - 64 + trailing_zeroes(structurals);
            structurals = structurals & (structurals - 1);
            *base_ptr.offset(base_isize + 7) = idx as u32 - 64 + trailing_zeroes(structurals);
            structurals = structurals & (structurals - 1);
            base += 8;
        }
    }

    base = next_base;

    let n_structural_indexes = base as isize;

    if n_structural_indexes == 0 {
        // pj.n_structural_indexes = base;
        return false;
    }

    if unsafe { *base_ptr.offset(n_structural_indexes - 1) } as usize > len {
        eprintln!("Internal bug");
        return false;
    }

    if len != unsafe { *base_ptr.offset(n_structural_indexes - 1) } as usize {
        pj.structural_indexes.push(len as u32);
    }

    pj.structural_indexes.push(0);

    true
}
