//! Module for IDs of CLI objects that can be used to lookup history in [`History`].
//!
//! There are two aspect for an ID: the amount of value it can take, and whether it is certain.
//! Each is represented by a different type or enum variant.
//!
//! - The amount of value affects the data type it gets from [`History::find`].
//! - An ID is *certain* if and only if it never needs custom completion logic.
//!   It will always return [`CompletionGroup::Ready`] during completion.
//!
//! |                  | no value  | single value             | multi value             |
//! |------------------|-----------|--------------------------|-------------------------|
//! | **Is certain**   | [`NoVal`] | [`SingleVal::Certain`]   | [`MultiVal::Certain`]   |
//! | **Not certain**  | N/A       | [`SingleVal::Uncertain`] | [`MultiVal::Uncertain`] |
//!
//! An ID that takes no value never needs a completion logic, hence is always certain.

#[cfg(doc)]
use crate::CompletionGroup;
#[cfg(doc)]
use crate::History;

/// Id for things that cannot have value.
///
/// When searching for it in [`History`], it will still have an integer value `count`,
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
/// When searching for it in [`History`], it will have a single string `value`
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
/// When searching for it in [`History`], it will have a vector of string `values`
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

#[doc(hidden)]
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
