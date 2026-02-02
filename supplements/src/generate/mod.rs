use clap::{
    Command,
    builder::{ArgAction, PossibleValue},
};
use std::borrow::Cow;
use std::io::Write;

mod utils;

pub fn generate(cmd: &mut Command, w: &mut impl Write) -> std::io::Result<()> {
    cmd.build();

    generate_recur(true, "", cmd, w)
}

struct NameType(&'static str);
impl NameType {
    const FLAG: Self = NameType("Flag");
    const ARG: Self = NameType("Arg");
    const COMMAND: Self = NameType("Cmd");
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
{indent}    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {{
{indent}        vec![]
{indent}    }}
{indent}    fn id() -> id::Flag {{
{indent}        id::Flag::new(line!(), \"{name}\")
{indent}    }}
{indent}    fn generate() -> Flag {{
{indent}        Flag {{
{indent}            id: Self::id(),
{indent}            info: info::FlagInfo {{
{indent}                short: &[{shorts}],
{indent}                long: &[{longs}],
{indent}                description: \"{description}\",
{indent}            }},
{indent}            comp_options: Some(Self::comp_options),
{indent}            once: {once},
{indent}        }}
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
        flag_names.push((is_const, rust_name.clone()));
    }
    Ok(flag_names)
}

fn generate_subcmd_names(cmd: &Command) -> impl Iterator<Item = (String, String)> {
    utils::non_help_subcmd(cmd).map(|c| {
        let name = c.get_name();
        let ty_name = format!("I{}{}", NameType::COMMAND, to_pascal_case(name));
        (to_snake_case(name), ty_name)
    })
}

fn generate_recur(
    first: bool,
    indent: &str,
    cmd: &Command,
    w: &mut impl Write,
) -> std::io::Result<()> {
    let name = cmd.get_name();
    let description = cmd.get_before_help().unwrap_or_default().to_string();
    if !first {
        writeln!(w, "{indent}pub mod {} {{", to_snake_case(cmd.get_name()))?;
    } // else: it's the first time, don't need a mod

    {
        // TODO: if everything (flag, arg, command) inside `cmd` is const, `cmd` can as well be const

        let inner_indent = format!("    {indent}");
        let indent = if !first { &inner_indent } else { indent };

        writeln!(w, "{indent}use supplements::*;")?;

        let flags = generate_flags_in_cmd(&indent, cmd, w)?;

        let rust_name = NameType::COMMAND;
        writeln!(w, "{indent}pub trait {rust_name} {{")?;

        for (is_const, flag) in flags.iter() {
            if *is_const {
                continue;
            }
            writeln!(w, "{indent}    type I{flag}: {flag};")?;
        }

        let flags = flags.iter().map(|(is_const, f)| {
            if *is_const {
                Cow::Borrowed(f)
            } else {
                Cow::Owned(format!("Self::I{f}::generate()"))
            }
        });
        let flags = JoinQuotes(None, flags);

        let sub_cmds: Vec<_> = generate_subcmd_names(cmd).collect();
        for (mod_name, ty_name) in sub_cmds.iter() {
            writeln!(
                w,
                "{indent}    type {ty_name}: {mod_name}::{};",
                NameType::COMMAND
            )?;
        }
        let sub_cmds = JoinQuotes(
            None,
            sub_cmds
                .iter()
                .map(|(m, c)| format!("<Self::{c} as {m}::{}>::generate()", NameType::COMMAND)),
        );

        writeln!(
            w,
            "\
{indent}    fn id() -> id::Command {{
{indent}        id::Command::new(line!(), \"{name}\")
{indent}    }}
{indent}    fn generate() -> Command {{
{indent}        Command {{
{indent}            id: Self::id(),
{indent}            info: info::CommandInfo {{
{indent}                name: \"{name}\",
{indent}                description: \"{description}\",
{indent}            }},
{indent}            all_flags: vec![{flags}],
{indent}            args: vec![/*TODO*/],
{indent}            commands: vec![{sub_cmds}],
{indent}        }}
{indent}    }}
{indent}}}"
        )?;

        for sub_cmd in utils::non_help_subcmd(cmd) {
            generate_recur(false, &indent, sub_cmd, w)?;
        }
    }
    if !first {
        writeln!(w, "{indent}}}")?;
    }
    Ok(())
}
