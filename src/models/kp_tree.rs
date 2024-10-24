use super::kp_group::KpGroup;

pub struct KpTree {
    pub root_group: KpGroup,
}

impl KpTree {
    pub fn new(root_group: KpGroup) -> KpTree {
        KpTree {
            root_group
        }
    }
}