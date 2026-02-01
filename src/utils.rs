use std::iter::{Peekable, once};

use crate::error::Error;
use crate::id;
use crate::parsed_flag::ParsedFlag;
use crate::{Completion, Either, History, Result};

pub struct FlagInfo {
    pub short: &'static [char],
    pub long: &'static [&'static str],
    pub description: &'static str,
}
pub struct CommandInfo {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct Flag {
    pub id: id::Flag,
    pub info: FlagInfo,
    pub comp_options: Option<fn(&History, &str) -> Vec<Completion>>,
    pub once: bool,
}
pub struct Arg {
    pub id: id::Arg,
    pub comp_options: fn(&History, &str) -> Vec<Completion>,
    // TODO: infinite args?
}
pub struct Command {
    pub id: id::Command,
    pub info: CommandInfo,
    pub true_flags: Vec<Flag>,
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
        let iter_long = self.info.long.iter().map(|t| Either::A(t));
        let iter_short = self.info.short.iter().map(|t| Either::B(t));
        let iter = iter_long.chain(iter_short);

        iter.filter_map(move |t| match t {
            Either::A(l) if is_long != Some(false) => {
                Some(Completion::new(&format!("--{l}"), self.info.description))
            }
            Either::B(s) if is_long != Some(true) => {
                Some(Completion::new(&format!("-{s}"), self.info.description))
            }
            _ => None,
        })
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
        args: impl Iterator<Item = String>,
        last_is_empty: bool,
    ) -> Result<Vec<Completion>> {
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
        self.true_flags.iter().filter(|f| {
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
                        return Err(Error::ValueForBoolFlag(arg));
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
            ParsedFlag::Short { body, equal } => {
                let flag = self.find_short_flag(body, history)?;
                handle_flag!(flag, equal, history);
            }
            ParsedFlag::MultiShort(body) => {
                let mut body = body.chars().peekable();
                loop {
                    let Some(ch) = body.next() else {
                        break;
                    };
                    let flag = self.find_short_flag(ch, history)?;
                    match body.peek() {
                        None => {
                            handle_flag!(flag, None::<&str>, history);
                        }
                        Some('=') => {
                            body.next();
                            let equal: String = body.collect();
                            handle_flag!(flag, Some(equal), history);
                            break;
                        }
                        _ => {
                            if flag.comp_options.is_some() {
                                history.push_flag(flag.id, body.collect());
                                break;
                            }
                            history.push_pure_flag(flag.id);
                        }
                    }
                }
            }
        }

        self.supplement_recur(false, history, args)
    }

    fn supplement_last(&self, history: &mut History, arg: String) -> Result<Vec<Completion>> {
        let ret = match ParsedFlag::new(&arg)? {
            ParsedFlag::Empty => {
                // TODO: error if both empty?
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
            ParsedFlag::DoubleDash => self
                .flags(history)
                .map(|f| f.gen_completion(Some(true)))
                .flatten()
                .collect(),
            ParsedFlag::SingleDash => self
                .flags(history)
                .map(|f| f.gen_completion(None))
                .flatten()
                .collect(),
            _ => unimplemented!(),
        };
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
}
