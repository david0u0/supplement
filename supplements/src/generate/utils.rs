use clap::{Arg, Command, builder::StyledStr};

pub(super) fn flags(p: &Command) -> impl Iterator<Item = &Arg> {
    p.get_arguments().filter(|a| !a.is_positional())
}

pub(super) fn args(p: &Command) -> impl Iterator<Item = &Arg> {
    p.get_arguments().filter(|a| a.is_positional())
}

pub(super) fn non_help_subcmd(p: &Command) -> impl Iterator<Item = &Command> {
    // TODO: Check if the help is is default implementation
    p.get_subcommands().filter(|c| c.get_name() != "help")
}

pub(super) fn escape_help(help: &StyledStr) -> String {
    help.to_string().replace('\n', " ").replace('"', "\\\"")
}
