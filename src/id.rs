pub(crate) type PossibleValues = &'static [(&'static str, &'static str)];

#[derive(Debug, Eq, Clone, Copy)]
#[doc(hidden)]
pub struct AlwaysEq(PossibleValues);

impl PartialEq for AlwaysEq {
    fn eq(&self, _: &Self) -> bool {
        true // Avoid useless computation. Comparing ID is sufficient.
    }
}

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
    Certain(u32, AlwaysEq),
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
    Certain(u32, AlwaysEq),
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
}
impl<ID> SingleVal<ID> {
    pub const fn new(id: ID) -> Self {
        SingleVal::Uncertain(id)
    }
    pub const fn new_certain(id: u32, values: PossibleValues) -> Self {
        SingleVal::Certain(id, AlwaysEq(values))
    }
    pub const fn into(self) -> Valued<ID> {
        Valued::Single(self)
    }
}
impl<ID> MultiVal<ID> {
    pub const fn new(id: ID) -> Self {
        MultiVal::Uncertain(id)
    }
    pub const fn new_certain(id: u32, values: PossibleValues) -> Self {
        MultiVal::Certain(id, AlwaysEq(values))
    }
    pub const fn into(self) -> Valued<ID> {
        Valued::Multi(self)
    }
}

pub(crate) enum Either<A, B> {
    A(A),
    B(B),
}

impl<ID> Valued<ID> {
    pub(crate) fn id(self) -> Either<ID, PossibleValues> {
        match self {
            Valued::Single(SingleVal::Uncertain(id)) => Either::A(id),
            Valued::Multi(MultiVal::Uncertain(id)) => Either::A(id),
            Valued::Single(SingleVal::Certain(_, v)) => Either::B(v.0),
            Valued::Multi(MultiVal::Certain(_, v)) => Either::B(v.0),
        }
    }
}
