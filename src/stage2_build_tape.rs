use super::error::SimdJsonError;
use super::parsed_json::ParsedJson;

pub fn unified_machine(buf: &[u8], len: usize, pj: &mut ParsedJson) -> Result<(), SimdJsonError> {
    pj.is_valid = true;
    Ok(())
}
