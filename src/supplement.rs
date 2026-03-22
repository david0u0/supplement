use std::fmt::Debug;

pub trait Supplement {
    type ID: Debug;
    fn id_from_cmd(cmd: &[&str]) -> Option<(Option<Self::ID>, u32)>;
}
