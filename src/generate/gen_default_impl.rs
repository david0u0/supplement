use super::abstraction::{Command, CommandMut, clap};
use super::generate_mod_name;
use super::utils;
use super::{Config, NameType, Trace};
use std::io::Write;
use utils::gen_rust_name;

#[cfg(feature = "clap-3")]
pub fn generate_default(
    cmd: &mut clap::Command<'static>,
    config: Config,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let cmd = CommandMut(cmd);
    generate_inner(cmd, config, w)
}
#[cfg(feature = "clap-4")]
pub fn generate_default(
    cmd: &mut clap::Command,
    config: Config,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let cmd = CommandMut(cmd);
    generate_inner(cmd, config, w)
}

fn generate_inner(
    mut cmd: CommandMut,
    mut config: Config,
    w: &mut impl Write,
) -> std::io::Result<()> {
    cmd.build();
    let cmd = cmd.to_const();
    generate_recur(&[], &[], &mut config, &cmd, w)
}

fn join_mod_prefix(prev: &[Trace]) -> String {
    let mut ret = String::new();
    for t in prev.iter() {
        ret += &t.mod_name;
        ret += "::";
    }
    ret
}

fn generate_recur(
    prev: &[Trace],
    global_flags: &[String],
    config: &mut Config,
    cmd: &Command<'_>,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let mut global_flags = global_flags.to_vec();
    let prefix = join_mod_prefix(prev);

    for flag in utils::flags(cmd) {
        let name = flag.get_id().to_string();
        if flag.is_global_set() {
            if global_flags.iter().any(|f| *f == name) {
                continue;
            } else {
                global_flags.push(name.clone());
            }
        }

        if config.is_ignored(prev, &name) {
            continue;
        }

        let takes_values = flag.takes_values();
        let possible_values = flag.get_possible_values();
        let is_const = !takes_values || !possible_values.is_empty();

        if !is_const {
            let rust_name = gen_rust_name(NameType::FLAG, &name, false);
            writeln!(w, "impl {prefix}{rust_name} for Supplements {{}}")?;
        }
    }

    for arg in utils::args(cmd) {
        let name = arg.get_id().to_string();
        let rust_name = gen_rust_name(NameType::ARG, &name, false);
        writeln!(w, "impl {prefix}{rust_name} for Supplements {{}}")?;
    }

    if cmd.is_allow_external_subcommands_set() {
        let rust_name = NameType::EXTERNAL.to_string();
        writeln!(w, "impl {prefix}{rust_name} for Supplements {{}}")?;
    }

    for sub_cmd in utils::non_help_subcmd(cmd) {
        let mut prev = prev.to_vec();
        let cmd_id = sub_cmd.get_name().to_string();
        if config.is_ignored(&prev, &cmd_id) {
            continue;
        }
        let mod_name = generate_mod_name(&cmd_id);
        prev.push(Trace { cmd_id, mod_name });
        generate_recur(&prev, &global_flags, config, &sub_cmd, w)?
    }

    Ok(())
}
