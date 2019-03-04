use super::parsed_json::{ParsedJson, JSON_VALUE_MASK};

struct ScopeIndex {
    start_of_scope: usize,
    scope_type: u8,
}

impl Default for ScopeIndex {
    fn default() -> ScopeIndex {
        ScopeIndex {
            start_of_scope: 0,
            scope_type: 0,
        }
    }
}

pub struct ParsedJsonIterator<'a> {
    pj: &'a ParsedJson,
    depth: usize,
    location: usize,
    tape_length: usize,
    current_type: u8,
    current_val: u64,
    depth_index: Vec<ScopeIndex>,
}

impl<'a> ParsedJsonIterator<'a> {
    pub fn new(pj: &'a ParsedJson) -> ParsedJsonIterator<'a> {
        let mut depth = 0;
        let mut location = 0;
        let mut tape_length = 0;
        let mut current_type = 0;
        let mut current_val = 0;
        if pj.is_valid() {
            let mut depth_index = Vec::with_capacity(pj.depth_capacity());
            if depth_index.capacity() == 0 {
                return ParsedJsonIterator {
                    pj,
                    depth,
                    location,
                    tape_length,
                    current_type,
                    current_val,
                    depth_index,
                };
            }

            let mut scope_index = ScopeIndex::default();
            scope_index.start_of_scope = location;
            current_val = pj.tape[location];
            location += 1;
            current_type = current_val.overflowing_shr(56).0 as u8;
            scope_index.scope_type = current_type;
            depth_index.push(scope_index);

            if current_type == b'r' {
                tape_length = (current_val & JSON_VALUE_MASK) as usize;
                if location < tape_length {
                    current_val = pj.tape[location];
                    current_type = current_val.overflowing_shr(56).0 as u8;
                    depth += 1;
                    depth_index[depth].start_of_scope = location;
                    depth_index[depth].scope_type = current_type;
                }
            }

            return ParsedJsonIterator {
                pj,
                depth,
                location,
                tape_length,
                current_type,
                current_val,
                depth_index,
            };
        } else {
            panic!("Json is invalid");
        }
    }

    pub fn is_ok(&self) -> bool {
        self.location < self.tape_length
    }

    /// returns the current depth (start at 1 with 0 reserved for the fictitious root node)
    pub fn get_depth(&self) -> usize {
        self.depth
    }

    pub fn move_forward(&mut self) -> bool {
        if self.location + 1 >= self.tape_length {
            return false;
        }
        true
    }
}

// impl<'a> From<&ParsedJsonIterator<>> for ParsedJsonIterator

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
