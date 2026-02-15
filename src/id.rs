/// Id for things that cannot have value.
/// All subcommand belong to this, and flags can also have no value.
///
/// When searching for it in `History`, it will still have an integer value `count`,
/// which represents how many times it's seen in the CLI command
/// ```no_run
/// use supplements::{History, id};
/// let history = History::default();
/// let id: id::NoVal = id::NoVal::new(0, "");
/// let c: u32 = history.find(id).unwrap().count; // Represents how many times it's seen in the CLI command
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NoVal(u32, pub(crate) &'static str);

/// Id for things that have at most one value.
/// When searching for it in `History`, it will have a single string `value`
/// ```no_run
/// use supplements::{History, id};
/// let history = History::default();
/// let id: id::SingleVal = id::SingleVal::new(0, "");
/// let v: &str = &history.find(id).unwrap().value;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SingleVal(u32, pub(crate) &'static str);

/// Id for things that can have more than one value.
/// When searching for it in `History`, it will have a vector of string `values`
/// ```no_run
/// use supplements::{History, id};
/// let history = History::default();
/// let id: id::MultiVal = id::MultiVal::new(0, "");
/// let v: &[String] = &history.find(id).unwrap().values;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MultiVal(u32, pub(crate) &'static str);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Valued {
    Single(SingleVal),
    Multi(MultiVal),
}

impl NoVal {
    pub const fn new(id: u32, ident: &'static str) -> Self {
        NoVal(id, ident)
    }
}
impl SingleVal {
    pub const fn new(id: u32, ident: &'static str) -> Self {
        SingleVal(id, ident)
    }
    pub const fn into(self) -> Valued {
        Valued::Single(self)
    }
}
impl MultiVal {
    pub const fn new(id: u32, ident: &'static str) -> Self {
        MultiVal(id, ident)
    }
    pub const fn into(self) -> Valued {
        Valued::Multi(self)
    }
}
