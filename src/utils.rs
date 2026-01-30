use std::iter::{Peekable, once};

use crate::parsed_flag::ParsedFlag;
use crate::{CompResult, History, SupplementID};

pub struct FlagInfo {
    pub short: Option<char>,
    pub long: &'static str,
    pub description: &'static str,
}
pub struct CommandInfo {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct Flag {
    pub id: SupplementID,
    pub info: FlagInfo,
    pub comp_options: Option<fn(&History, &str) -> Vec<CompResult>>,
    pub once: bool,
}
pub struct Arg {
    pub id: SupplementID,
    pub comp_options: fn(&History, &str) -> Vec<CompResult>,
    // TODO: infinite args?
}
pub struct Command {
    pub id: SupplementID,
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
    ) -> Option<Vec<CompResult>> {
        let value = args.next().unwrap();
        if args.peek().is_none() {
            return Some((self.comp_options)(history, &value));
        }

        history.push_arg(self.id, value);
        None
    }
}

impl Flag {
    fn supplement(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Option<Vec<CompResult>> {
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
    ) -> Vec<CompResult> {
        let mut history = History::default();
        self.supplement_with_history(&mut history, args, last_is_empty)
    }

    pub fn supplement_with_history(
        &self,
        history: &mut History,
        args: impl Iterator<Item = String>,
        last_is_empty: bool,
    ) -> Vec<CompResult> {
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

    fn supplement_recur(
        &self,
        is_first: bool,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Vec<CompResult> {
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
                        unimplemented!("error: value for a boolean flag");
                    }
                    $history.push_flag($flag.id, equal.to_string());
                } else {
                    let res = $flag.supplement($history, args);
                    if let Some(res) = res {
                        return res;
                    }
                }
            };
        }

        match ParsedFlag::new(&arg) {
            ParsedFlag::SingleDash | ParsedFlag::DoubleDash => {
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
                let flag = self.flags(history).find(|f| f.info.long == body);
                let Some(flag) = flag else { unimplemented!() };
                handle_flag!(flag, equal, history);
            }
            ParsedFlag::Short { body, equal } => {
                let flag = self.flags(history).find(|f| f.info.short == Some(body));
                let Some(flag) = flag else { unimplemented!() };
                handle_flag!(flag, equal, history);
            }
            ParsedFlag::MultiShort { body, equal } => {
                let mut body = body.chars().peekable();
                loop {
                    let Some(ch) = body.next() else {
                        break;
                    };
                    let is_last = body.peek().is_none();
                    let flag = self.flags(history).find(|f| f.info.short == Some(ch));
                    let Some(flag) = flag else { unimplemented!() };

                    if is_last {
                        handle_flag!(flag, equal, history);
                    } else {
                        if flag.comp_options.is_some() {
                            if equal.is_some() {
                                // e.g. git commit -ma=abcd
                                unimplemented!();
                            }
                            // e.g. git commit -mabcde
                            history.push_flag(flag.id, body.collect());
                            break;
                        }

                        history.push_pure_flag(flag.id);
                    }
                }
            }
            ParsedFlag::Error(_) | ParsedFlag::Empty => {
                unimplemented!()
            }
        }

        self.supplement_recur(false, history, args)
    }

    fn supplement_last(&self, history: &mut History, arg: String) -> Vec<CompResult> {
        let all_long_flags = self.flags(history).map(|f| CompResult {
            value: format!("--{}", f.info.long),
            description: f.info.description.to_string(),
        });

        match ParsedFlag::new(&arg) {
            ParsedFlag::Empty => {
                let cmd_iter = self.commands.iter().map(|c| CompResult {
                    value: c.info.name.to_string(),
                    description: c.info.description.to_string(),
                });
                let arg_comp = if let Some(arg_obj) = self.args.first() {
                    (arg_obj.comp_options)(history, &arg)
                } else {
                    vec![]
                };
                return cmd_iter.chain(arg_comp.into_iter()).collect();
            }
            ParsedFlag::DoubleDash => {
                return all_long_flags.collect();
            }
            ParsedFlag::SingleDash => {
                let iter = self.flags(history).filter_map(|f| {
                    if let Some(c) = f.info.short {
                        Some(CompResult {
                            value: format!("-{}", c),
                            description: f.info.description.to_string(),
                        })
                    } else {
                        None
                    }
                });
                let iter = iter.chain(all_long_flags);
                return iter.collect();
            }
            _ => unimplemented!(),
        }
    }
    fn supplement_args(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
        arg: String,
    ) -> Vec<CompResult> {
        let mut args = args.chain(once(arg)).peekable();
        for arg_obj in self.args.iter() {
            let res = arg_obj.supplement(history, &mut args);
            if let Some(res) = res {
                return res;
            }
        }

        panic!("too many args");
    }
}
