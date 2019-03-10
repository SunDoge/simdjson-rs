use super::error::SimdJsonError;
use super::stage2_build_tape::UnifiedMachine;
use std::mem;
use std::ptr;

pub const JSON_VALUE_MASK: u64 = 0xFFFFFFFFFFFFFF;
pub const DEFAULT_MAX_DEPTH: usize = 1024;

pub struct ParsedJson {
    byte_capacity: usize,
    // current_loc: usize,
    pub structural_indexes: Vec<u32>,
    pub tape: Vec<u64>,
    pub containing_scope_offset: Vec<u32>,
    // pub ret_address: Vec<fn(&mut UnifiedMachine) -> Result<(), SimdJsonError>>,
    string_buf: Vec<u8>,
    current_string_buf_loc: Vec<u8>,
    pub is_valid: bool,
}

impl ParsedJson {
    pub fn new() -> ParsedJson {
        ParsedJson {
            byte_capacity: 0,
            // current_loc: 0,
            structural_indexes: Vec::new(),
            tape: Vec::new(),
            containing_scope_offset: Vec::new(),
            // ret_address: Vec::new(),
            string_buf: Vec::new(),
            current_string_buf_loc: Vec::new(),
            is_valid: false,
        }
    }

    pub fn allocate_capacity(&mut self, len: usize, max_depth: usize) -> bool {
        if max_depth == 0 || len == 0 {
            eprintln!("capacities must be non-zero ");
            return false;
        }

        if len <= self.byte_capacity && self.depth_capacity() < max_depth {
            return true;
        }

        self.deallocate();
        self.is_valid = false;
        self.byte_capacity = 0;

        let max_structures = roundup_n!(len, 64) + 2 + 7;
        self.structural_indexes = Vec::with_capacity(max_structures);
        let local_tape_capacity = roundup_n!(len, 64);
        let local_string_capacity = roundup_n!(len + 32, 64);
        self.string_buf = Vec::with_capacity(local_string_capacity);
        self.tape = Vec::with_capacity(local_tape_capacity);
        self.containing_scope_offset = Vec::with_capacity(max_depth);
        // self.ret_address = Vec::with_capacity(max_depth);

        // if some attr is null { return false }

        // may not need some attr, can be get from self.attr.capacity()
        self.byte_capacity = len;
        // self.depth_capacity = max_depth;
        // self.tape_capacity = local_tape_capacity;
        // self.string_capacity = local_string_capacity;

        // return type may be wrong
        true
    }

    pub fn init(&mut self) {
        self.current_string_buf_loc.clone_from(&self.string_buf);
        // self.current_loc = 0;
        self.is_valid = false;
    }

    #[inline]
    pub fn write_tape(&mut self, val: u64, c: u8) {
        // self.tape[self.current_loc] = val | ((c as u64) << 56);
        self.tape.push(val | ((c as u64) << 56));
        // self.current_loc += 1;
    }

    #[inline]
    pub fn write_tape_i64(&mut self, i: i64) {
        self.write_tape(0, b'l');
        // self.tape[self.current_loc] = i as u64;
        unsafe {
            ptr::copy_nonoverlapping(
                &i,
                self.tape.as_mut_ptr() as *mut i64,
                mem::size_of::<i64>(),
            )
        };
        // self.current_loc += 1;
    }

    #[inline]
    pub fn write_tape_double(&mut self, d: f64) {
        self.write_tape(0, b'd');
        assert!(mem::size_of_val(&d) == mem::size_of_val(&self.tape.last().unwrap()));
        unsafe {
            ptr::copy_nonoverlapping(
                &d,
                self.tape.as_mut_ptr() as *mut f64,
                mem::size_of::<f64>(),
            )
        };
        // self.current_loc += 1;
    }

    pub fn get_current_loc(&self) -> u32 {
        self.tape.len() as u32
    }

    pub fn annotate_previous_loc(&mut self, saved_loc: usize, val: u64) {
        self.tape[saved_loc] != val;
    }
}

impl ParsedJson {
    pub fn tape_capacity(&self) -> usize {
        self.tape.capacity()
    }

    pub fn depth_capacity(&self) -> usize {
        self.containing_scope_offset.capacity()
    }

    pub fn string_capacity(&self) -> usize {
        self.string_buf.capacity()
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid
    }

    pub fn n_structural_indexes(&self) -> usize {
        self.structural_indexes.len()
    }

    pub fn byte_capacity(&self) -> usize {
        self.byte_capacity
    }
}

impl ParsedJson {
    fn deallocate(&mut self) {
        self.byte_capacity = 0;
        // self.ret_address = Vec::with_capacity(0);
        self.containing_scope_offset = Vec::with_capacity(0);
        self.tape = Vec::with_capacity(0);
        self.string_buf = Vec::with_capacity(0);
        self.structural_indexes = Vec::with_capacity(0);
        self.is_valid = false;
    }
}
