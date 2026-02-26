/// Id for things that cannot have value.
/// All subcommand belong to this, and flags can also have no value.
///
/// When searching for it in `History`, it will still have an integer value `count`,
/// which represents how many times it's seen in the CLI command
/// ```no_run
/// use supplement::{History, id};
/// let history = History::<()>::new();
/// let id = id::NoVal::new(0);
/// let c: u32 = history.find(id).unwrap().count; // Represents how many times it's seen in the CLI command
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NoVal(u32);

/// Id for things that have at most one value.
/// When searching for it in `History`, it will have a single string `value`
/// ```no_run
/// use supplement::{History, id};
/// let history = History::new();
/// let id = id::SingleVal::new(());
/// let v: &str = &history.find(id).unwrap().value;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SingleVal<ID> {
    Uncertain(ID),
    Certain(u32),
}

/// Id for things that can have more than one value.
/// When searching for it in `History`, it will have a vector of string `values`
/// ```no_run
/// use supplement::{History, id};
/// let history = History::new();
/// let id = id::MultiVal::new(());
/// let v: &[String] = &history.find(id).unwrap().values;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MultiVal<ID> {
    Uncertain(ID),
    Certain(u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Valued<ID> {
    Single(SingleVal<ID>),
    Multi(MultiVal<ID>),
}

impl NoVal {
    pub const fn new(id: u32) -> Self {
        NoVal(id)
    }
    pub const fn new_certain(id: u32) -> Self {
        NoVal(id)
    }
}
impl<ID> SingleVal<ID> {
    pub const fn new(id: ID) -> Self {
        SingleVal::Uncertain(id)
    }
    pub const fn new_certain(id: u32) -> Self {
        SingleVal::Certain(id)
    }
    pub const fn into(self) -> Valued<ID> {
        Valued::Single(self)
    }
}
impl<ID> MultiVal<ID> {
    pub const fn new(id: ID) -> Self {
        MultiVal::Uncertain(id)
    }
    pub const fn new_certain(id: u32) -> Self {
        MultiVal::Certain(id)
    }
    pub const fn into(self) -> Valued<ID> {
        Valued::Multi(self)
    }
}

impl<ID> Valued<ID> {
    pub fn id(self) -> Option<ID> {
        match self {
            Valued::Single(SingleVal::Uncertain(id)) => Some(id),
            Valued::Multi(MultiVal::Uncertain(id)) => Some(id),
            _ => None,
        }
    }
}
