use super::NameType;
use super::abstraction::{Command, CommandMut, clap};
use super::utils;
use std::io::Write;
use utils::{gen_rust_name, to_snake_case};

#[cfg(feature = "clap-3")]
pub fn generate_default(
    cmd: &mut clap::Command<'static>,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let cmd = CommandMut(cmd);
    generate_inner(cmd, w)
}
#[cfg(feature = "clap-4")]
pub fn generate_default(cmd: &mut clap::Command, w: &mut impl Write) -> std::io::Result<()> {
    let cmd = CommandMut(cmd);
    generate_inner(cmd, w)
}

fn generate_inner(mut cmd: CommandMut, w: &mut impl Write) -> std::io::Result<()> {
    cmd.build();
    let cmd = cmd.to_const();
    generate_recur(&[], &cmd, w)
}

fn generate_recur(prev: &[String], cmd: &Command<'_>, w: &mut impl Write) -> std::io::Result<()> {
    let mut prefix = prev.join("::");
    if !prefix.is_empty() {
        prefix += "::";
    }

    for flag in utils::flags(cmd) {
        let takes_values = flag.takes_values();
        let possible_values = flag.get_possible_values();
        let is_const = !takes_values || !possible_values.is_empty();

        if !is_const {
            let name = flag.get_id().to_string();
            let rust_name = gen_rust_name(NameType::FLAG, &name, false);
            writeln!(w, "impl {prefix}{rust_name} for Supplements {{}}")?;
        }
    }

    for arg in utils::args(cmd) {
        let name = arg.get_id().to_string();
        let rust_name = gen_rust_name(NameType::ARG, &name, false);
        writeln!(w, "impl {prefix}{rust_name} for Supplements {{}}")?;
    }

    for sub_cmd in utils::non_help_subcmd(cmd) {
        let mut prev = prev.to_vec();
        let name = to_snake_case(sub_cmd.get_name());
        prev.push(name);
        generate_recur(&prev, &sub_cmd, w)?
    }

    Ok(())
}
