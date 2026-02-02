use std::iter::{Peekable, once};

use crate::error::Error;
use crate::id;
use crate::info::*;
use crate::parsed_flag::ParsedFlag;
use crate::{Completion, History, Result};

type CompOption = fn(&History, &str) -> Vec<Completion>;

pub struct Flag {
    pub id: id::Flag,
    pub info: FlagInfo,
    pub comp_options: Option<CompOption>,
    pub once: bool,
}
pub struct Arg {
    pub id: id::Arg,
    pub comp_options: CompOption,
    // TODO: infinite args?
}
pub struct Command {
    pub id: id::Command,
    pub info: CommandInfo,
    pub all_flags: Vec<Flag>,
    pub args: Vec<Arg>,
    pub commands: Vec<Command>,
}

impl Arg {
    fn supplement(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Option<Vec<Completion>> {
        let value = args.next().unwrap();
        // TODO: use `ParsedFlag` to check if `value` is valid
        if args.peek().is_none() {
            return Some((self.comp_options)(history, &value));
        }

        history.push_arg(self.id, value);
        None
    }
}

impl Flag {
    fn gen_completion(&self, is_long: Option<bool>) -> impl Iterator<Item = Completion> {
        let (long, short) = match is_long {
            None => (self.info.long, self.info.short),
            Some(true) => (self.info.long, &[] as &[char]),
            Some(false) => (&[] as &[&str], self.info.short),
        };
        long.iter()
            .map(|l| Completion::new(&format!("--{l}"), self.info.description))
            .chain(
                short
                    .iter()
                    .map(|s| Completion::new(&format!("-{s}"), self.info.description)),
            )
    }
    fn supplement(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Option<Vec<Completion>> {
        let Some(comp_options) = self.comp_options else {
            history.push_pure_flag(self.id);
            return None;
        };

        let arg = args.next().unwrap();
        if args.peek().is_none() {
            return Some(comp_options(&history, &arg));
        }

        history.push_flag(self.id, arg);
        None
    }
}

impl Command {
    pub fn supplement(
        &self,
        args: impl Iterator<Item = String>,
        last_is_empty: bool,
    ) -> Result<Vec<Completion>> {
        let mut history = History::default();
        self.supplement_with_history(&mut history, args, last_is_empty)
    }

    pub fn supplement_with_history(
        &self,
        history: &mut History,
        mut args: impl Iterator<Item = String>,
        last_is_empty: bool,
    ) -> Result<Vec<Completion>> {
        args.next(); // ignore the first arg which is the program's name

        let last_arg = if last_is_empty {
            Some(String::new())
        } else {
            None
        };

        let mut args = args.chain(last_arg.into_iter()).peekable();
        if args.peek().is_none() {
            panic!();
        }

        self.supplement_recur(true, history, &mut args)
    }

    fn flags(&self, history: &History) -> impl Iterator<Item = &Flag> {
        self.all_flags.iter().filter(|f| {
            if !f.once {
                true
            } else {
                let exists = history.find(f.id).is_some();
                if exists {
                    log::debug!("flag {:?} already exists", f.id);
                }
                !exists
            }
        })
    }

    fn find_flag<F: FnMut(&Flag) -> bool>(
        &self,
        arg: &str,
        history: &History,
        mut filter: F,
    ) -> Result<&Flag> {
        match self.flags(history).find(|f| filter(f)) {
            Some(flag) => Ok(flag),
            None => Err(Error::FlagNotFound(arg.to_owned())),
        }
    }

    fn find_long_flag(&self, flag: &str, history: &History) -> Result<&Flag> {
        self.find_flag(flag, history, |f| f.info.long.iter().any(|l| *l == flag))
    }
    fn find_short_flag(&self, flag: char, history: &History) -> Result<&Flag> {
        self.find_flag(&flag.to_string(), history, |f| {
            f.info.short.iter().any(|s| *s == flag)
        })
    }

    fn supplement_recur(
        &self,
        is_first: bool,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Result<Vec<Completion>> {
        let arg = args.next().unwrap();
        if is_first {
            history.push_command(self.id);
        }

        if args.peek().is_none() {
            return self.supplement_last(history, arg);
        }

        macro_rules! handle_flag {
            ($flag:expr, $equal:expr, $history:expr) => {
                if let Some(equal) = $equal {
                    if $flag.comp_options.is_none() {
                        return Err(Error::BoolFlagEqualsValue(arg));
                    }
                    $history.push_flag($flag.id, equal.to_string());
                } else {
                    let res = $flag.supplement($history, args);
                    if let Some(res) = res {
                        return Ok(res);
                    }
                }
            };
        }

        match ParsedFlag::new(&arg)? {
            ParsedFlag::SingleDash | ParsedFlag::DoubleDash | ParsedFlag::Empty => {
                return self.supplement_args(history, args, arg);
            }
            ParsedFlag::NotFlag => {
                let command = self.commands.iter().find(|c| arg == c.info.name);
                return match command {
                    Some(command) => command.supplement_recur(true, history, args),
                    None => self.supplement_args(history, args, arg),
                };
            }
            ParsedFlag::Long { body, equal } => {
                let flag = self.find_long_flag(body, history)?;
                handle_flag!(flag, equal, history);
            }
            ParsedFlag::Shorts => {
                let resolved = self.resolve_shorts(history, &arg)?;
                handle_flag!(resolved.last_flag, resolved.value, history);
            }
        }

        self.supplement_recur(false, history, args)
    }

    fn supplement_last(&self, history: &mut History, arg: String) -> Result<Vec<Completion>> {
        let mut raise_empty_err = true;
        let ret: Vec<_> = match ParsedFlag::new(&arg)? {
            ParsedFlag::Empty | ParsedFlag::NotFlag => {
                let cmd_iter = self.commands.iter().map(|c| Completion {
                    value: c.info.name.to_string(),
                    description: c.info.description.to_string(),
                });
                let arg_comp = if let Some(arg_obj) = self.args.first() {
                    (arg_obj.comp_options)(history, &arg)
                } else {
                    vec![]
                };
                cmd_iter.chain(arg_comp.into_iter()).collect()
            }
            ParsedFlag::DoubleDash | ParsedFlag::Long { equal: None, .. } => self
                .flags(history)
                .map(|f| f.gen_completion(Some(true)))
                .flatten()
                .collect(),
            ParsedFlag::SingleDash => self
                .flags(history)
                .map(|f| f.gen_completion(None))
                .flatten()
                .collect(),
            ParsedFlag::Long {
                equal: Some(value),
                body,
            } => {
                raise_empty_err = false;
                let flag = self.find_long_flag(body, history)?;
                let Some(comp_options) = flag.comp_options else {
                    return Err(Error::BoolFlagEqualsValue(arg));
                };
                comp_options(history, value)
                    .into_iter()
                    .map(|c| Completion::new(&format!("--{}={}", body, c.value), &c.description))
                    .collect()
            }
            ParsedFlag::Shorts => {
                let resolved = self.resolve_shorts(history, &arg)?;
                if let Some(comp_options) = resolved.last_flag.comp_options {
                    let value = resolved.value.unwrap_or("");
                    comp_options(history, value)
                        .into_iter()
                        .map(|c| {
                            Completion::new(
                                &format!("{}{}", resolved.flag_part, c.value),
                                &c.description,
                            )
                        })
                        .collect()
                } else {
                    log::debug!("list short flags with history {:?}", history);
                    history.push_pure_flag(resolved.last_flag.id);
                    self.flags(history)
                        .map(|f| f.gen_completion(Some(false)))
                        .flatten()
                        .map(|c| {
                            let flag = &c.value[1..]; // skip the first '-' character
                            Completion::new(
                                &format!("{}{}", resolved.flag_part, &flag),
                                &c.description,
                            )
                        })
                        .collect()
                }
            }
        };
        if ret.is_empty() && raise_empty_err {
            return Err(Error::NoPossibleCompletion);
        }
        Ok(ret)
    }
    fn supplement_args(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
        arg: String,
    ) -> Result<Vec<Completion>> {
        let mut args = once(arg).chain(args).peekable();
        for arg_obj in self.args.iter() {
            let res = arg_obj.supplement(history, &mut args);
            if let Some(res) = res {
                return Ok(res);
            }
        }

        panic!("too many args");
    }

    fn resolve_shorts<'a, 'b>(
        &'b self,
        history: &mut History,
        shorts: &'a str,
    ) -> Result<ResolvedMultiShort<'a, 'b>> {
        let mut chars = shorts.chars().peekable();
        let mut len = 1; // ignore the first '-'
        chars.next(); // ignore the first '-'
        loop {
            len += 1;
            let ch = chars.next().unwrap();
            let flag = self.find_short_flag(ch, history)?;
            match chars.peek() {
                None => {
                    return Ok(ResolvedMultiShort {
                        flag_part: shorts,
                        last_flag: flag,
                        value: None,
                    });
                }
                Some('=') => {
                    if flag.comp_options.is_none() {
                        return Err(Error::BoolFlagEqualsValue(shorts.to_owned()));
                    };
                    len += 1;
                    return Ok(ResolvedMultiShort {
                        flag_part: &shorts[..len],
                        last_flag: flag,
                        value: Some(&shorts[len..]),
                    });
                }
                _ => {
                    if flag.comp_options.is_some() {
                        return Ok(ResolvedMultiShort {
                            flag_part: &shorts[..len],
                            last_flag: flag,
                            value: Some(&shorts[len..]),
                        });
                    }

                    history.push_pure_flag(flag.id);
                }
            }
        }
    }
}

struct ResolvedMultiShort<'a, 'b> {
    flag_part: &'a str,
    last_flag: &'b Flag,
    value: Option<&'a str>,
}
