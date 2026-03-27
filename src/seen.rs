//! The module for values seen in the CLI.
//!
//! Define a collection of seen values [`Seen`], and the simplest unit [`SeenUnit`].

use crate::id;

#[derive(Debug, Eq, PartialEq)]
pub struct SeenUnitNoVal {
    pub id: id::NoVal,
    pub count: u32,
}
#[derive(Debug, Eq, PartialEq)]
pub struct SeenUnitSingleVal {
    pub id: id::SingleVal,
    pub value: String,
}
#[derive(Debug, Eq, PartialEq)]
pub struct SeenUnitMultiVal {
    pub id: id::MultiVal,
    pub values: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SeenUnit {
    No(SeenUnitNoVal),
    Single(SeenUnitSingleVal),
    Multi(SeenUnitMultiVal),
}

pub trait Getter {
    type Ret;
    fn match_and_cast<'a>(&self, h: &'a SeenUnit) -> Option<&'a Self::Ret>;
}
impl Getter for id::NoVal {
    type Ret = SeenUnitNoVal;
    fn match_and_cast<'a>(&self, h: &'a SeenUnit) -> Option<&'a Self::Ret> {
        match h {
            SeenUnit::No(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}
impl Getter for id::SingleVal {
    type Ret = SeenUnitSingleVal;
    fn match_and_cast<'a>(&self, h: &'a SeenUnit) -> Option<&'a Self::Ret> {
        match h {
            SeenUnit::Single(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}
impl Getter for id::MultiVal {
    type Ret = SeenUnitMultiVal;
    fn match_and_cast<'a>(&self, h: &'a SeenUnit) -> Option<&'a Self::Ret> {
        match h {
            SeenUnit::Multi(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}

/// A structure that records all seen CLI objects, along with their value if they have some.
///
/// In the derive workflow, the command structure is encoded in the *"Accessor"* of generated ID.
/// You can call the functions of these accessor to get strongly typed values.
/// ```no_run
/// use std::path::Path;
/// use supplement::Seen;
/// use supplement::helper::id_no_assoc as id;
/// # #[derive(Clone, Copy)]
/// # pub enum ID {
/// #     X3Xsub(RootAccessor, SubID),
/// # }
/// # #[derive(Clone, Copy)]
/// # pub enum SubID {
/// #     X8XCheckoutfiles(CheckoutAccessor, ()),
/// # }
///
/// # #[derive(Clone, Copy)]
/// # pub struct RootAccessor;
/// # #[derive(Clone, Copy)]
/// # pub struct CheckoutAccessor;
/// # impl RootAccessor {
/// #     pub fn git_dir(&self, seen: &Seen) -> Option<&Path> {
/// #         unimplemented!()
/// #     }
/// # }
/// # impl CheckoutAccessor {
/// #     pub fn file_or_commit(&self, seen: &Seen) -> Option<&str> {
/// #         unimplemented!()
/// #     }
/// #     pub fn files(&self, seen: &Seen) -> impl Iterator<Item = &Path> {
/// #         [].into_iter()
/// #     }
/// # }
/// fn handle_comp(id: ID, seen: &Seen) {
///     match id {
///         id!(ID.sub(root_acc) SubID.Checkout.files(chk_acc)) => {
///             // use `root_acc` to get the root args/flags
///             let _git_dir: Option<&Path> = root_acc.git_dir(seen);
///
///             // use `chk_acc` to get the args/flags for `checkout` sub-command
///             let _file_or_commit: Option<&str> = chk_acc.file_or_commit(seen);
///             let _files: Vec<&Path> = chk_acc.files(seen).collect();
///
///             // NOTE: the type system guarantees that you can **NEVER** get anything from other subcommands!
///         }
///     }
/// }
/// ```
///
///
/// Alternatively, you can search in the seen arguments by IDs using [`Seen::find`].
#[derive(Debug, Eq, PartialEq, Default)]
pub struct Seen(Vec<SeenUnit>);

impl Seen {
    pub fn new() -> Self {
        Default::default()
    }

    pub(crate) fn push_no_val(&mut self, id: id::NoVal) {
        log::debug!("push no value {:?}", id);
        for h in self.0.iter_mut() {
            match h {
                SeenUnit::No(h) if h.id == id => {
                    h.count += 1;
                    return;
                }
                _ => (),
            }
        }

        self.0.push(SeenUnit::No(SeenUnitNoVal { id, count: 1 }));
    }
    pub(crate) fn push_single_val(&mut self, id: id::SingleVal, value: String) {
        log::debug!("push single val {:?} {}", id, value);
        for h in self.0.iter_mut() {
            match h {
                SeenUnit::Single(h) if h.id == id => {
                    log::info!(
                        "push single val {:?}: {} where old value exists: {}",
                        id,
                        value,
                        h.value
                    );
                    h.value = value;
                    return;
                }
                _ => (),
            }
        }

        self.0
            .push(SeenUnit::Single(SeenUnitSingleVal { id, value }));
    }
    pub(crate) fn push_multi_val(&mut self, id: id::MultiVal, value: String) {
        log::debug!("push multi val {:?} {}", id, value);
        for h in self.0.iter_mut() {
            match h {
                SeenUnit::Multi(h) if h.id == id => {
                    h.values.push(value);
                    return;
                }
                _ => (),
            }
        }

        let values = vec![value];
        self.0
            .push(SeenUnit::Multi(SeenUnitMultiVal { id, values }));
    }

    pub(crate) fn push_valued(&mut self, id: id::Valued, value: String) {
        match id {
            id::Valued::Single(id) => self.push_single_val(id, value),
            id::Valued::Multi(id) => self.push_multi_val(id, value),
        }
    }

    /// Find the seen values by their ID.
    ///
    /// In the derive & code-gen workflow, you probably don't want to call this function directly,
    /// but instead would prefer the generated accessor functions.
    /// (Refer to [`Seen`] for more information)
    ///
    /// ----
    ///
    /// Based on the type of ID, the returned `SeenUnit` will contain different types of value.
    /// - [`id::NoVal`]: An integer that represents how many times it's seen in the CLI command
    /// - [`id::SingleVal`]: A single string
    /// - [`id::MultiVal`]: A vector of string
    ///
    /// ```no_run
    /// use supplement::{Seen, id};
    /// let seen = Seen::new();
    ///
    /// let id = id::NoVal::new(0);
    /// let c: u32 = seen.find(id).unwrap().count;
    ///
    /// let id = id::SingleVal::new(0);
    /// let v: &String = &seen.find(id).unwrap().value;
    ///
    /// let id = id::MultiVal::new(0);
    /// let v: &[String] = &seen.find(id).unwrap().values;
    /// ```
    pub fn find<I: Getter>(&self, id: I) -> Option<&I::Ret> {
        for h in self.0.iter() {
            let h = id.match_and_cast(h);
            if h.is_some() {
                return h;
            }
        }
        None
    }

    #[doc(hidden)]
    pub fn into_inner(self) -> Vec<SeenUnit> {
        self.0
    }
}
