use std::error;
use std::fmt;

pub use SimdJsonError::*;

#[derive(Debug)]
pub enum SimdJsonError {
    Capacity,
    Memalloc,
    TapeError,
}

impl fmt::Display for SimdJsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                Capacity => "This ParsedJson can't support a document that big",
                Memalloc => "Error allocating memory, we're most likely out of memory",
                TapeError => "Something went wrong while writing to the tape",
            }
        )
    }
}

impl error::Error for SimdJsonError {}
