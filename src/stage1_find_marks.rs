#[cfg(target_arch = "x86")]
use std::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

use super::parsed_json::ParsedJson;

pub fn cmp_mask_against_input(input_lo: __m256i, input_hi: __m256i, mask: __m256i) -> u64 {
    0
}

pub fn find_structural_bits(buf: &[u8], len: usize, pj: &ParsedJson) -> bool {
    true
}
