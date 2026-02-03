use clap::{
    Command,
    builder::{ArgAction, PossibleValue},
};
use std::borrow::Cow;
use std::io::Write;

mod utils;

pub fn generate(cmd: &mut Command, w: &mut impl Write) -> std::io::Result<()> {
    cmd.build();

    writeln!(w, "pub struct Supplements;")?;
    generate_recur(0, "", cmd, w)
}

struct NameType(&'static str);
impl NameType {
    const FLAG: Self = NameType("Flag");
    const ARG: Self = NameType("Arg");
    const COMMAND: Self = NameType("CMD");
}
impl std::fmt::Display for NameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
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

fn to_snake_case(s: &str) -> String {
    s.replace('-', "_") // TODO
}

fn to_screaming_snake_case(s: &str) -> String {
    s.replace('-', "_").to_uppercase() // TODO
}

fn gen_rust_name(ty: NameType, name: &str, is_const: bool) -> String {
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

struct JoinQuotes<I>(Option<char>, I);
impl<T, I> std::fmt::Display for JoinQuotes<I>
where
    T: std::fmt::Display,
    I: Iterator<Item = T> + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for t in self.1.clone() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            if let Some(quote) = self.0 {
                write!(f, "{}{}{}", quote, t, quote)?;
            } else {
                write!(f, "{t}")?;
            }
        }
        Ok(())
    }
}

struct CompOptionDisplay<'a>(&'a [PossibleValue]);
impl<'a> std::fmt::Display for CompOptionDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            write!(f, "None")?;
        } else {
            write!(f, "Some(|_, _| vec![")?;
            let mut first = true;
            for p in self.0.iter() {
                if first {
                    first = false;
                } else {
                    write!(f, ", ")?;
                }
                write!(
                    f,
                    "Completion::new(\"{}\", \"{}\")",
                    p.get_name(),
                    p.get_help().unwrap_or_default()
                )?
            }
            write!(f, "])")?;
        }
        Ok(())
    }
}

fn generate_args_in_cmd(
    indent: &str,
    cmd: &Command,
    w: &mut impl Write,
) -> std::io::Result<Vec<String>> {
    let mut args_names = vec![];
    for arg in utils::args(cmd) {
        let name = arg.get_id().to_string();

        log::debug!("generating arg {}", name);

        let num_args = arg.get_num_args().expect("built");
        let max_values = num_args.max_values();
        let rust_name = gen_rust_name(NameType::ARG, &name, false);

        writeln!(
            w,
            "\
{indent}pub trait {rust_name} {{
{indent}    const ID: id::Arg = id::Arg::new(line!(), \"{name}\");
{indent}    const OBJ: Arg = Arg {{
{indent}        id: Self::ID,
{indent}        comp_options: Self::comp_options,
{indent}        max_values: {max_values},
{indent}    }};

{indent}    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {{
{indent}        vec![]
{indent}    }}
{indent}}}"
        )?;

        args_names.push(rust_name);
    }

    Ok(args_names)
}

fn generate_flags_in_cmd(
    indent: &str,
    cmd: &Command,
    w: &mut impl Write,
) -> std::io::Result<Vec<(bool, String)>> {
    let mut flag_names = vec![];

    for flag in utils::flags(cmd) {
        let name = flag.get_id().to_string();
        if name == "help" {
            log::debug!("skipping help flag");
            continue;
        }

        log::debug!("generating flag {}", name);

        let shorts = flag.get_short_and_visible_aliases().unwrap_or_default();
        let longs = flag.get_long_and_visible_aliases().unwrap_or_default();
        let num_args = flag.get_num_args().expect("built");
        let takes_values = num_args.takes_values();

        let once = match flag.get_action() {
            ArgAction::Count | ArgAction::Append => false,
            _ => true, // NOTE: should also check `flag.overrides`, but it's private :(
        };
        let description = utils::escape_help(flag.get_help().unwrap_or_default());

        let shorts = JoinQuotes(Some('\''), shorts.iter());
        let longs = JoinQuotes(Some('\"'), longs.iter());
        let possible_values = flag.get_possible_values();
        let has_possible_values = !possible_values.is_empty();

        let is_const = !takes_values || has_possible_values;
        let rust_name = gen_rust_name(NameType::FLAG, &name, is_const);
        if !is_const {
            writeln!(
                w,
                "\
{indent}pub trait {rust_name} {{
{indent}    const ID: id::Flag = id::Flag::new(line!(), \"{name}\");
{indent}    const OBJ: Flag = Flag {{
{indent}        id: Self::ID,
{indent}        info: info::FlagInfo {{
{indent}            short: &[{shorts}],
{indent}            long: &[{longs}],
{indent}            description: \"{description}\",
{indent}        }},
{indent}        comp_options: Some(Self::comp_options),
{indent}        once: {once},
{indent}    }};

{indent}    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {{
{indent}        vec![]
{indent}    }}
{indent}}}"
            )?;
        } else {
            let comp_options = CompOptionDisplay(&possible_values);
            writeln!(
                w,
                "\
{indent}pub const {rust_name}: Flag = Flag {{
{indent}    id: id::Flag::new(line!(), \"{name}\"),
{indent}    info: info::FlagInfo {{
{indent}        short: &[{shorts}],
{indent}        long: &[{longs}],
{indent}        description: \"{description}\",
{indent}    }},
{indent}    comp_options: {comp_options},
{indent}    once: {once},
{indent}}};"
            )?;
        }
        flag_names.push((is_const, rust_name));
    }
    Ok(flag_names)
}

fn generate_subcmd_names(cmd: &Command) -> impl Iterator<Item = String> {
    utils::non_help_subcmd(cmd).map(|c| to_snake_case(c.get_name()))
}

fn generate_recur(
    level: usize,
    indent: &str,
    cmd: &Command,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let name = cmd.get_name();
    let description = utils::escape_help(cmd.get_about().unwrap_or_default());
    if level > 0 {
        writeln!(w, "{indent}pub mod {} {{", to_snake_case(cmd.get_name()))?;
    } // else: it's the first time, don't need a mod

    {
        let inner_indent = format!("    {indent}");
        let indent = if level > 0 { &inner_indent } else { indent };

        if level > 0 {
            let pre = "super::".repeat(level);
            writeln!(w, "{indent}#[allow(unused)]")?;
            writeln!(w, "{indent}use {pre}Supplements;")?;
        }
        writeln!(w, "{indent}use supplements::*;")?;

        let flags = generate_flags_in_cmd(&indent, cmd, w)?;
        let args = generate_args_in_cmd(&indent, cmd, w)?;

        let rust_name = NameType::COMMAND;

        let args = JoinQuotes(
            None,
            args.iter().map(|a| format!("<Supplements as {a}>::OBJ")),
        );
        let flags = JoinQuotes(
            None,
            flags.iter().map(|(is_const, f)| {
                if *is_const {
                    Cow::Borrowed(f)
                } else {
                    Cow::Owned(format!("<Supplements as {f}>::OBJ"))
                }
            }),
        );

        let sub_cmds: Vec<_> = generate_subcmd_names(cmd).collect();
        let sub_cmds = JoinQuotes(
            None,
            sub_cmds
                .iter()
                .map(|m| format!("{m}::{}", NameType::COMMAND)),
        );

        writeln!(
            w,
            "\
{indent}pub const {rust_name}: Command = Command {{
{indent}    id: id::Command::new(line!(), \"{name}\"),
{indent}    info: info::CommandInfo {{
{indent}        name: \"{name}\",
{indent}        description: \"{description}\",
{indent}    }},
{indent}    all_flags: &[{flags}],
{indent}    args: &[{args}],
{indent}    commands: &[{sub_cmds}],
{indent}}};"
        )?;

        for sub_cmd in utils::non_help_subcmd(cmd) {
            generate_recur(level + 1, &indent, sub_cmd, w)?;
        }
    }
    if level > 0 {
        writeln!(w, "{indent}}}")?;
    }
    Ok(())
}
