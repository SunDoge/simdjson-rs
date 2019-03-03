use super::error::SimdJsonError;
use super::parsed_json::{ParsedJson, DEFAULT_MAX_DEPTH};
use super::utils::SIMDJSON_PADDING;
use aligned_alloc::{aligned_alloc, aligned_free};
use std::mem;
use std::borrow::{Cow};


pub fn json_parse<'a, S>(
    buf: S,
    pj: &mut ParsedJson,
    realloc_if_needed: bool,
) -> Result<(), SimdJsonError> 
where S: Into<Cow<'a, str>>
{   
    let buf=buf.into();
    let mut reallocated = false;

    if realloc_if_needed {
        let pagesize = page_size::get();

        if buf.as_bytes()[buf.len() - 1] as usize % pagesize < SIMDJSON_PADDING {
            let tmpbuf = buf.as_ptr();
            buf = Cow::from()

            println!("some{}", buf);
        }
    }

    Ok(())
}

pub fn build_parsed_json(buf: &str, realloc_if_needed: bool) -> Result<ParsedJson, SimdJsonError> {
    let mut pj = ParsedJson::new();
    let ok = pj.allocate_capacity(buf.len(), DEFAULT_MAX_DEPTH);
    if ok {
        let res = json_parse(buf, &mut pj, realloc_if_needed);
    // assert_eq!(res.is_ok(), pj.is_valid());
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
        let _pj = build_parsed_json(buf, true);
    }
}
