use super::{Either, comp_from_possible, parse_flag};
use crate::completion::{CompletionGroup, Unready};
use crate::error::Error;
use crate::parsed_flag::ParsedFlag;
use crate::{Completion, History, Result, id};
use std::fmt::Debug;
use std::iter::Peekable;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompleteWithEqual {
    /// `ls --colo<TAB>` => `--color=`
    Must,
    /// `ls --colo<TAB>` => `--color=` and `--color`
    /// NOTE: everything marked as `Optional` should be meaningful.
    /// There should be a functional difference between `ls --color=always` and `ls --color always`.
    /// So `ls --width` doesn't count because `ls --width=10` is the same as `ls --width 10`.
    Optional,
    /// `ls --colo<TAB>` => `--color`
    /// NOTE: This doesn't mean it CAN'T have an equal (only pure boolean flags can't have equal).
    /// It just means we're not going to give an equal sign when hitting completion.
    NoNeed,
}

pub mod flag_type {
    use super::*;

    #[doc(hidden)]
    #[derive(Clone, Copy)]
    pub struct Bool {
        pub(crate) id: id::NoVal,
    }
    impl Bool {
        pub(crate) fn push<ID: PartialEq + Debug>(&self, history: &mut History<ID>) {
            history.push_no_val(self.id)
        }
    }
    #[doc(hidden)]
    #[derive(Clone, Copy)]
    pub struct Valued<ID> {
        pub(crate) id: id::Valued<ID>,
        pub(crate) complete_with_equal: CompleteWithEqual,
    }
    impl<ID: PartialEq + Copy + Debug> Valued<ID> {
        pub(crate) fn push(&self, history: &mut History<ID>, arg: String) {
            history.push_valued(self.id, arg)
        }
    }

    #[derive(Clone, Copy)]
    pub enum Type<ID> {
        Bool(Bool),
        Valued(Valued<ID>),
    }
    impl<ID> Type<ID> {
        pub const fn new_bool(id: id::NoVal) -> Self {
            Type::Bool(Bool { id })
        }
        pub const fn new_valued(
            id: id::Valued<ID>,
            complete_with_equal: CompleteWithEqual,
        ) -> Self {
            Type::Valued(Valued {
                id,
                complete_with_equal,
            })
        }
    }
}

use flag_type::*;

pub struct Flag<ID> {
    pub ty: Type<ID>,
    pub short: &'static [char],
    pub long: &'static [&'static str],
    pub description: &'static str,
    pub once: bool,
}

impl<ID: PartialEq + Copy + Debug> Flag<ID> {
    pub fn get_description(&self) -> &'static str {
        if !self.description.is_empty() {
            return self.description;
        }
        if let Some(long) = self.long.get(0) {
            return *long;
        }
        ""
    }
    pub(super) fn gen_completion(&self, is_long: Option<bool>) -> impl Iterator<Item = Completion> {
        let (long, short) = match is_long {
            None => (self.long, self.short),
            Some(true) => (self.long, &[] as &[char]),
            Some(false) => (&[] as &[&str], self.short),
        };

        let long = long
            .iter()
            .map(|l| Completion::new(&format!("--{l}"), self.get_description()));
        let short = short
            .iter()
            .map(|s| Completion::new(&format!("-{s}"), self.get_description()));
        let iter = long.chain(short).take(1);
        iter.flat_map(|mut comp| {
            let mut more = None;
            match self.ty {
                Type::Valued(Valued {
                    complete_with_equal,
                    ..
                }) => match complete_with_equal {
                    CompleteWithEqual::NoNeed => (),
                    CompleteWithEqual::Must => {
                        comp.value += "=";
                    }
                    CompleteWithEqual::Optional => {
                        more = Some(comp.clone());
                        comp.value += "=";
                    }
                },
                _ => (),
            }
            std::iter::once(comp).chain(more.into_iter())
        })
    }

    pub(super) fn exists_in_history(&self, history: &History<ID>) -> bool {
        match self.ty {
            Type::Bool(Bool { id, .. }) => history.find(id).is_some(),
            Type::Valued(Valued {
                id: id::Valued::Single(id),
                ..
            }) => history.find(id).is_some(),
            Type::Valued(Valued {
                id: id::Valued::Multi(id),
                ..
            }) => history.find(id).is_some(),
        }
    }

    pub(super) fn supplement(
        &self,
        history: &mut History<ID>,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Result<Option<CompletionGroup<ID>>> {
        let valued = match self.ty {
            Type::Bool(inner) => {
                inner.push(history);
                return Ok(None);
            }
            Type::Valued(inner) => inner,
        };
        let name = self.name();

        match valued.complete_with_equal {
            CompleteWithEqual::Must => return Err(Error::RequiresEqual(name)),
            CompleteWithEqual::NoNeed => (),
            CompleteWithEqual::Optional => {
                // TODO: Maybe one day clap will tell us.
                log::info!(
                    "Optional flag {} doesn't have value. Push an empty string to history because we don't know its default value (clap wouldn't tell us).",
                    name
                );
                valued.push(history, String::new());
                return Ok(None);
            }
        }

        let value = args.next().unwrap();
        match parse_flag(&value, false) {
            ParsedFlag::NotFlag | ParsedFlag::Empty | ParsedFlag::SingleDash => (),
            ParsedFlag::DoubleDash | ParsedFlag::Long { .. } | ParsedFlag::Shorts => {
                log::warn!(
                    "`--{name} {value}` is invalid. Maybe you should write it like `--{name}={value}",
                );
                return Err(Error::FlagNoValue(name));
            }
        }

        if args.peek().is_none() {
            let unready = Unready::new(String::new(), value.clone());
            let group = match valued.id.id() {
                Either::A(id) => CompletionGroup::Unready { unready, value, id },
                Either::B(values) => comp_from_possible(unready, values),
            };
            return Ok(Some(group));
        }

        valued.push(history, value);
        Ok(None)
    }

    pub(crate) fn name(&self) -> &'static str {
        self.long.first().map(|s| *s).unwrap_or_default()
    }
}
