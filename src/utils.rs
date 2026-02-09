use std::iter::Peekable;

use crate::arg_context::ArgsContext;
use crate::completion::CompletionGroup;
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
    pub max_values: usize,
}
pub struct Command {
    pub id: id::Command,
    pub info: CommandInfo,
    pub all_flags: &'static [Flag],
    pub args: &'static [Arg],
    pub commands: &'static [Command],
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
    ) -> Result<Option<CompletionGroup>> {
        let Some(comp_options) = self.comp_options else {
            history.push_pure_flag(self.id);
            return Ok(None);
        };

        let arg = args.next().unwrap();
        match parse_flag(&arg, false)? {
            ParsedFlag::NotFlag | ParsedFlag::Empty | ParsedFlag::SingleDash => (),
            ParsedFlag::DoubleDash | ParsedFlag::Long { .. } | ParsedFlag::Shorts => {
                let name = self.id.name();
                log::warn!(
                    "`--{name} {arg}` is invalid. Maybe you should write it like `--{name}={arg}",
                );
                return Err(Error::FlagNoValue(name));
            }
        }

        if args.peek().is_none() {
            let group = CompletionGroup::new(comp_options(&history, &arg), arg);
            return Ok(Some(group));
        }

        history.push_flag(self.id, arg);
        Ok(None)
    }
}

fn supplement_arg(history: &mut History, ctx: &mut ArgsContext, arg: String) -> Result {
    let Some(arg_obj) = ctx.next_arg() else {
        return Err(Error::UnexpectedArg(arg));
    };
    history.push_arg(arg_obj.id, arg);
    Ok(())
}
fn parse_flag(s: &str, disable_flag: bool) -> Result<ParsedFlag<'_>> {
    if disable_flag {
        log::info!("flag is disabled: {}", s);
        Ok(ParsedFlag::NotFlag)
    } else {
        ParsedFlag::new(s).map_err(|e| e.into())
    }
}

impl Command {
    pub fn supplement(&self, args: impl Iterator<Item = String>) -> Result<CompletionGroup> {
        let mut history = History::default();
        self.supplement_with_history(&mut history, args)
    }

    pub fn supplement_with_history(
        &self,
        history: &mut History,
        mut args: impl Iterator<Item = String>,
    ) -> Result<CompletionGroup> {
        args.next(); // ignore the first arg which is the program's name

        let mut args = args.peekable();
        if args.peek().is_none() {
            return Err(Error::ArgsTooShort);
        }

        self.supplement_recur(&mut None, history, &mut args)
    }

    fn doing_external(&self, ctx: &ArgsContext) -> bool {
        let has_subcmd = !self.commands.is_empty();
        has_subcmd && ctx.has_seen_arg()
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
        args_ctx_opt: &mut Option<ArgsContext<'_>>,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Result<CompletionGroup> {
        let arg = args.next().unwrap();

        let args_ctx = if let Some(ctx) = args_ctx_opt {
            ctx
        } else {
            *args_ctx_opt = Some(ArgsContext::new(&self.args));
            args_ctx_opt.as_mut().unwrap()
        };

        if args.peek().is_none() {
            return self.supplement_last(args_ctx, history, arg);
        }

        macro_rules! handle_flag {
            ($flag:expr, $equal:expr, $history:expr) => {
                if let Some(equal) = $equal {
                    if $flag.comp_options.is_none() {
                        return Err(Error::BoolFlagEqualsValue(arg));
                    }
                    $history.push_flag($flag.id, equal.to_string());
                } else {
                    let res = $flag.supplement($history, args)?;
                    if let Some(res) = res {
                        return Ok(res);
                    }
                }
            };
        }

        match parse_flag(&arg, self.doing_external(args_ctx))? {
            ParsedFlag::SingleDash | ParsedFlag::DoubleDash | ParsedFlag::Empty => {
                supplement_arg(history, args_ctx, arg)?;
            }
            ParsedFlag::NotFlag => {
                let command = if args_ctx.has_seen_arg() {
                    None
                } else {
                    self.commands.iter().find(|c| arg == c.info.name)
                };
                match command {
                    Some(command) => {
                        history.push_command(command.id);
                        return command.supplement_recur(&mut None, history, args);
                    }
                    None => {
                        log::info!("No subcommand. Try fallback args.");
                        supplement_arg(history, args_ctx, arg)?;
                    }
                }
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

        self.supplement_recur(args_ctx_opt, history, args)
    }

    fn supplement_last(
        &self,
        args_ctx: &mut ArgsContext,
        history: &mut History,
        arg: String,
    ) -> Result<CompletionGroup> {
        let mut raise_empty_err = true;
        let ret: Vec<_> = match parse_flag(&arg, self.doing_external(args_ctx))? {
            ParsedFlag::Empty | ParsedFlag::NotFlag => {
                raise_empty_err = false;
                let cmd_slice = if args_ctx.has_seen_arg() {
                    log::info!("no completion for subcmd because we've already seen some args");
                    &[]
                } else {
                    log::debug!("completion for {} subcommands", self.commands.len());
                    self.commands
                };
                let cmd_iter = cmd_slice
                    .iter()
                    .map(|c| Completion::new(c.info.name, c.info.description).group("command"));
                let arg_comp = if let Some(arg_obj) = args_ctx.next_arg() {
                    log::debug!("completion for args {:?}", arg_obj.id);
                    (arg_obj.comp_options)(history, &arg)
                } else {
                    if cmd_slice.is_empty() {
                        return Err(Error::UnexpectedArg(arg));
                    }
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
                    .map(|c| c.value(|v| format!("--{body}={v}")))
                    .collect()
            }
            ParsedFlag::Shorts => {
                let resolved = self.resolve_shorts(history, &arg)?;
                if let Some(comp_options) = resolved.last_flag.comp_options {
                    raise_empty_err = false;
                    let value = resolved.value.unwrap_or("");
                    comp_options(history, value)
                        .into_iter()
                        .map(|c| c.value(|v| format!("{}{}", resolved.flag_part, v)))
                        .collect()
                } else {
                    log::debug!("list short flags with history {:?}", history);
                    history.push_pure_flag(resolved.last_flag.id);
                    self.flags(history)
                        .map(|f| f.gen_completion(Some(false)))
                        .flatten()
                        .map(|c| {
                            c.value(|v| {
                                let flag = &v[1..]; // skip the first '-' character
                                format!("{}{}", resolved.flag_part, flag)
                            })
                        })
                        .collect()
                }
            }
        };
        if ret.is_empty() && raise_empty_err {
            return Err(Error::NoPossibleCompletion);
        }
        Ok(CompletionGroup::new(ret, arg))
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
