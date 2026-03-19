pub trait Supplement {
    type ID;
    fn id_from_cmd(cmd: &[&str]) -> Option<(Option<Self::ID>, u32)>;
}
