use super::abstraction::{Arg, Command};

pub(super) fn flags<'a>(p: &Command<'a>) -> impl Iterator<Item = Arg<'a>> {
    p.get_arguments().filter(|a| !a.is_positional())
}

pub(super) fn args<'a>(p: &Command<'a>) -> impl Iterator<Item = Arg<'a>> {
    p.get_arguments().filter(|a| a.is_positional())
}

pub(super) fn non_help_subcmd<'a>(p: &Command<'a>) -> impl Iterator<Item = Command<'a>> {
    // TODO: Check if the help is default implementation
    p.get_subcommands().filter(|c| c.get_name() != "help")
}

pub(super) fn escape_help(help: &str) -> String {
    help.replace('\n', " ").replace('"', "\\\"")
}
