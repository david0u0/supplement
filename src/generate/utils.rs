use super::NameType;
use super::abstraction::{Arg, Command};

pub(super) fn flags<'a>(p: &Command<'a>) -> impl Iterator<Item = Arg<'a>> {
    p.get_arguments()
        .filter(|a| !a.is_positional() && a.get_id() != "help")
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

fn to_pascal_case(s: &str) -> String {
    let mut ret = String::new();
    for s in to_snake_case(s).split('_') {
        let mut chars = s.chars();
        match chars.next() {
            None => continue,
            Some(first) => {
                ret += &first.to_uppercase().to_string();
                ret += &(chars.collect::<String>());
            }
        }
    }
    ret
}

pub(crate) fn to_snake_case(s: &str) -> String {
    s.replace('-', "_").to_lowercase() // TODO
}

pub(crate) fn to_screaming_snake_case(s: &str) -> String {
    s.replace('-', "_").to_uppercase() // TODO
}

pub(crate) fn gen_rust_name(ty: NameType, name: &str, is_const: bool) -> String {
    let mut ret = ty.to_string();
    if is_const {
        ret = ret.to_uppercase();
    }

    if is_const {
        ret += "_";
        ret += &to_screaming_snake_case(name);
    } else {
        ret += &to_pascal_case(name);
    }

    ret
}
