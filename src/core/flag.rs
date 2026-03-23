use super::{CowOwned, CowSlice, CowStr, PossibleValues, comp_with_possible, parse_flag};
use crate::completion::{CompletionGroup, Unready};
use crate::error::Error;
use crate::parsed_flag::ParsedFlag;
use crate::{Completion, Result, Seen, id};
use std::fmt::Debug;
use std::iter::Peekable;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompleteWithEqual {
    /// `ls --colo<TAB>` => `--color=`
    Must,
    /// `ls --colo<TAB>` => `--color=` and `--color`
    /// NOTE: everything marked as `Optional` should be meaningful.
    /// There should be a functional difference between `--color=always` and `--color always`
    Optional,
    /// `ls --colo<TAB>` => `--color`
    /// NOTE: This doesn't mean it CAN'T have an equal (only pure boolean flags can't have equal).
    /// It just means we're not going to give an equal sign when hitting completion
    NoNeed,
}

pub mod flag_type {
    use super::*;

    #[doc(hidden)]
    #[derive(Clone, Copy, Debug)]
    pub struct Bool {
        pub(crate) seen_id: id::NoVal,
    }
    impl Bool {
        pub(crate) fn push(&self, seen: &mut Seen) {
            seen.push_no_val(self.seen_id)
        }
    }
    #[doc(hidden)]
    #[derive(Debug)]
    pub struct Valued<ID> {
        pub(crate) id: Option<ID>,
        pub(crate) seen_id: id::Valued,
        pub(crate) complete_with_equal: CompleteWithEqual,
        pub(crate) possible_values: PossibleValues,
    }
    impl<ID> Valued<ID> {
        pub(crate) fn push(&self, seen: &mut Seen, arg: String) {
            seen.push_valued(self.seen_id, arg)
        }
    }

    #[derive(Debug)]
    pub enum Type<ID> {
        Bool(Bool),
        Valued(Valued<ID>),
    }
    impl<ID> Type<ID> {
        pub const fn new_bool(seen_id: id::NoVal) -> Self {
            Type::Bool(Bool { seen_id })
        }
        pub const fn new_valued(
            id: Option<ID>,
            seen_id: id::Valued,
            complete_with_equal: CompleteWithEqual,
            possible_values: PossibleValues,
        ) -> Self {
            Type::Valued(Valued {
                id,
                seen_id,
                complete_with_equal,
                possible_values,
            })
        }
    }
}

use flag_type::*;

type Longs = CowOwned<&'static str, String>;
impl Longs {
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        let (v1, v2): (&[&str], &[String]) = match self {
            CowOwned::Borrow(v) => (*v, &[]),
            CowOwned::Owned(v) => (&[], v.as_slice()),
        };
        v1.iter().map(|s| *s).chain(v2.iter().map(|s| s.as_str()))
    }
}

/// The object to represent a CLI flag.
///
/// NOTE: this includes boolean flags (e.g. `ls -l`) **AND** valued flags (e.g. `ls --color`).
/// The later is often called "option", but that creates confusion with Rust's `Option` type.
///
/// For the difference between boolean and valued flags, check the field [`Flag::ty`].
#[derive(Debug)]
pub struct Flag<ID> {
    pub ty: Type<ID>,
    pub short: CowSlice<char>,
    pub long: Longs,
    pub description: CowStr,
    pub once: bool,
}

impl<ID: PartialEq + Copy + Debug> Flag<ID> {
    pub fn get_description(&self) -> &str {
        if !self.description.is_empty() {
            return &self.description;
        }
        if let Some(long) = self.long.iter().next() {
            return long;
        }
        ""
    }
    pub(super) fn gen_completion(&self, is_long: Option<bool>) -> impl Iterator<Item = Completion> {
        let long_iter = self.long.iter();
        let (long, short) = match is_long {
            None => (Some(long_iter), &*self.short),
            Some(true) => (Some(long_iter), &[] as &[char]),
            Some(false) => (None, &*self.short),
        };

        let long = long
            .into_iter()
            .flatten()
            .map(|l| Completion::new(format!("--{l}"), self.get_description()));
        let short = short
            .iter()
            .map(|s| Completion::new(format!("-{s}"), self.get_description()));
        let iter = long.chain(short).take(1);
        iter.flat_map(|mut comp| {
            let mut more = None;
            if let Type::Valued(Valued {
                complete_with_equal,
                ..
            }) = self.ty
            {
                match complete_with_equal {
                    CompleteWithEqual::NoNeed => (),
                    CompleteWithEqual::Must => {
                        comp.value += "=";
                    }
                    CompleteWithEqual::Optional => {
                        more = Some(comp.clone());
                        comp.value += "=";
                    }
                }
            }
            std::iter::once(comp).chain(more)
        })
    }

    pub(super) fn exists_in_seen(&self, seen: &Seen) -> bool {
        match self.ty {
            Type::Bool(Bool { seen_id, .. }) => seen.find(seen_id).is_some(),
            Type::Valued(Valued {
                seen_id: id::Valued::Single(id),
                ..
            }) => seen.find(id).is_some(),
            Type::Valued(Valued {
                seen_id: id::Valued::Multi(id),
                ..
            }) => seen.find(id).is_some(),
        }
    }

    pub(super) fn supplement(
        &self,
        seen: &mut Seen,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Result<Option<CompletionGroup<ID>>> {
        let valued = match &self.ty {
            Type::Bool(inner) => {
                inner.push(seen);
                return Ok(None);
            }
            Type::Valued(inner) => inner,
        };
        let name = self.name();

        match valued.complete_with_equal {
            CompleteWithEqual::Must => return Err(Error::RequiresEqual(name.to_owned())),
            CompleteWithEqual::NoNeed => (),
            CompleteWithEqual::Optional => {
                // TODO: Maybe one day clap will tell us.
                log::info!(
                    "Optional flag {} doesn't have value. Push an empty string to seen because we don't know its default value (clap wouldn't tell us).",
                    name
                );
                valued.push(seen, String::new());
                return Ok(None);
            }
        }

        let arg = args.next().unwrap();
        match parse_flag(&arg, false) {
            ParsedFlag::NotFlag | ParsedFlag::Empty | ParsedFlag::SingleDash => (),
            ParsedFlag::DoubleDash | ParsedFlag::Long { .. } | ParsedFlag::Shorts => {
                log::warn!(
                    "`--{name} {arg}` is invalid. Maybe you should write it like `--{name}={arg}",
                );
                return Err(Error::FlagNoValue(name.to_owned()));
            }
        }

        if args.peek().is_none() {
            let unready = Unready::new(String::new(), arg.clone());
            let group = comp_with_possible(unready, &valued.possible_values, arg, valued.id);
            return Ok(Some(group));
        }

        valued.push(seen, arg);
        Ok(None)
    }

    pub(crate) fn name(&self) -> &str {
        self.long.iter().next().unwrap_or_default()
    }
}
