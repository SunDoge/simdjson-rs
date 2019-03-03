use super::parsed_json::ParsedJson;

pub fn json_parse(buf: &str, pj: &mut ParsedJson, realloc_if_needed: bool) -> Result<(), String> {
    let pagesize = page_size::get();
    Ok(())
}

pub fn build_parsed_json(buf: &str, realloc_if_needed: bool) -> Result<ParsedJson, String> {
    Ok(ParsedJson::new())
}
