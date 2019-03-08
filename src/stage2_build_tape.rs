use super::error::SimdJsonError;
use super::parsed_json::ParsedJson;
use super::utils::char::is_not_structural_or_whitespace;
use std::mem;
use std::ptr;

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

// pub fn unified_machine(
//     buf: *const u8,
//     len: usize,
//     pj: &mut ParsedJson,
// ) -> Result<(), SimdJsonError> {
//     let mut i = 0;
//     let mut idx = 0;
//     let mut c = 0;
//     let mut depth = 0;

//     pj.init();

//     if pj.byte_capacity() < len {
//         return Err(SimdJsonError::Capacity);
//     }

//     // [TODO] computed goto

//     pj.ret_address[depth] = 's' as i8;

//     pj.containing_scope_offset[depth] = pj.get_current_loc();
//     pj.write_tape(0, b'r');
//     depth += 1;
//     if depth > pj.byte_capacity() {
//         return fail(i, idx, c, depth, buf, pj);
//     }

//     update_char(&mut i, &mut idx, &mut c, buf, pj);

//     match c {
//         b'{' => {
//             pj.containing_scope_offset[depth] = pj.get_current_loc();
//             pj.ret_address[depth] = 's' as i8;
//             depth += 1;
//             if depth > pj.depth_capacity() {
//                 return Err(SimdJsonError::TapeError);
//             }
//             pj.write_tape(0, c);
//         }
//         _ => unreachable!(),
//     }

//     pj.is_valid = true;
//     Ok(())
// }

// fn fail(
//     _i: usize,
//     _idx: u32,
//     _c: u8,
//     _depth: usize,
//     _buf: *const u8,
//     _pj: &ParsedJson,
// ) -> Result<(), SimdJsonError> {
//     Err(SimdJsonError::TapeError)
// }

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

        self.pj.is_valid = true;
        Ok(())
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
}
