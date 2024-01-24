pub mod patch {
    use std::{error::Error, fmt};


    pub struct Patch {
        pub offset: u32,
        pub org: u8,
        pub new: u8
    }


    #[derive(Debug)]
    pub enum PatchError {
        ByteMismatch,
    }
    
    impl fmt::Display for PatchError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                PatchError::ByteMismatch => write!(f, "Byte at target offset does not match the original byte in the patch. The patch may be intended for a different executable."),
            }
        }
    }
    
    impl Error for PatchError {}
}