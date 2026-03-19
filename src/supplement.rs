pub trait Supplement {
    type ID;
    fn id_from_cmd(cmd: &[&str]) -> Option<Self::ID>;
}
