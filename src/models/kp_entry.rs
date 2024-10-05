pub struct KpEntry {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl KpEntry {
    pub fn new(username: Option<String>, password: Option<String>) -> KpEntry {
        KpEntry {
            username,
            password
        }
    }
}