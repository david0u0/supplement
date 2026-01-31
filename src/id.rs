#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Arg(u32, &'static str);
impl Arg {
    pub const fn new(id: u32, ident: &'static str) -> Self {
        Arg(id, ident)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Flag(u32, &'static str);
impl Flag {
    pub const fn new(id: u32, ident: &'static str) -> Self {
        Flag(id, ident)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Command(u32, &'static str);
impl Command {
    pub const fn new(id: u32, ident: &'static str) -> Self {
        Command(id, ident)
    }
}
