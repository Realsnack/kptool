use std::fmt;

#[derive(Debug)]
pub enum KpError {
    GroupNotFound(String),
    EntryNotFound(String),
    PasswordNotFound(String),
}

impl fmt::Display for KpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KpError::GroupNotFound(ref path) => write!(f, "Group not found: {}", path),
            KpError::EntryNotFound(ref path) => write!(f, "Entry not found: {}", path),
            KpError::PasswordNotFound(ref path) => write!(f, "Password not found for entry: {}", path),
        }
    }
}

impl std::error::Error for KpError {}
