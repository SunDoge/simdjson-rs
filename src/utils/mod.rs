pub mod char;
pub mod format;

#[cfg(target_arch = "x86")]
use std::arch::x86::__m256i;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::__m256i;

use aligned_alloc::aligned_alloc;
use std::mem;

pub const SIMDJSON_PADDING: usize = mem::size_of::<__m256i>(); // 32

macro_rules! roundup_n {
    ($a:expr, $n:expr) => {
        // origin: (((a) + ((n)-1)) & ~((n)-1))
        // https://github.com/lemire/simdjson/blob/master/include/simdjson/common_defs.h#L21
        (($a) + (($n) - 1)) & !(($n) - 1)
    };
}

pub fn allocate_padded_buffer(length: usize) -> Vec<u8> {
    let total_padded_length = length + SIMDJSON_PADDING;
    let padded_buffer = aligned_alloc(total_padded_length, 64) as *mut u8;
    unsafe { Vec::from_raw_parts(padded_buffer, total_padded_length, total_padded_length) }
}
