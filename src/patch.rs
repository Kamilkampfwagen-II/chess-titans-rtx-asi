use std::{error::Error, fmt};


pub struct Patch {
    pub offset: u32,
    pub org: u8,
    pub new: u8
}


#[derive(Debug)]
pub enum PatchError {
    ByteMismatch(u32, u8, u8),
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PatchError::ByteMismatch(offset, expected, found) => 
                write!(f, "Byte {:#x} at target offset {:#x} does not match the original byte {:#x} in the patch. The patch may be intended for a different executable.", found, offset, expected),
        }
    }
}

impl Error for PatchError {}