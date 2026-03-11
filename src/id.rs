//! Module for IDs of CLI objects that can be used to lookup values in [`Seen`].
//!
//! There are two aspect for an ID: the amount of value it can take, and whether it is customizable.
//! Each is represented by a different type or enum variant.
//!
//! - The amount of value affects the data type it gets from [`Seen::find`].
//! - An ID is *customizable* if and only if it needs custom completion logic.
//!   It will always return [`CompletionGroup::Unready`] during completion.
//!
//! |                  | no value  | single value          | multi value          |
//! |------------------|-----------|-----------------------|----------------------|
//! | **Customizable** | N/A       | [`SingleVal::Custom`] | [`MultiVal::Custom`] |
//! | **Static**       | [`NoVal`] | [`SingleVal::Static`] | [`MultiVal::Static`] |
//!
//! An ID that takes no value never needs a completion logic, hence is always static.

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
/// let seen = Seen::<()>::new();
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
/// let id = id::SingleVal::new(());
/// let v: &str = &seen.find(id).unwrap().value;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SingleVal<ID> {
    Custom(ID),
    Static(u32),
}

/// Id for things that can have more than one value.
/// When searching for it in [`Seen`], it will have a vector of string `values`
/// ```no_run
/// use supplement::{Seen, id};
/// let seen = Seen::new();
/// let id = id::MultiVal::new(());
/// let v: &[String] = &seen.find(id).unwrap().values;
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MultiVal<ID> {
    Custom(ID),
    Static(u32),
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
    pub const fn new_static(id: u32) -> Self {
        NoVal(id)
    }
}
impl<ID> SingleVal<ID> {
    pub const fn new(id: ID) -> Self {
        SingleVal::Custom(id)
    }
    pub const fn new_static(id: u32) -> Self {
        SingleVal::Static(id)
    }
    pub const fn into(self) -> Valued<ID> {
        Valued::Single(self)
    }
}
impl<ID> MultiVal<ID> {
    pub const fn new(id: ID) -> Self {
        MultiVal::Custom(id)
    }
    pub const fn new_static(id: u32) -> Self {
        MultiVal::Static(id)
    }
    pub const fn into(self) -> Valued<ID> {
        Valued::Multi(self)
    }
}

impl<ID> Valued<ID> {
    pub fn id(self) -> Option<ID> {
        match self {
            Valued::Single(SingleVal::Custom(id)) => Some(id),
            Valued::Multi(MultiVal::Custom(id)) => Some(id),
            _ => None,
        }
    }
}
