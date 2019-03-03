use super::parsed_json::{ParsedJson, DEFAULT_MAX_DEPTH};

pub fn json_parse(buf: &str, pj: &mut ParsedJson, realloc_if_needed: bool) -> Result<(), String> {
    let pagesize = page_size::get();
    Ok(())
}

pub fn build_parsed_json(buf: &str, realloc_if_needed: bool) -> Result<ParsedJson, String> {
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
