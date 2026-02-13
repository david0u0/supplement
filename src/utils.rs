use std::iter::Peekable;

use crate::arg_context::ArgsContext;
use crate::completion::CompletionGroup;
use crate::error::Error;
use crate::id;
use crate::parsed_flag::ParsedFlag;
use crate::{Completion, History, Result};

type CompOption = fn(&History, &str) -> Vec<Completion>;

pub struct Flag {
    pub id: id::Flag,
    pub short: &'static [char],
    pub long: &'static [&'static str],
    pub description: &'static str,
    pub comp_options: Option<CompOption>,
    pub once: bool,
}
pub struct Arg {
    pub id: id::Arg,
    pub comp_options: CompOption,
    pub max_values: usize,
}

/// The object to represent a command.
/// Usually this object is a constant created by code-gen,
/// and user can just call `supplement` function for CLI completion.
pub struct Command {
    pub id: id::NoVal,
    pub name: &'static str,
    pub description: &'static str,
    pub all_flags: &'static [Flag],
    pub args: &'static [Arg],
    pub commands: &'static [Command],
}

impl Flag {
    fn gen_completion(&self, is_long: Option<bool>) -> impl Iterator<Item = Completion> {
        let (long, short) = match is_long {
            None => (self.long, self.short),
            Some(true) => (self.long, &[] as &[char]),
            Some(false) => (&[] as &[&str], self.short),
        };
        long.iter()
            .map(|l| Completion::new(&format!("--{l}"), self.description))
            .chain(
                short
                    .iter()
                    .map(|s| Completion::new(&format!("-{s}"), self.description)),
            )
    }

    fn exists_in_history(&self, history: &History) -> bool {
        match self.id {
            id::Flag::No(id) => history.find(id).is_some(),
            id::Flag::Single(id) => history.find(id).is_some(),
            id::Flag::Multi(id) => history.find(id).is_some(),
        }
    }
    fn push_pure_flag(&self, history: &mut History) {
        match self.id {
            id::Flag::No(id) => history.push_no_val(id),
            _ => unreachable!(),
        }
    }
    fn push_flag(&self, history: &mut History, arg: String) {
        match self.id {
            id::Flag::Single(id) => history.push_single_val(id, arg),
            id::Flag::Multi(id) => history.push_multi_val(id, arg),
            _ => unreachable!(),
        }
    }

    fn supplement(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Result<Option<CompletionGroup>> {
        let Some(comp_options) = self.comp_options else {
            self.push_pure_flag(history);
            return Ok(None);
        };

        let arg = args.next().unwrap();
        match parse_flag(&arg, false) {
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

        self.push_flag(history, arg);
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
fn parse_flag(s: &str, disable_flag: bool) -> ParsedFlag<'_> {
    if disable_flag {
        log::info!("flag is disabled: {}", s);
        ParsedFlag::NotFlag
    } else {
        ParsedFlag::new(s)
    }
}

impl Command {
    /// The main entry point of CLI completion.
    ///
    /// ```
    /// # use supplements::*;
    /// # use supplements::completion::CompletionGroup;
    /// const fn create_cmd(name: &'static str, subcmd: &'static [Command]) -> Command {
    ///     Command {
    ///         id: id::NoVal::new(0, name),
    ///         name,
    ///         description: "",
    ///         all_flags: &[],
    ///         args: &[],
    ///         commands: subcmd,
    ///     }
    /// }
    ///
    /// const cmd1: Command = create_cmd("cmd1", &[]);
    /// const cmd2: Command = create_cmd("cmd2", &[]);
    /// let root = create_cmd("root", &[cmd1, cmd2]);
    ///
    /// let args = ["root", ""].iter().map(|s| s.to_string());
    /// let comps: CompletionGroup = root.supplement(args).unwrap();
    /// let comps = comps.into_inner().0;
    /// assert_eq!(comps[0], Completion::new("cmd1", "").group("command"));
    /// assert_eq!(comps[1], Completion::new("cmd2", "").group("command"));
    /// ```
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
                let exists = f.exists_in_history(history);
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
        self.find_flag(flag, history, |f| f.long.iter().any(|l| *l == flag))
    }
    fn find_short_flag(&self, flag: char, history: &History) -> Result<&Flag> {
        self.find_flag(&flag.to_string(), history, |f| {
            f.short.iter().any(|s| *s == flag)
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
                    $flag.push_flag($history, equal.to_string());
                } else {
                    let res = $flag.supplement($history, args)?;
                    if let Some(res) = res {
                        return Ok(res);
                    }
                }
            };
        }

        match parse_flag(&arg, self.doing_external(args_ctx)) {
            ParsedFlag::SingleDash | ParsedFlag::DoubleDash | ParsedFlag::Empty => {
                supplement_arg(history, args_ctx, arg)?;
            }
            ParsedFlag::NotFlag => {
                let command = if args_ctx.has_seen_arg() {
                    None
                } else {
                    self.commands.iter().find(|c| arg == c.name)
                };
                match command {
                    Some(command) => {
                        history.push_no_val(command.id);
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
        fn check_no_flag(v: Vec<Completion>) -> Result<Vec<Completion>> {
            if v.is_empty() {
                return Err(Error::UnexpectedFlag);
            }
            Ok(v)
        }

        let ret: Vec<_> = match parse_flag(&arg, self.doing_external(args_ctx)) {
            ParsedFlag::Empty | ParsedFlag::NotFlag => {
                let cmd_slice = if args_ctx.has_seen_arg() {
                    log::info!("no completion for subcmd because we've already seen some args");
                    &[]
                } else {
                    log::debug!("completion for {} subcommands", self.commands.len());
                    self.commands
                };
                let cmd_iter = cmd_slice
                    .iter()
                    .map(|c| Completion::new(c.name, c.description).group("command"));
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
            ParsedFlag::DoubleDash | ParsedFlag::Long { equal: None, .. } => check_no_flag(
                self.flags(history)
                    .map(|f| f.gen_completion(Some(true)))
                    .flatten()
                    .collect(),
            )?,
            ParsedFlag::SingleDash => check_no_flag(
                self.flags(history)
                    .map(|f| f.gen_completion(None))
                    .flatten()
                    .collect(),
            )?,
            ParsedFlag::Long {
                equal: Some(value),
                body,
            } => {
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
                    let value = resolved.value.unwrap_or("");
                    comp_options(history, value)
                        .into_iter()
                        .map(|c| c.value(|v| format!("{}{}", resolved.flag_part, v)))
                        .collect()
                } else {
                    log::debug!("list short flags with history {:?}", history);
                    resolved.last_flag.push_pure_flag(history);
                    check_no_flag(
                        self.flags(history)
                            .map(|f| f.gen_completion(Some(false)))
                            .flatten()
                            .map(|c| {
                                c.value(|v| {
                                    let flag = &v[1..]; // skip the first '-' character
                                    format!("{}{}", resolved.flag_part, flag)
                                })
                            })
                            .collect(),
                    )?
                }
            }
        };
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

                    flag.push_pure_flag(history);
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
