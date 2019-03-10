use super::error::SimdJsonError;
use super::parsed_json::ParsedJson;
use super::utils::char::is_not_structural_or_whitespace;
use std::mem;
use std::ptr;

use super::utils::string_parsing_avx2::parse_string;
use super::utils::SIMDJSON_PADDING;

// macro_rules! update_char {
//     ($i: ) => {
//         idx = pj.structural_indexes[i];
//         i += 1;
//         unsafe { c = *buf.add(idx) }
//     };
// }

fn update_char(i: &mut usize, idx: &mut u32, c: &mut u8, buf: *const u8, pj: &ParsedJson) {
    *idx = pj.structural_indexes[*i];
    *i += 1;
    unsafe { *c = *buf.offset(*idx as isize) }
}

fn is_valid_true_atom(loc: *const u8) -> bool {
    let tv = "true    ".as_ptr() as u64;
    let mask4 = 0x00000000ffffffffu64;
    let mut error = 0u32;
    let mut locval = 0u64;

    unsafe {
        ptr::copy_nonoverlapping(loc, locval as *mut u8, mem::size_of::<u64>());
    }

    error = ((locval & mask4) ^ tv) as u32;

    error |= is_not_structural_or_whitespace(unsafe { *loc.offset(4) });

    error == 0
}

fn is_valid_false_atom(loc: *const u8) -> bool {
    let fv = "false   ".as_ptr() as u64;
    let mask5 = 0x000000ffffffffffu64;
    let mut error = 0u32;
    let mut locval = 0u64;

    unsafe {
        ptr::copy_nonoverlapping(loc, locval as *mut u8, mem::size_of::<u64>());
    }

    error = ((locval & mask5) ^ fv) as u32;

    error |= is_not_structural_or_whitespace(unsafe { *loc.offset(5) });

    error == 0
}

fn is_valid_null_atom(loc: *const u8) -> bool {
    let nv = "null    ".as_ptr() as u64;
    let mask4 = 0x00000000ffffffffu64;
    let mut error = 0u32;
    let mut locval = 0u64;

    unsafe {
        ptr::copy_nonoverlapping(loc, locval as *mut u8, mem::size_of::<u64>());
    }

    error = ((locval & mask4) ^ nv) as u32;

    error |= is_not_structural_or_whitespace(unsafe { *loc.offset(4) });

    error == 0
}

pub fn unified_machine(
    buf: *const u8,
    len: usize,
    pj: &mut ParsedJson,
) -> Result<(), SimdJsonError> {
    UnifiedMachine::new(buf, len, pj).run()
}

pub struct UnifiedMachine<'a> {
    i: usize,     // index of the structural character (0,1,2,3...)
    idx: isize,   // location of the structural character in the input (buf)
    c: u8,        // used to track the (structural) character we are looking at
    depth: usize, // could have an arbitrary starting depth
    buf: *const u8,
    len: usize,
    pj: &'a mut ParsedJson,
    ret_address: Vec<fn(&mut UnifiedMachine<'a>) -> Result<(), SimdJsonError>>,
}

impl<'a> UnifiedMachine<'a> {
    pub fn new(buf: *const u8, len: usize, pj: &'a mut ParsedJson) -> UnifiedMachine<'a> {
        UnifiedMachine {
            i: 0,
            idx: 0,
            c: 0,
            depth: 0,
            buf,
            len,
            pj,
            ret_address: Vec::new(),
        }
    }

    pub fn run(&mut self) -> Result<(), SimdJsonError> {
        self.pj.init();
        if self.pj.byte_capacity() < self.len {
            return Err(SimdJsonError::Capacity);
        }

        self.ret_address = Vec::with_capacity(self.pj.depth_capacity());
        // self.ret_address[self.depth] = Self::start_continue;
        self.ret_address.push(Self::start_continue);
        self.pj
            .containing_scope_offset
            .push(self.pj.get_current_loc());
        self.pj.write_tape(0, b'r');
        self.depth += 1;

        if self.depth > self.pj.depth_capacity() {
            return self.fail();
        }

        self.update_char();

        match self.c {
            b'{' => {
                self.pj
                    .containing_scope_offset
                    .push(self.pj.get_current_loc());
                self.ret_address.push(Self::start_continue);
                self.depth += 1;
                if self.depth > self.pj.depth_capacity() {
                    return self.fail();
                }

                self.pj.write_tape(0, self.c);
                self.object_begin()
            }
            b'[' => {
                self.pj
                    .containing_scope_offset
                    .push(self.pj.get_current_loc());
                self.ret_address.push(Self::start_continue);
                self.depth += 1;
                if self.depth > self.pj.depth_capacity() {
                    return self.fail();
                }

                self.pj.write_tape(0, self.c);
                self.array_begin()
            }
            b'"' => {
                if !parse_string(self.buf, self.len, self.pj, self.depth, self.idx) {
                    return self.fail();
                }
                Ok(())
            }
            b't' => {
                let mut copy = Vec::with_capacity(self.len + SIMDJSON_PADDING);
                unsafe {
                    ptr::copy_nonoverlapping(self.buf, copy.as_mut_ptr(), self.len);
                }
                copy[self.len] = b'\0';
                if !is_valid_true_atom(unsafe { copy.as_ptr().offset(self.idx) }) {
                    return self.fail();
                }

                self.pj.write_tape(0, self.c);
                Ok(())
            }
            b'f' => Ok(()),
            b'n' => Ok(()),
            b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => Ok(()),
            b'-' => Ok(()),
            _ => self.fail(),
        }
    }

    fn update_char(&mut self) {
        self.idx = self.pj.structural_indexes[self.i] as isize;
        self.i += 1;
        unsafe {
            self.c = *self.buf.offset(self.idx);
        }
    }

    fn fail(&mut self) -> Result<(), SimdJsonError> {
        Err(SimdJsonError::TapeError)
    }

    fn succeed(&mut self) -> Result<(), SimdJsonError> {
        self.depth -= 1;
        if self.depth != 0 {
            panic!("internal bug");
        }

        if self.pj.containing_scope_offset[self.depth] != 0 {
            panic!("internal bug");
        }

        self.pj.annotate_previous_loc(
            self.pj.containing_scope_offset[self.depth] as usize,
            self.pj.get_current_loc() as u64,
        );

        self.pj
            .write_tape(self.pj.containing_scope_offset[self.depth] as u64, b'r');

        self.pj.is_valid = true;
        Ok(())
    }

    fn start_continue(&mut self) -> Result<(), SimdJsonError> {
        if self.i + 1 == self.pj.n_structural_indexes() {
            self.succeed()
        } else {
            self.fail()
        }
    }

    fn object_begin(&mut self) -> Result<(), SimdJsonError> {
        self.update_char();
        match self.c {
            b'"' => {
                if !parse_string(self.buf, self.len, self.pj, self.depth, self.idx) {
                    return self.fail();
                }

                self.object_key_state()
            }
            b'}' => self.scope_end(),
            _ => self.fail(),
        }
    }

    fn object_key_state(&mut self) -> Result<(), SimdJsonError> {
        self.update_char();

        if self.c != b':' {
            return self.fail();
        }

        self.update_char();

        match self.c {
            _ => return self.fail(),
        }
    }

    fn object_continue(&mut self) -> Result<(), SimdJsonError> {
        self.update_char();

        match self.c {
            _ => return self.fail(),
        }
    }

    fn scope_end(&mut self) -> Result<(), SimdJsonError> {
        self.depth -= 1;
        self.pj
            .write_tape(self.pj.containing_scope_offset[self.depth] as u64, self.c);
        self.pj.annotate_previous_loc(
            self.pj.containing_scope_offset[self.depth] as usize,
            self.pj.get_current_loc() as u64,
        );
        self.ret_address[self.depth](self)
    }

    fn array_begin(&mut self) -> Result<(), SimdJsonError> {
        self.update_char();
        if self.c == b'[' {
            return self.scope_end();
        }
        Ok(())
    }

    fn main_array_switch(&mut self) -> Result<(), SimdJsonError> {
        match self.c {
            _ => self.fail(),
        }
    }

    fn array_continue(&mut self) -> Result<(), SimdJsonError> {
        self.update_char();
        match self.c {
            b',' => {
                self.update_char();
                self.main_array_switch()
            }
            b']' => self.scope_end(),
            _ => self.fail(),
        }
    }
}
