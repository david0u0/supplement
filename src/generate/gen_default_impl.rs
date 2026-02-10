use super::NameType;
use super::Setting;
use super::abstraction::{Command, CommandMut, clap};
use super::generate_mod_name;
use super::utils;
use std::io::Write;
use utils::gen_rust_name;

#[cfg(feature = "clap-3")]
pub fn generate_default(
    cmd: &mut clap::Command<'static>,
    setting: Setting,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let cmd = CommandMut(cmd);
    generate_inner(cmd, w)
}
#[cfg(feature = "clap-4")]
pub fn generate_default(
    cmd: &mut clap::Command,
    setting: Setting,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let cmd = CommandMut(cmd);
    generate_inner(cmd, w)
}

fn generate_inner(mut cmd: CommandMut, w: &mut impl Write) -> std::io::Result<()> {
    cmd.build();
    let cmd = cmd.to_const();
    generate_recur(&[], &[], &cmd, w)
}

fn generate_recur(
    prev: &[String],
    global_flags: &[String],
    cmd: &Command<'_>,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let mut global_flags = global_flags.to_vec();
    let mut prefix = prev.join("::");
    if !prefix.is_empty() {
        prefix += "::";
    }

    for flag in utils::flags(cmd) {
        let name = flag.get_id().to_string();
        if flag.is_global_set() {
            if global_flags.iter().any(|f| *f == name) {
                continue;
            } else {
                global_flags.push(name.clone());
            }
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
        let name = generate_mod_name(sub_cmd.get_name());
        prev.push(name);
        generate_recur(&prev, &global_flags, &sub_cmd, w)?
    }

    Ok(())
}
