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
    mut buf: Vec<u8>,
    pj: &mut ParsedJson,
    realloc_if_needed: bool,
) -> Result<(), SimdJsonError> {
    let mut reallocated = false;
    let len = buf.len();

    if realloc_if_needed {
        let pagesize = page_size::get();

        if buf[buf.len() - 1] as usize % pagesize < SIMDJSON_PADDING {
            let tmpbuf = buf.as_ptr();
            buf = allocate_padded_buffer(buf.len());
            if buf.capacity() == 0 {
                return Err(SimdJsonError::Memalloc);
            }
            unsafe { ptr::copy_nonoverlapping(tmpbuf, buf.as_ptr() as *mut u8, len) };

            reallocated = true;

            println!("some{:?}", buf);
        }
    }

    let res = if find_structural_bits(&buf, len, pj) {
        unified_machine(&buf, len, pj)
    } else {
        Ok(())
    };

    if reallocated {
        unsafe { aligned_free(buf.as_ptr() as *mut ()) };
    }

    res
}

pub fn build_parsed_json(
    buf: Vec<u8>,
    realloc_if_needed: bool,
) -> Result<ParsedJson, SimdJsonError> {
    let mut pj = ParsedJson::new();
    let ok = pj.allocate_capacity(buf.len(), DEFAULT_MAX_DEPTH);
    if ok {
        let res = json_parse(buf, &mut pj, realloc_if_needed);
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
        let _pj = build_parsed_json(buf.as_bytes().to_vec(), true);
    }
}
