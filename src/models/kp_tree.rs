use super::kp_group::KpGroup;

pub struct KpTree {
    pub root_group: KpGroup,
    pub file_hash: Option<String>,
}

impl KpTree {
    pub fn new(root_group: KpGroup, file_hash: String) -> KpTree {
        KpTree {
            root_group,
            file_hash: Some(file_hash),
        }
    }
}