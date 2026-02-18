mod flag;
pub use crate::id;
pub use flag::{CompleteWithEqual, Flag, flag_type};

use std::iter::Peekable;

use crate::arg_context::ArgsContext;
use crate::completion::{CompletionGroup, Unready};
use crate::error::Error;
use crate::parsed_flag::ParsedFlag;
use crate::{Completion, History, Result};
use std::fmt::Debug;

pub struct Arg<ID> {
    pub id: id::Valued<ID>,
    pub max_values: usize,
}

/// The object to represent a command.
/// Usually this object is a constant created by code-gen,
/// and user can just call `supplement` function for CLI completion.
pub struct Command<ID: 'static> {
    pub name: &'static str,
    pub description: &'static str,
    pub all_flags: &'static [Flag<ID>],
    pub args: &'static [Arg<ID>],
    pub commands: &'static [Command<ID>],
}

fn supplement_arg<ID: PartialEq + Copy + Debug>(
    history: &mut History<ID>,
    ctx: &mut ArgsContext<ID>,
    arg: String,
) -> Result {
    let Some(arg_obj) = ctx.next_arg() else {
        return Err(Error::UnexpectedArg(arg));
    };
    history.push_valued(arg_obj.id, arg);
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

fn check_no_flag<ID>(arg: String, v: Vec<Completion>) -> Result<CompletionGroup<ID>> {
    if v.is_empty() {
        return Err(Error::UnexpectedFlag);
    }
    Ok(CompletionGroup::new_ready(v, arg))
}

impl<ID: 'static + Copy + PartialEq + Debug> Command<ID> {
    /// The main entry point of CLI completion.
    ///
    /// ```
    /// # use supplements::core::*;
    /// # use supplements::*;
    /// # use supplements::completion::CompletionGroup;
    /// # type ID = u32;
    /// const fn create_cmd(name: &'static str, subcmd: &'static [Command<ID>]) -> Command<ID> {
    ///     Command {
    ///         name,
    ///         description: "",
    ///         all_flags: &[],
    ///         args: &[],
    ///         commands: subcmd,
    ///     }
    /// }
    ///
    /// const CHECKOUT: Command<ID> = create_cmd("checkout", &[]);
    /// const LOG: Command<ID> = create_cmd("log", &[]);
    /// let root = create_cmd("qit", &[CHECKOUT, LOG]);
    ///
    /// let args = ["qit", ""].iter().map(|s| s.to_string());
    /// let (_history, grp) = root.supplement(args).unwrap();
    /// let ready = match grp {
    ///     CompletionGroup::Ready(ready) => ready,
    ///     CompletionGroup::Unready{ .. } => unreachable!(),
    /// };
    ///
    /// let comps = ready.into_inner().0;
    /// assert_eq!(comps[0], Completion::new("checkout", "").group("command"));
    /// assert_eq!(comps[1], Completion::new("log", "").group("command"));
    /// ```
    pub fn supplement(
        &self,
        args: impl Iterator<Item = String>,
    ) -> Result<(History<ID>, CompletionGroup<ID>)> {
        let mut history = History::<ID>::new();
        let grp = self.supplement_with_history(&mut history, args)?;
        Ok((history, grp))
    }

    pub fn supplement_with_history(
        &self,
        history: &mut History<ID>,
        mut args: impl Iterator<Item = String>,
    ) -> Result<CompletionGroup<ID>> {
        args.next(); // ignore the first arg which is the program's name

        let mut args = args.peekable();
        if args.peek().is_none() {
            return Err(Error::ArgsTooShort);
        }

        self.supplement_recur(&mut None, history, &mut args)
    }

    fn doing_external(&self, ctx: &ArgsContext<'_, ID>) -> bool {
        let has_subcmd = !self.commands.is_empty();
        has_subcmd && ctx.has_seen_arg()
    }
    fn flags(&self, history: &History<ID>) -> impl Iterator<Item = &Flag<ID>> {
        self.all_flags.iter().filter(|f| {
            if !f.once {
                true
            } else {
                let exists = f.exists_in_history(history);
                if exists {
                    log::debug!("flag {:?} already exists", f.name());
                }
                !exists
            }
        })
    }

    fn find_flag<F: FnMut(&Flag<ID>) -> bool>(
        &self,
        arg: &str,
        history: &History<ID>,
        mut filter: F,
    ) -> Result<&Flag<ID>> {
        match self.flags(history).find(|f| filter(f)) {
            Some(flag) => Ok(flag),
            None => Err(Error::FlagNotFound(arg.to_owned())),
        }
    }

    fn find_long_flag(&self, flag: &str, history: &History<ID>) -> Result<&Flag<ID>> {
        self.find_flag(flag, history, |f| f.long.iter().any(|l| *l == flag))
    }
    fn find_short_flag(&self, flag: char, history: &History<ID>) -> Result<&Flag<ID>> {
        self.find_flag(&flag.to_string(), history, |f| {
            f.short.iter().any(|s| *s == flag)
        })
    }

    fn supplement_recur(
        &self,
        args_ctx_opt: &mut Option<ArgsContext<'_, ID>>,
        history: &mut History<ID>,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Result<CompletionGroup<ID>> {
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
                    match $flag.ty {
                        flag_type::Type::Valued(flag) => flag.push($history, equal.to_string()),
                        _ => return Err(Error::BoolFlagEqualsValue(arg)),
                    }
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
        args_ctx: &mut ArgsContext<'_, ID>,
        history: &mut History<ID>,
        arg: String,
    ) -> Result<CompletionGroup<ID>> {
        let ret: CompletionGroup<ID> = match parse_flag(&arg, self.doing_external(args_ctx)) {
            ParsedFlag::Empty | ParsedFlag::NotFlag => {
                let cmd_slice = if args_ctx.has_seen_arg() {
                    log::info!("no completion for subcmd because we've already seen some args");
                    &[]
                } else {
                    log::debug!("completion for {} subcommands", self.commands.len());
                    self.commands
                };
                let cmd_comps: Vec<_> = cmd_slice
                    .iter()
                    .map(|c| Completion::new(c.name, c.description).group("command"))
                    .collect();
                if let Some(arg_obj) = args_ctx.next_arg() {
                    log::debug!("completion for args {:?}", arg_obj.id);
                    let unready = Unready::new(String::new(), arg.clone()).preexist(cmd_comps);
                    match arg_obj.id.id() {
                        Some(id) => CompletionGroup::Unready {
                            id,
                            unready,
                            value: arg,
                        },
                        None => unready.to_ready_grp(vec![]), // TODO: arg can have possible value?
                    }
                } else {
                    if cmd_comps.is_empty() {
                        return Err(Error::UnexpectedArg(arg));
                    }
                    CompletionGroup::new_ready(cmd_comps, arg)
                }
            }
            ParsedFlag::DoubleDash | ParsedFlag::Long { equal: None, .. } => check_no_flag(
                arg,
                self.flags(history)
                    .map(|f| f.gen_completion(Some(true)))
                    .flatten()
                    .collect(),
            )?,
            ParsedFlag::SingleDash => check_no_flag(
                arg,
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
                let valued = match flag.ty {
                    flag_type::Type::Valued(valued) => valued,
                    _ => return Err(Error::BoolFlagEqualsValue(arg)),
                };
                let prefix = format!("--{body}=");
                let value = value.to_string();
                let unready = Unready::new(prefix, arg);

                match valued.id.id() {
                    Some(id) => CompletionGroup::Unready { unready, value, id },
                    None => unready.to_ready_grp(valued.comp_from_possible()),
                }
            }
            ParsedFlag::Shorts => self.supplement_last_short_flags(history, arg)?,
        };
        Ok(ret)
    }

    fn resolve_shorts<'a, 'b>(
        &'b self,
        history: &mut History<ID>,
        shorts: &'a str,
    ) -> Result<ResolvedMultiShort<'a, 'b, ID>> {
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
                    if matches!(flag.ty, flag_type::Type::Bool(_)) {
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
                    let valued = match flag.ty {
                        flag_type::Type::Bool(inner) => {
                            inner.push(history);
                            continue;
                        }
                        flag_type::Type::Valued(valued) => valued,
                    };

                    match valued.complete_with_equal {
                        CompleteWithEqual::Must => {
                            return Err(Error::RequiresEqual(flag.name()));
                        }
                        CompleteWithEqual::Optional => {
                            // TODO: Maybe one day clap will tell us.
                            log::info!(
                                "Optional flag {} doesn't have value. Push an empty string to history because we don't know its default value (clap wouldn't tell us).",
                                flag.name(),
                            );
                            valued.push(history, String::new());
                        }
                        CompleteWithEqual::NoNeed => {
                            return Ok(ResolvedMultiShort {
                                flag_part: &shorts[..len],
                                last_flag: flag,
                                value: Some(&shorts[len..]),
                            });
                        }
                    }
                }
            }
        }
    }

    fn supplement_last_short_flags(
        &self,
        history: &mut History<ID>,
        arg: String,
    ) -> Result<CompletionGroup<ID>> {
        let resolved = self.resolve_shorts(history, &arg)?;
        let flag = resolved.last_flag;
        let ret = match flag.ty {
            flag_type::Type::Valued(valued) => {
                let value = resolved.value.unwrap_or_default().to_string();
                let mut eq = "";
                let mut more = vec![];
                if valued.complete_with_equal != CompleteWithEqual::NoNeed {
                    if resolved.value.is_none() {
                        // E.g. `cmd -af`
                        eq = "=";
                        if valued.complete_with_equal == CompleteWithEqual::Optional {
                            // Want: `-af=opt1`, `-af=opt2`, `-af`
                            // NOTE that we don't want `-afx`, `-afy` where x and y are other flags. That's too much.
                            more =
                                vec![Completion::new(resolved.flag_part, &flag.get_description())];
                        }
                    } else {
                        // E.g. `cmd -af=xyz` or `cmd -af=`.
                        // It's impossible to be `cmd -afxyz`, either it throw error or `f` won't be the last flag.
                        // Want: `-af=opt1`, `-af=opt2`
                    }
                }
                let prefix = format!("{}{}", resolved.flag_part, eq);
                let unready = Unready::new(prefix, arg).preexist(more);

                match valued.id.id() {
                    Some(id) => CompletionGroup::Unready { unready, value, id },
                    None => unready.to_ready_grp(valued.comp_from_possible()),
                }
            }
            flag_type::Type::Bool(inner) => {
                log::debug!("list short flags with history {:?}", history);
                inner.push(history);
                let comps = self
                    .flags(history)
                    .map(|f| f.gen_completion(Some(false)))
                    .flatten()
                    .map(|c| {
                        c.value(|v| {
                            let flag = &v[1..]; // skip the first '-' character
                            format!("{}{}", resolved.flag_part, flag)
                        })
                    })
                    .collect();
                check_no_flag(arg, comps)?
            }
        };
        Ok(ret)
    }
}

#[derive(Clone, Copy)]
struct ResolvedMultiShort<'a, 'b, ID> {
    flag_part: &'a str,
    last_flag: &'b Flag<ID>,
    value: Option<&'a str>,
}
