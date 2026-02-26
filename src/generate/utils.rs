use super::abstraction::{Arg, Command};
use super::{NameType, Trace};
use crate::core::CompleteWithEqual;

pub(super) fn flags<'a>(p: &Command<'a>) -> impl Iterator<Item = Arg<'a>> {
    let custom_help = p.is_disable_help_flag_set();
    p.get_arguments()
        .filter(move |a| !a.is_positional() && (a.get_id() != "help" || custom_help))
}

pub(super) fn args<'a>(p: &Command<'a>) -> impl Iterator<Item = Arg<'a>> {
    p.get_arguments().filter(|a| a.is_positional())
}

pub(super) fn non_help_subcmd<'a>(p: &Command<'a>) -> impl Iterator<Item = Command<'a>> {
    let custom_help = p.is_disable_help_subcommand_set();
    p.get_subcommands()
        .filter(move |c| custom_help || c.get_name() != "help")
}

pub(super) fn escape_help(help: &str) -> String {
    help.replace('\n', " ").replace('"', "\\\"")
}

pub(crate) fn to_pascal_case(s: &str) -> String {
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

pub(crate) fn gen_enum_name(ty: NameType, name: &str) -> String {
    if ty == NameType::EXTERNAL {
        return ty.to_string();
    }
    let name = to_pascal_case(name);
    format!("{ty}{name}")
}

pub(crate) fn gen_rust_name(ty: NameType, name: &str) -> String {
    let mut ret = ty.to_string();
    ret = ret.to_uppercase();
    if !name.is_empty() {
        ret += "_";
        ret += &to_screaming_snake_case(name);
    }
    ret
}

pub fn compute_flag_equal(arg: Arg<'_>, strict: bool) -> Result<&'static str, &'static str> {
    let s = match compute_flag_equal_enum(arg, strict)? {
        CompleteWithEqual::Optional => "CompleteWithEqual::Optional",
        CompleteWithEqual::Must => "CompleteWithEqual::Must",
        CompleteWithEqual::NoNeed => "CompleteWithEqual::NoNeed",
    };
    Ok(s)
}
fn compute_flag_equal_enum(arg: Arg<'_>, strict: bool) -> Result<CompleteWithEqual, &'static str> {
    let optional = arg.get_min_num_args() == 0;
    if !arg.is_require_equals_set() {
        if optional {
            const MSG: &str = "\
Optional flags without require_equals is weird \
e.g. `ls --color some/dir` will supply `some/dir` to `color` flag.
supplement will just treat the next completion as if it's not optional \
i.e. `ls --color <TAB>` results in [always, auto, never], not the file completion.";
            if strict {
                return Err(MSG);
            } else {
                log::warn!("{}", MSG);
            }
        }
        // This can be `Optional`, but it will make the completion unnecessarily longer
        return Ok(CompleteWithEqual::NoNeed);
    }
    if optional {
        Ok(CompleteWithEqual::Optional)
    } else {
        Ok(CompleteWithEqual::Must)
    }
}

pub fn get_id_value(prev: &[Trace], ty: NameType, id: &str) -> String {
    // pub enum ID {
    //     A,
    //     B,
    //     C(cmd_c::ID)
    //     D(cmd_d::ID)
    // }
    // mod cmd_d {
    //     pub enum ID {
    //         E,
    //         F(cmd_f::ID)
    //     }
    //     mod cmd_f {
    //         pub enum ID {
    //             G
    //         }
    //     }
    // }
    //
    // level = 0 => ID::A
    // level = 1 => super::ID::C(ID::E)
    // level = 2 => super::super::ID::C(super::ID::F(ID::G))

    let mut ret = String::new();
    let mut level = prev.len();
    for trace in prev.iter() {
        let super_str = "super::".repeat(level);
        level -= 1;
        let enum_name = gen_enum_name(NameType::COMMAND, &trace.cmd_id);
        ret += &format!("{super_str}ID::{enum_name}(");
    }

    let enum_name = gen_enum_name(ty, id);
    ret += &format!("ID::{enum_name}");
    ret += &")".repeat(prev.len());
    ret
}
