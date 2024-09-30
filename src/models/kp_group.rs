use std::collections::HashMap;

use super::kp_entry::KpEntry;

pub struct KpGroup {
    pub entries: HashMap<String, KpEntry>,
    pub groups: HashMap<String, KpGroup>,
}
