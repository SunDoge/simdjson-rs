use super::error::SimdJsonError;
use super::parsed_json::{ParsedJson, DEFAULT_MAX_DEPTH};
use super::stage1_find_marks::find_structural_bits;
use super::stage2_build_tape::unified_machine;
use super::utils::{allocate_padded_buffer, SIMDJSON_PADDING};

use aligned_alloc::aligned_free;
use std::borrow::Cow;
use std::mem;
use std::ptr;

// pub fn json_parse<'a, S>(
//     buf: S,
//     pj: &mut ParsedJson,
//     realloc_if_needed: bool,
// ) -> Result<(), SimdJsonError>
// where
//     S: Into<Cow<'a, str>>,
// {

pub fn json_parse(
    buf: *const u8,
    len: usize,
    pj: &mut ParsedJson,
    realloc_if_needed: bool,
) -> Result<(), SimdJsonError> {
    if pj.byte_capacity() < len {
        return Err(SimdJsonError::Capacity);
    }

    if realloc_if_needed {
        let pagesize = page_size::get();

        if unsafe { buf.add(len - 1) } as usize % pagesize < SIMDJSON_PADDING {
            println!("some{:?}", buf);

            let new_buf = allocate_padded_buffer(len);

            if new_buf.is_null() {
                return Err(SimdJsonError::Memalloc);
            }

            unsafe { ptr::copy_nonoverlapping(buf, new_buf, len) };

            let res = if find_structural_bits(new_buf, len, pj) {
                unified_machine(new_buf, len, pj)
            } else {
                Ok(())
            };

            unsafe { aligned_free(new_buf as *mut ()) };
            return res;
        }
    }

    if find_structural_bits(buf, len, pj) {
        unified_machine(buf, len, pj)
    } else {
        Ok(())
    }
}

pub fn build_parsed_json(
    // buf: Vec<u8>,
    buf: *const u8,
    len: usize,
    realloc_if_needed: bool,
) -> Result<ParsedJson, SimdJsonError> {
    let mut pj = ParsedJson::new();
    let ok = pj.allocate_capacity(len, DEFAULT_MAX_DEPTH);
    if ok {
        let res = json_parse(buf, len, &mut pj, realloc_if_needed);
        assert_eq!(res.is_ok(), pj.is_valid());
    } else {
        eprintln!("failure during memory allocation ");
    }
    Ok(pj)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let buf = r#"{"a": "b"}"#;
        let _pj = build_parsed_json(buf.as_ptr(), buf.len(), true);
    }

    #[test]
    fn realloc() {
        let mut s = vec![1, 2, 3];
        let mut s1 = vec![2, 3];
        println!("{}", s[1]);
        println!("{:?}", s.as_ptr());
        println!("{:x}", s.as_ptr() as usize);
    }
}
