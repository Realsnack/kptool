use std::fmt;

#[derive(Debug)]
pub enum KpError {
    GroupNotFound(String),
    EntryNotFound(String),
    PasswordNotFound(String),
    UsernameNotFound(String),
    TemplateVariablesNotFound(Vec<(String, KpError)>),
    NoVariablesInSourceFile(String),
}

impl fmt::Display for KpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            KpError::GroupNotFound(ref path) => write!(f, "Group '{}' not found", path),
            KpError::EntryNotFound(ref path) => write!(f, "Entry '{}' not found", path),
            KpError::PasswordNotFound(ref path) => write!(f, "Password '{}' not found for entry", path),
            KpError::UsernameNotFound(ref path) => write!(f, "Username '{}' not found for entry", path),
            KpError::TemplateVariablesNotFound(ref vec) => {
                let mut result = String::new();
                for (variable, error) in vec {
                    result.push_str(&format!("Error for template variable '{}' - {}\n", variable, error));
                }
                write!(f, "{}", result)
            },
            KpError::NoVariablesInSourceFile(ref file) => write!(f, "No variables found for replacing in template file {}", file),
        }
    }
}

impl std::error::Error for KpError {}
