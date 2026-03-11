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
pub struct SeenUnitSingleVal<ID> {
    pub id: id::SingleVal<ID>,
    pub value: String,
}
#[derive(Debug, Eq, PartialEq)]
pub struct SeenUnitMultiVal<ID> {
    pub id: id::MultiVal<ID>,
    pub values: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SeenUnit<ID> {
    No(SeenUnitNoVal),
    Single(SeenUnitSingleVal<ID>),
    Multi(SeenUnitMultiVal<ID>),
}

pub trait Getter<ID> {
    type Ret;
    fn match_and_cast<'a>(&self, h: &'a SeenUnit<ID>) -> Option<&'a Self::Ret>;
}
impl<ID> Getter<ID> for id::NoVal {
    type Ret = SeenUnitNoVal;
    fn match_and_cast<'a>(&self, h: &'a SeenUnit<ID>) -> Option<&'a Self::Ret> {
        match h {
            SeenUnit::No(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}
impl<ID: PartialEq> Getter<ID> for id::SingleVal<ID> {
    type Ret = SeenUnitSingleVal<ID>;
    fn match_and_cast<'a>(&self, h: &'a SeenUnit<ID>) -> Option<&'a Self::Ret> {
        match h {
            SeenUnit::Single(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}
impl<ID: PartialEq> Getter<ID> for id::MultiVal<ID> {
    type Ret = SeenUnitMultiVal<ID>;
    fn match_and_cast<'a>(&self, h: &'a SeenUnit<ID>) -> Option<&'a Self::Ret> {
        match h {
            SeenUnit::Multi(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}

/// A structure that records all seen CLI objects, along with their value if they have some.
///
/// In the code-gen workflow, you can call function `ID::with_seen` to get a strongly typed context,
/// which encodes the command structure with its enum structure.
///
/// ```no_run
/// # mod def {
/// #     #[derive(Clone, Copy)]
/// #     pub enum ID<T = ()> {
/// #         CMDCheckout(T, checkout::ID<T>)
/// #     }
/// #     impl <T> ID<T> {
/// #         pub fn with_seen<U>(self, u: U) -> ID<U> {
/// #             unimplemented!()
/// #         }
/// #         pub fn val_git_dir(&self) -> Option<&str> {
/// #             unimplemented!()
/// #         }
/// #     }
/// #     pub mod checkout {
/// #         #[derive(Clone, Copy)]
/// #         pub enum ID<T> {
/// #             ValFiles(T)
/// #         }
/// #         impl<T> ID<T> {
/// #             pub fn val_files(&self) -> Option<&str> {
/// #                 unimplemented!()
/// #             }
/// #             pub fn val_file_or_commit(&self) -> &[String] {
/// #                 unimplemented!()
/// #             }
/// #         }
/// #     }
/// # }
/// # use def::ID;
/// use supplement::Seen;
/// use supplement::helper::id;
///
/// fn handle_comp(id: ID, seen: Seen<ID>) {
///     let id_with_ctx: ID<&Seen<ID>> = id.with_seen(&seen);
///     match id_with_ctx {
///         id!(def(root_id) checkout(chk_id) files) => {
///             // use `root_id` to get the root args/flags
///             let _git_dir: Option<&str> = root_id.val_git_dir();
///
///             // use `chk_id` to get the args/flags for `checkout` sub-command
///             let _files: Option<&str> = chk_id.val_files();
///             let _file_or_commit: &[String] = chk_id.val_file_or_commit();
///
///             // NOTE: the type system guarantees that you can **NEVER** get anything from other subcommands!
///         }
///     }
/// }
/// ```
///
///
/// Alternatively, you can search in the seen arguments by IDs using [`Seen::find`].
#[derive(Debug, Eq, PartialEq)]
pub struct Seen<ID>(Vec<SeenUnit<ID>>);

impl<ID> Default for Seen<ID> {
    fn default() -> Self {
        Seen(vec![])
    }
}

impl<ID: PartialEq + std::fmt::Debug> Seen<ID> {
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

        self.0
            .push(SeenUnit::No(SeenUnitNoVal { id, count: 1 }));
    }
    pub(crate) fn push_single_val(&mut self, id: id::SingleVal<ID>, value: String) {
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
    pub(crate) fn push_multi_val(&mut self, id: id::MultiVal<ID>, value: String) {
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

    pub(crate) fn push_valued(&mut self, id: id::Valued<ID>, value: String) {
        match id {
            id::Valued::Single(id) => self.push_single_val(id, value),
            id::Valued::Multi(id) => self.push_multi_val(id, value),
        }
    }

    /// Find the seen values by their ID.
    ///
    /// In the code-gen workflow, you probably don't want to call this function directly,
    /// but instead would prefer the generated `ID::with_seen` function.
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
    /// let seen = Seen::<()>::new();
    ///
    /// let id = id::NoVal::new(0);
    /// let c: u32 = seen.find(id).unwrap().count;
    ///
    /// let id = id::SingleVal::new(());
    /// let v: &String = &seen.find(id).unwrap().value;
    ///
    /// let id = id::MultiVal::new(());
    /// let v: &[String] = &seen.find(id).unwrap().values;
    /// ```
    pub fn find<I: Getter<ID>>(&self, id: I) -> Option<&I::Ret> {
        for h in self.0.iter() {
            let h = id.match_and_cast(h);
            if h.is_some() {
                return h;
            }
        }
        None
    }

    #[doc(hidden)]
    pub fn into_inner(self) -> Vec<SeenUnit<ID>> {
        self.0
    }
}
