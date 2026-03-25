use crate::clap::{Command as ClapCommand, CommandFactory};
use crate::gen_prelude::*;
use crate::{CompletionGroup, Result, id};
use std::fmt::Debug;

/// Trait for CLI completion.
///
/// This is the primary way to create a [`Command`] object, which can be used for completion.
/// Or even simpler, directly call [`Supplement::supplement`] to start the CLI completion.
pub trait Supplement: CommandFactory {
    /// ID can be used to uniquely identify a CLI flag or arg.
    ///
    /// With the derive macro, the ID is quite convoluted and hard to match on.
    /// Users are expected to use [`crate::helper::id!`] to help construct the match arms.
    type ID: Debug + PartialEq + Copy + 'static;

    type Ctx: Default + Debug + PartialEq + Copy + 'static;

    fn id_from_cmd(cmd: &[impl AsRef<str>]) -> Option<(Option<Self::ID>, u32)>;
    fn gen_cmd() -> Command<Self::ID> {
        let mut cmd = Self::command();
        cmd.build();
        gen_cmd_inner::<Self>(true, &cmd, &[], &mut vec![])
    }
    fn supplement(args: impl Iterator<Item = String>) -> Result<(Seen, CompletionGroup<Self::ID>)> {
        let cmd = Self::gen_cmd();
        cmd.supplement(args)
    }
}

#[derive(Clone)]
struct GlobalFlag<ID> {
    id: Option<ID>,
    seen_id: u32,
    name: String,
    _ignored: bool, // TODO: implement this somehow?
}

fn gen_cmd_inner<Root: Supplement>(
    first: bool,
    cmd: &ClapCommand,
    trace: &[String],
    global_flags: &mut Vec<GlobalFlag<Root::ID>>,
) -> Command<Root::ID> {
    let name: Cow<'_, str> = Cow::Owned(cmd.get_name().to_string());
    let mut trace = trace.to_vec();
    if !first {
        trace.push(name.to_string());
    }
    let description = Cow::Owned(cmd.get_about().map(|s| s.to_string()).unwrap_or_default());

    let flags: Vec<Flag<Root::ID>> = cmd
        .get_arguments()
        .filter(|a| !a.is_positional())
        .filter(|a| {
            // TODO: custom help
            let id = a.get_id().as_str();
            id != "help"
        })
        .map(|arg| gen_flag::<Root>(arg, &trace, global_flags))
        .collect();

    let mut args: Vec<Arg<Root::ID>> = cmd
        .get_arguments()
        .filter(|a| a.is_positional())
        .map(|arg| gen_arg::<Root>(arg, &trace))
        .collect();

    let commands: Vec<Command<Root::ID>> = cmd
        .get_subcommands()
        .filter(|c| c.get_name() != "help") // TODO: custom help
        .map(|sub| gen_cmd_inner::<Root>(false, sub, &trace, global_flags))
        .collect();

    if cmd.is_allow_external_subcommands_set() {
        trace.push("".to_owned()); // NOTE: "" is for external subcommand
        let (id, seen_id) =
            Root::id_from_cmd(&trace).unwrap_or_else(|| panic!("{trace:?} not found"));
        args.push(Arg {
            id,
            seen_id: id::MultiVal::new(seen_id).into(),
            max_values: usize::MAX,
            possible_values: CowOwned::Borrow(&[]),
        })
    }

    Command {
        name,
        description,
        all_flags: CowSlice::Owned(flags),
        args: CowSlice::Owned(args),
        commands: CowSlice::Owned(commands),
    }
}

fn gen_flag<Root: Supplement>(
    arg: &crate::clap::Arg,
    trace: &[String],
    global_flags: &mut Vec<GlobalFlag<Root::ID>>,
) -> Flag<Root::ID> {
    let name = arg.get_id();
    let mut trace = trace.to_vec();
    trace.push(name.to_string());

    let (id, seen_id) = if arg.is_global_set() {
        if let Some(prev) = global_flags.iter().find(|f| &f.name == name) {
            log::info!("get existing global flag {name}");
            (prev.id, prev.seen_id)
        } else {
            log::info!("get new global flag {name}");
            let (id, seen_id) =
                Root::id_from_cmd(&trace).unwrap_or_else(|| panic!("{trace:?} not found"));
            global_flags.push(GlobalFlag {
                id,
                seen_id,
                name: name.to_string(),
                _ignored: false,
            });
            (id, seen_id)
        }
    } else {
        Root::id_from_cmd(&trace).unwrap_or_else(|| panic!("{trace:?} not found"))
    };

    let shorts: Vec<char> = arg.get_short_and_visible_aliases().unwrap_or_default();
    let longs: Vec<String> = arg
        .get_long_and_visible_aliases()
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    let description = Cow::Owned(arg.get_help().map(|s| s.to_string()).unwrap_or_default());

    let num_args = arg.get_num_args();
    let takes_values = num_args.map(|n| n.takes_values()).unwrap_or(false);

    // TODO: re-implement `generate/mod.rs`: `let (once, ty) = match flag.get_action() ...`
    let once = !arg.is_global_set();

    let ty = if takes_values {
        let possible_values: Vec<(String, String)> = arg
            .get_value_parser()
            .possible_values()
            .map(|pvs| {
                pvs.map(|pv| {
                    (
                        pv.get_name().to_string(),
                        pv.get_help().map(|h| h.to_string()).unwrap_or_default(),
                    )
                })
                .collect()
            })
            .unwrap_or_default();

        let complete_with_equal = if arg.is_require_equals_set() {
            let min = num_args.map(|n| n.min_values()).unwrap_or(1);
            if min == 0 {
                CompleteWithEqual::Optional
            } else {
                CompleteWithEqual::Must
            }
        } else {
            CompleteWithEqual::NoNeed
        };

        let max = num_args.map(|n| n.max_values()).unwrap_or(1);
        let seen_id = if max > 1 {
            id::MultiVal::new(seen_id).into()
        } else {
            id::SingleVal::new(seen_id).into()
        };

        flag_type::Type::new_valued(
            id,
            seen_id,
            complete_with_equal,
            CowOwned::Owned(possible_values),
        )
    } else {
        // TODO: this ID is still generated?
        flag_type::Type::new_bool(id::NoVal::new(seen_id))
    };

    Flag {
        ty,
        short: CowSlice::Owned(shorts),
        long: CowOwned::Owned(longs),
        description,
        once,
    }
}

fn gen_arg<Root: Supplement>(arg: &crate::clap::Arg, trace: &[String]) -> Arg<Root::ID> {
    let name = arg.get_id();
    let mut trace = trace.to_vec();
    trace.push(name.to_string());

    let max_values = arg.get_num_args().expect("built").max_values();

    let (id, seen_id) = Root::id_from_cmd(&trace).unwrap_or_else(|| panic!("{trace:?} not found"));

    let possible_values: Vec<(String, String)> = arg
        .get_value_parser()
        .possible_values()
        .map(|pvs| {
            pvs.map(|pv| {
                (
                    pv.get_name().to_string(),
                    pv.get_help().map(|h| h.to_string()).unwrap_or_default(),
                )
            })
            .collect()
        })
        .unwrap_or_default();

    let seen_id = if max_values > 1 {
        id::MultiVal::new(seen_id).into()
    } else {
        id::SingleVal::new(seen_id).into()
    };

    Arg {
        id,
        seen_id,
        max_values,
        possible_values: CowOwned::Owned(possible_values),
    }
}
