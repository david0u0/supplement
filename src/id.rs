//! Module for IDs of CLI objects that can be used to lookup values in [`Seen`].
//!
//! ID also describes the amount of value it can take, encoded in their types.

#[cfg(doc)]
use crate::CompletionGroup;
#[cfg(doc)]
use crate::Seen;

/// Id for things that cannot have value.
///
/// When searching for it in [`Seen`], it will still have an integer value `count`,
/// which represents how many times it's seen in the CLI command
/// ```no_run
/// use supplement::{Seen, id};
/// let seen = Seen::new();
/// let id = id::NoVal::new(0);
/// let c: u32 = seen.find(id).unwrap().count; // Represents how many times it's seen in the CLI command
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NoVal(u32);

/// Id for things that have at most one value.
/// When searching for it in [`Seen`], it will have a single string `value`
/// ```no_run
/// use supplement::{Seen, id};
/// let seen = Seen::new();
/// let id = id::SingleVal::new(0);
/// let v: &str = &seen.find(id).unwrap().value;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SingleVal(u32);

/// Id for things that can have more than one value.
/// When searching for it in [`Seen`], it will have a vector of string `values`
/// ```no_run
/// use supplement::{Seen, id};
/// let seen = Seen::new();
/// let id = id::MultiVal::new(0);
/// let v: &[String] = &seen.find(id).unwrap().values;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MultiVal(u32);

#[doc(hidden)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Valued {
    Single(SingleVal),
    Multi(MultiVal),
}

impl NoVal {
    pub const fn new(id: u32) -> Self {
        NoVal(id)
    }
}
impl SingleVal {
    pub const fn new(id: u32) -> Self {
        SingleVal(id)
    }
    pub const fn into(self) -> Valued {
        Valued::Single(self)
    }
}
impl MultiVal {
    pub const fn new(id: u32) -> Self {
        MultiVal(id)
    }
    pub const fn into(self) -> Valued {
        Valued::Multi(self)
    }
}
