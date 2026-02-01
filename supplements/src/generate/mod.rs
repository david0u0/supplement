use clap::{Command, builder::PossibleValue};
use std::borrow::Cow;
use std::io::Write;

mod utils;

pub fn generate(cmd: &mut Command, w: &mut impl Write) -> std::io::Result<()> {
    cmd.build();

    generate_recur("", cmd, w)
}

struct NameType(&'static str);
impl NameType {
    const FLAG: Self = NameType("Flag");
    const ARG: Self = NameType("Arg");
    const COMMAND: Self = NameType("Cmd");
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
    let mut ret = ty.0.to_owned();
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

struct JoinQuotes<I>(I, Option<char>);
impl<T, I> std::fmt::Display for JoinQuotes<I>
where
    T: std::fmt::Display,
    I: Iterator<Item = T> + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut first = true;
        for t in self.0.clone() {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            if let Some(quote) = self.1 {
                write!(f, "{}{}{}", quote, t, quote)?;
            } else {
                write!(f, "{t}")?;
            }
        }
        Ok(())
    }
}

struct PossibleValueDisplay<'a>(&'a PossibleValue);
impl<'a> std::fmt::Display for PossibleValueDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Completion::new(\"{}\", \"{}\")",
            self.0.get_name(),
            self.0.get_help().unwrap_or_default()
        )
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
        let once = num_args.max_values() <= 1;

        let description = utils::escape_help(flag.get_help().unwrap_or_default());

        let rust_name = gen_rust_name(NameType::FLAG, &name, !takes_values);
        let shorts = JoinQuotes(shorts.iter(), Some('\''));
        let longs = JoinQuotes(longs.iter(), Some('\"'));

        if takes_values {
            let possible_values = flag.get_possible_values();
            let possible_values = JoinQuotes(
                possible_values.iter().map(|p| PossibleValueDisplay(p)),
                None,
            );

            flag_names.push((false, rust_name.clone()));
            writeln!(
                w,
                "\
{indent}pub trait {rust_name} {{
{indent}    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {{
{indent}        vec![{possible_values}]
{indent}    }}
{indent}    fn id() -> id::Flag {{
{indent}        id::Flag::new(line!(), \"{name}\")
{indent}    }}
{indent}    fn generate() -> Flag {{
{indent}        Flag {{
{indent}            id: Self::id(),
{indent}            info: FlagInfo {{
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
            flag_names.push((true, rust_name.clone()));
            writeln!(
                w,
                "\
{indent}pub const {rust_name}: Flag = Flag {{
{indent}    id: id::Flag::new(line!(), \"{name}\"),
{indent}    info: FlagInfo {{
{indent}        short: &[{shorts}],
{indent}        long: &[{longs}],
{indent}        description: \"{description}\",
{indent}    }},
{indent}    comp_options: None,
{indent}    once: {once},
{indent}}};"
            )?;
        }
    }
    Ok(flag_names)
}
fn generate_recur(indent: &str, cmd: &Command, w: &mut impl Write) -> std::io::Result<()> {
    let name = cmd.get_name();
    let description = cmd.get_before_help().unwrap_or_default().to_string();
    if indent != "" {
        writeln!(w, "{indent}pub mod {} {{", to_snake_case(cmd.get_name()))?;
    } // else: it's the first time, don't need a mod

    {
        let inner_indent = format!("    {indent}");
        let indent = if indent != "" { &inner_indent } else { indent };

        writeln!(w, "{indent}use supplements::*;")?;

        let flags = generate_flags_in_cmd(&indent, cmd, w)?;

        let rust_name = NameType::COMMAND.0;
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
        let flags = JoinQuotes(flags, None);

        writeln!(
            w,
            "\
{indent}    fn id() -> id::Command {{
{indent}        id::Command::new(line!(), \"{name}\")
{indent}    }}
{indent}    fn generate() -> Command {{
{indent}        Command {{
{indent}            id: Self::id(),
{indent}            all_flags: vec![{flags}],
{indent}            info: CommandInfo {{
{indent}                name: \"{name}\",
{indent}                description: \"{description}\",
{indent}            }},
{indent}            args: vec![/*TODO*/],
{indent}            commands: vec![/*TODO*/],
{indent}        }}
{indent}    }}
{indent}}}"
        )?;
    }
    if indent != "" {
        writeln!(w, "{indent}}}")?;
    }
    Ok(())
}
