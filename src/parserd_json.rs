const JSON_VALUE_MASK: u64 = 0xFFFFFFFFFFFFFF;
const DEFAULT_MAX_DEPTH: usize = 1024;

pub struct ParsedJson {
    byte_capacity: usize,
    depth_capacity: usize,
    tape_capacity: usize,
    string_capacity: usize,
    current_loc: u32,
    n_structural_indexes: u32,
    structural_indexes: Vec<u32>,
    tape: Vec<u64>,
    containing_scope_offset: Vec<u32>,
    ret_address: Vec<char>,
    string_buf: String,
    current_string_buf_loc: String,
    is_valid: bool,
}

impl ParsedJson {
    pub fn new() -> ParsedJson {
        ParsedJson {
            byte_capacity: 0,
            depth_capacity: 0,
            tape_capacity: 0,
            string_capacity: 0,
            current_loc: 0,
            n_structural_indexes: 0,
            structural_indexes: Vec::new(),
            tape: Vec::new(),
            containing_scope_offset: Vec::new(),
            ret_address: Vec::new(),
            string_buf: String::new(),
            current_string_buf_loc: String::new(),
            is_valid: false,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    pub fn allocate_capacity(&mut self, len: usize, max_depth: usize) -> bool {
        if max_depth == 0 || len == 0 {
            eprintln!("capacities must be non-zero ");
            return false;
        }

        if len <= self.byte_capacity && self.depth_capacity < max_depth {
            return true;
        }

        self.deallocate();
        self.is_valid = false;
        self.byte_capacity = 0;
        self.n_structural_indexes = 0;
        let max_structures = roundup_n!(len, 64) + 2 + 7;
        self.structural_indexes = Vec::with_capacity(max_structures);
        let local_tape_capacity = roundup_n!(len, 64);
        let local_string_capacity = roundup_n!(len + 32, 64);
        self.string_buf = String::with_capacity(local_string_capacity);
        self.tape = Vec::with_capacity(local_tape_capacity);
        self.containing_scope_offset = Vec::with_capacity(max_depth);
        self.ret_address = Vec::with_capacity(max_depth);

        // if some attr is null { return false }

        // may not need some attr, can be get from self.attr.capacity()
        self.byte_capacity = len;
        self.depth_capacity = max_depth;
        self.tape_capacity = local_tape_capacity;
        self.string_capacity = local_string_capacity;

        // return type may be wrong
        true
    }

    pub fn init(&mut self) {
        self.current_string_buf_loc.clone_from(&self.string_buf);
        self.current_loc = 0;
        self.is_valid = false;
    }
}

impl ParsedJson {
    fn deallocate(&mut self) {
        self.byte_capacity = 0;
        self.depth_capacity = 0;
        self.tape_capacity = 0;
        self.string_capacity = 0;
        self.ret_address.clear();
        self.containing_scope_offset.clear();
        self.tape.clear();
        self.string_buf.clear();
        self.structural_indexes.clear();
        self.is_valid = false;
    }
}
