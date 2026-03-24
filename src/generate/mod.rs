use crate::error::GenerateError;
use std::io::Write;

mod abstraction;
mod config;
mod utils;
use abstraction::{Arg, ArgAction, ClapCommand, Command, CommandMut, PossibleValue};
pub use config::Config;
use utils::{Join, ctx_func, gen_enum_name, gen_rust_name, to_screaming_snake_case, to_snake_case};

#[derive(Clone)]
pub(crate) struct Trace {
    cmd_id: String,
}

#[cfg(doc)]
use crate::clap;
#[cfg(doc)]
use crate::core;

/// Generate the scaffold for your completion based on [`clap::Command`] object.
/// After importing the generated code, you can call [`core::Command::supplement`] to get the completion result.
///
/// ```no_run
/// # #[cfg(feature = "clap-3")]
/// # use clap3 as clap;
/// # #[cfg(feature = "clap-4")]
/// # use clap4 as clap;
///
/// use supplement::generate::generate;
/// let config = Default::default();
/// let mut cmd = clap::Command::new("git");
///
/// // Print the generated code to stdout
/// generate(&mut cmd, config, &mut std::io::stdout()).unwrap();
/// ```
///
/// full example can be found in `supplement-example/src/main.rs`
pub fn generate(
    cmd: ClapCommand<'_>,
    mut config: Config,
    w: &mut impl Write,
) -> Result<(), GenerateError> {
    let mut cmd = CommandMut(cmd);
    cmd.build();
    let cmd = cmd.into_const();

    writeln!(w, "type GlobalID = ID;")?;
    generate_recur(&[], "", &mut config, &cmd, &[], w)?;
    config.check_unprocessed_config()
}

struct CmdUnit {
    mod_name: String,
    enum_name: Option<String>,
}
struct ValUnit {
    rust_name: String,
    enum_name: Option<String>,
    ctx_ty: Option<ValType>,
}

#[derive(Clone)]
struct GlobalFlag {
    level: usize,
    id: String,
    ignored: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct NameType(&'static str);
impl NameType {
    const COMMAND: Self = NameType("CMD");
    const EXTERNAL: Self = NameType("External");
    const VAL: Self = NameType("Val");
}
impl std::fmt::Display for NameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn format_possible_values(values: &[PossibleValue]) -> String {
    if values.is_empty() {
        return "CowOwned::Borrow(&[])".to_string();
    }
    let inner = Join(values.iter().map(|p| {
        let name = p.get_name();
        let help = p.get_help().unwrap_or_default();
        format!("(\"{name}\", \"{help}\")")
    }));
    format!("CowOwned::Borrow(&[{inner}])")
}

macro_rules! handle_custom {
    ($is_static:ident, $force_custom:expr, $name:expr) => {
        let force_custom = $force_custom;
        if force_custom {
            if !$is_static {
                return Err(GenerateError::AlreadyCustom($name));
            }
            $is_static = false;
        }
    };
}

#[derive(Clone, Copy)]
enum ValType {
    No,
    Single,
    Multi,
}
impl ValType {
    fn get_rust_type(self) -> &'static str {
        match self {
            ValType::No => "u32",
            ValType::Single => "Option<&'a str>",
            ValType::Multi => "&'a [String]",
        }
    }
}

struct FlagDisplayHelper<'a> {
    id_name: &'a str,
    ty: ValType,
    flag: Arg<'a>,
    strict: bool,
    force_custom: bool,

    prev: &'a [Trace],
}
impl FlagDisplayHelper<'_> {
    fn is_static(&self) -> Result<bool, GenerateError> {
        let mut is_static = match self.ty {
            ValType::No => {
                if self.force_custom {
                    return Err(GenerateError::CustomWithoutValue(
                        self.flag.get_id().to_string(),
                    ));
                }
                true
            }
            _ => !self.flag.get_possible_values().is_empty(),
        };
        handle_custom!(is_static, self.force_custom, self.flag.get_id().to_string());
        Ok(is_static)
    }
    fn id_line_str(&self) -> String {
        let id_name = self.id_name;
        let id_type = match self.ty {
            ValType::No => "NoVal",
            ValType::Single => "SingleVal",
            ValType::Multi => "MultiVal",
        };

        format!("const {id_name}: id::{id_type} = id::{id_type}::new(line!());")
    }
    fn type_str(&self) -> Result<String, GenerateError> {
        let id_name = self.id_name;
        let id = self.flag.get_id().to_string();
        let s = match self.ty {
            ValType::No => {
                format!("flag_type::Type::new_bool({id_name})")
            }
            _ => {
                let is_static = self.is_static()?;
                let id_value = if is_static {
                    "None".to_string()
                } else {
                    format!(
                        "Some({})",
                        utils::get_id_value(self.prev, NameType::VAL, &id)
                    )
                };

                let complete_with_equal = utils::compute_flag_equal(self.flag, self.strict)
                    .map_err(|msg| GenerateError::Strict { id, msg })?;
                let possible_values = self.flag.get_possible_values();
                let possible_values = format_possible_values(&possible_values);
                format!(
                    "flag_type::Type::new_valued({id_value}, {id_name}.into(), {complete_with_equal}, {possible_values})"
                )
            }
        };
        Ok(s)
    }
}

fn generate_args_in_cmd(
    indent: &str,
    cmd: &Command<'_>,
    config: &mut Config,
    prev: &[Trace],
    w: &mut impl Write,
) -> Result<Vec<ValUnit>, GenerateError> {
    let mut args_names = vec![];

    let ext_sub = if cmd.is_allow_external_subcommands_set() {
        log::debug!("generating external subcommand");
        let name = gen_rust_name(NameType::EXTERNAL, "");
        Some((
            "@ext".to_string(),
            name,
            usize::MAX,
            vec![],
            NameType::EXTERNAL,
        ))
    } else {
        None
    };
    let args = utils::args(cmd).map(|arg| {
        let name = arg.get_id().to_string();

        log::debug!("generating arg {}", name);

        let max_values = arg.get_max_num_args();
        let rust_name = gen_rust_name(NameType::VAL, &name);

        (
            name,
            rust_name,
            max_values,
            arg.get_possible_values(),
            NameType::VAL,
        )
    });
    let args = args.chain(ext_sub);

    for (name, rust_name, max_values, possible_values, name_type) in args {
        let id_name = to_screaming_snake_case(&format!("id_{rust_name}"));
        let (id_type, ty) = if max_values == 1 {
            ("id::SingleVal", ValType::Single)
        } else {
            ("id::MultiVal", ValType::Multi)
        };
        let force_custom = config.is_custom(prev, &name);
        let mut is_static = !possible_values.is_empty();
        handle_custom!(is_static, force_custom, name);

        let id_value = if is_static {
            "None".to_string()
        } else {
            format!("Some({})", utils::get_id_value(prev, name_type, &name))
        };
        let possible_values = format_possible_values(&possible_values);

        writeln!(
            w,
            "\
{indent}const {id_name}: {id_type} = {id_type}::new(line!());
{indent}const {rust_name}: Arg<GlobalID> = Arg {{
{indent}    id: {id_value},
{indent}    seen_id: {id_name}.into(),
{indent}    max_values: {max_values},
{indent}    possible_values: {possible_values},
{indent}}};"
        )?;

        let enum_name = gen_enum_name(name_type, &name);
        let enum_name = if is_static { None } else { Some(enum_name) };
        args_names.push(ValUnit {
            rust_name,
            enum_name,
            ctx_ty: Some(ty),
        });
    }

    Ok(args_names)
}

fn generate_flags_in_cmd(
    prev: &[Trace],
    indent: &str,
    config: &mut Config,
    cmd: &Command<'_>,
    global_flags: &mut Vec<GlobalFlag>,
    w: &mut impl Write,
) -> Result<Vec<ValUnit>, GenerateError> {
    let mut flag_names = vec![];

    for flag in utils::flags(cmd) {
        let name = flag.get_id().to_string();

        let ignored = config.is_ignored(prev, &name);

        let takes_values = flag.takes_values();
        let rust_name = gen_rust_name(NameType::VAL, &name);
        if flag.is_global_set() {
            let level = prev.len();
            if let Some(prev_flag) = global_flags.iter().find(|f| f.id == name) {
                log::info!("get existing global flag {name}");
                if !prev_flag.ignored && !ignored {
                    let mut super_name = "super::".repeat(level - prev_flag.level);
                    super_name += &rust_name;
                    flag_names.push(ValUnit {
                        rust_name: super_name,
                        enum_name: None,
                        ctx_ty: None,
                    });
                }
                continue;
            } else {
                log::info!("get new global flag {name}");
                global_flags.push(GlobalFlag {
                    level,
                    ignored,
                    id: name.clone(),
                });
            }
        }
        if ignored {
            continue;
        }

        log::debug!("generating flag {}", name);

        let shorts = flag.get_short_and_visible_aliases().unwrap_or_default();
        let longs = flag.get_long_and_visible_aliases().unwrap_or_default();

        let (once, ty) = match flag.get_action() {
            ArgAction::Count => (false, ValType::No),
            ArgAction::Append => (false, ValType::Multi),
            _ => {
                let once = !flag.is_global_set();
                if takes_values {
                    (once, ValType::Single)
                } else {
                    // TODO: for `once`, should also check override self
                    (once, ValType::No)
                }
            }
        };
        let description = utils::escape_help(&flag.get_help());

        let shorts = Join(shorts.iter().map(|s| format!("'{s}'")));
        let longs = Join(longs.iter().map(|s| format!("\"{s}\"")));
        let id_name = to_screaming_snake_case(&format!("id_{rust_name}"));
        let flag_display_helper = FlagDisplayHelper {
            ty,
            flag,
            prev,
            id_name: &id_name,
            strict: config.is_strict(),
            force_custom: config.is_custom(prev, &name),
        };

        let is_static = flag_display_helper.is_static()?;
        let enum_name = gen_enum_name(NameType::VAL, &name);
        let enum_name = if is_static { None } else { Some(enum_name) };

        let id_line = flag_display_helper.id_line_str();
        let type_str = flag_display_helper.type_str()?;

        writeln!(
            w,
            "\
{indent}{id_line}
{indent}const {rust_name}: Flag<GlobalID> = Flag {{
{indent}    short: CowSlice::Borrow(&[{shorts}]),
{indent}    long: CowOwned::Borrow(&[{longs}]),
{indent}    description: Cow::Borrowed(\"{description}\"),
{indent}    once: {once},
{indent}    ty: {type_str},
{indent}}};"
        )?;
        flag_names.push(ValUnit {
            rust_name,
            enum_name,
            ctx_ty: Some(ty),
        });
    }
    Ok(flag_names)
}

fn generate_mod_name(name: &str) -> String {
    to_snake_case(name)
}

fn check_cmd_not_empty(sub_cmds: &[CmdUnit], flags: &[ValUnit], args: &[ValUnit]) -> bool {
    if sub_cmds.iter().any(|c| c.enum_name.is_some()) {
        return true;
    }
    flags
        .iter()
        .chain(args.iter())
        .any(|v| v.enum_name.is_some())
}

fn generate_recur(
    prev: &[Trace],
    indent: &str,
    config: &mut Config,
    cmd: &Command<'_>,
    global_flags: &[GlobalFlag],
    w: &mut impl Write,
) -> Result<bool, GenerateError> {
    let mut global_flags = global_flags.to_vec();
    let name = cmd.get_name();
    let description = utils::escape_help(&cmd.get_about().unwrap_or_default());
    let level = prev.len();
    let cmd_not_empty: bool;
    {
        let inner_indent = format!("    {indent}");
        let indent = if level > 0 { &inner_indent } else { indent };

        if level > 0 {
            let pre = "super::".repeat(level);
            writeln!(w, "{indent}use {pre}GlobalID as GlobalID;")?;
        }
        writeln!(w, "{indent}use supplement::gen_prelude::*;\n")?;

        let flags = generate_flags_in_cmd(prev, indent, config, cmd, &mut global_flags, w)?;
        let args = generate_args_in_cmd(indent, cmd, config, prev, w)?;

        let mut sub_cmds: Vec<CmdUnit> = vec![];
        for sub_cmd in utils::non_help_subcmd(cmd) {
            let cmd_id = sub_cmd.get_name().to_string();
            if config.is_ignored(prev, &cmd_id) {
                continue;
            }

            let mod_name = generate_mod_name(&cmd_id);
            writeln!(w, "{indent}pub mod {mod_name} {{")?;
            let mut prev = prev.to_vec();
            prev.push(Trace {
                cmd_id: cmd_id.clone(),
            });
            let cur_cmd_not_empty =
                generate_recur(&prev, indent, config, &sub_cmd, &global_flags, w)?;
            writeln!(w, "{indent}}}")?;

            let enum_name = if cur_cmd_not_empty {
                Some(gen_enum_name(NameType::COMMAND, &cmd_id))
            } else {
                None
            };
            sub_cmds.push(CmdUnit {
                mod_name,
                enum_name,
            });
        }

        let cmd_name = NameType::COMMAND;
        cmd_not_empty = check_cmd_not_empty(&sub_cmds, &flags, &args);

        if cmd_not_empty {
            writeln!(w, "{indent}#[derive(Clone, Copy, PartialEq, Eq, Debug)]")?;
            writeln!(w, "{indent}pub enum ID<H = ()> {{")?;
            for val in args.iter().chain(flags.iter()) {
                if let Some(enum_name) = val.enum_name.as_ref() {
                    writeln!(w, "{indent}    {enum_name}(H),")?;
                }
            }
            for cmd in sub_cmds.iter() {
                if let Some(enum_name) = cmd.enum_name.as_ref() {
                    let mod_name = &cmd.mod_name;
                    writeln!(w, "{indent}    {enum_name}(H, {mod_name}::ID<H>),")?;
                }
            }
            writeln!(w, "{indent}}}")?;

            writeln!(w, "{indent}#[allow(dead_code)]")?;
            writeln!(w, "{indent}impl<'a> ID<&'a Seen> {{")?;
            for val in args.iter().chain(flags.iter()) {
                if let Some(ty) = val.ctx_ty.as_ref() {
                    let ctx_func = ctx_func(&val.rust_name, *ty);
                    let ty = ty.get_rust_type();
                    let name = to_snake_case(&val.rust_name);
                    writeln!(w, "{indent}    pub fn {name}(&self) -> {ty} {{")?;

                    writeln!(w, "{indent}        {ctx_func}")?;
                    writeln!(w, "{indent}    }}")?;
                }
            }
            writeln!(w, "{indent}}}")?;

            writeln!(w, "{indent}#[allow(dead_code)]")?;
            writeln!(w, "{indent}impl<H> ID<H> {{")?;
            let inner = format!("{indent}    ");
            write_with_ctx(w, &inner, &sub_cmds, args.iter().chain(flags.iter()))?;
            write_get_ctx(w, &inner, &sub_cmds, args.iter().chain(flags.iter()))?;
            writeln!(w, "{indent}}}")?;
        }

        let args = Join(args.iter().map(|x| &x.rust_name));
        let flags = Join(flags.iter().map(|x| &x.rust_name));
        let sub_cmds = Join(
            sub_cmds
                .iter()
                .map(|x| format!("{}::{}", x.mod_name, cmd_name)),
        );
        let scope = if level == 0 { "" } else { "(super)" };

        writeln!(
            w,
            "\
{indent}pub{scope} const {cmd_name}: Command<GlobalID> = Command {{
{indent}    name: Cow::Borrowed(\"{name}\"),
{indent}    description: Cow::Borrowed(\"{description}\"),
{indent}    all_flags: CowSlice::Borrow(&[{flags}]),
{indent}    args: CowSlice::Borrow(&[{args}]),
{indent}    commands: CowSlice::Borrow(&[{sub_cmds}]),
{indent}}};"
        )?;
    }
    Ok(cmd_not_empty)
}

fn write_with_ctx<'a>(
    w: &mut impl Write,
    indent: &str,
    sub_cmds: &[CmdUnit],
    vals: impl Iterator<Item = &'a ValUnit>,
) -> Result<(), std::io::Error> {
    writeln!(
        w,
        "{indent}pub fn with_seen(self, h: &Seen) -> ID<&Seen> {{"
    )?;
    writeln!(w, "{indent}    match self {{")?;
    for val in vals {
        if let Some(enum_name) = val.enum_name.as_ref() {
            writeln!(
                w,
                "{indent}        ID::{enum_name}(..) => ID::{enum_name}(h),"
            )?;
        }
    }
    for cmd in sub_cmds.iter() {
        if let Some(enum_name) = cmd.enum_name.as_ref() {
            writeln!(
                w,
                "{indent}        ID::{enum_name}(_, id) => ID::{enum_name}(h, id.with_seen(h)),"
            )?;
        }
    }
    writeln!(w, "{indent}    }}")?;
    writeln!(w, "{indent}}}")?;
    Ok(())
}

fn write_get_ctx<'a>(
    w: &mut impl Write,
    indent: &str,
    sub_cmds: &[CmdUnit],
    vals: impl Iterator<Item = &'a ValUnit>,
) -> Result<(), std::io::Error> {
    writeln!(w, "{indent}fn ctx(self) -> H {{")?;
    writeln!(w, "{indent}    match self {{")?;
    for val in vals {
        if let Some(enum_name) = val.enum_name.as_ref() {
            writeln!(w, "{indent}        ID::{enum_name}(c) => c,")?;
        }
    }
    for cmd in sub_cmds.iter() {
        if let Some(enum_name) = cmd.enum_name.as_ref() {
            writeln!(w, "{indent}        ID::{enum_name}(c, _) => c,")?;
        }
    }
    writeln!(w, "{indent}    }}")?;
    writeln!(w, "{indent}}}")?;
    Ok(())
}
