use std::iter::{Peekable, once};
use std::str::Chars;

pub struct FlagInfo {
    pub short: Option<char>,
    pub long: &'static str,
    pub description: &'static str,
}
pub struct CommandInfo {
    pub name: &'static str,
    pub description: &'static str,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SupplementID(u32, &'static str);

#[derive(Debug)]
pub enum SingleHistory {
    Flag { id: SupplementID, value: String },
    Command(SupplementID),
    Arg { id: SupplementID, value: String },
}
#[derive(Default, Debug)]
pub struct History(Vec<SingleHistory>);
impl History {
    pub fn push_arg(&mut self, id: SupplementID, value: String) {
        self.0.push(SingleHistory::Arg { id, value });
    }
    pub fn push_flag(&mut self, id: SupplementID, value: String) {
        self.0.push(SingleHistory::Flag { id, value });
    }
    pub fn push_pure_flag(&mut self, id: SupplementID) {
        self.0.push(SingleHistory::Flag {
            id,
            value: String::new(),
        });
    }
    pub fn push_command(&mut self, id: SupplementID) {
        self.0.push(SingleHistory::Command(id))
    }

    pub fn find(&self, id: SupplementID) -> Option<&SingleHistory> {
        self.0.iter().find(|h| {
            let cur_id = match h {
                SingleHistory::Arg { id, .. } => id,
                SingleHistory::Flag { id, .. } => id,
                SingleHistory::Command(id) => id,
            };
            *cur_id == id
        })
    }
}

#[derive(Debug)]
pub struct CompResult {
    value: String,
    description: String,
}
impl CompResult {
    pub fn new(value: &str, description: &str) -> Self {
        CompResult {
            value: value.to_owned(),
            description: description.to_owned(),
        }
    }
}

pub struct Flag {
    id: SupplementID,
    info: FlagInfo,
    comp_options: Option<fn(&History, &str) -> Vec<CompResult>>,
    once: bool,
}
pub struct Arg {
    id: SupplementID,
    comp_options: fn(&History, &str) -> Vec<CompResult>,
    // TODO: infinite args?
}
pub struct Command {
    id: SupplementID,
    info: CommandInfo,
    true_flags: Vec<Flag>,
    args: Vec<Arg>,
    commands: Vec<Command>,
}

#[derive(Debug)]
enum ParsedFlag<'a> {
    Empty,
    SingleDash,
    DoubleDash,
    Short {
        body: char,
        equal: Option<&'a str>,
    },
    MultiShort {
        body: Chars<'a>,
        equal: Option<&'a str>,
    },
    Long {
        body: &'a str,
        equal: Option<&'a str>,
    },
    NotFlag,
    Error,
}
impl<'a> ParsedFlag<'a> {
    fn is_valid_flag(s: &str, allow_single_dash: bool) -> bool {
        let s = &s[1..];
        let mut last_is_dash = false;
        let mut equal_appeard = false;
        for ch in s.chars() {
            if ch == '-' {
                if !allow_single_dash {
                    log::warn!("a dash appears when none is allowed");
                    return false;
                }
                if last_is_dash {
                    log::warn!("consecutive dashes");
                    return false;
                }
                last_is_dash = true;
                continue;
            }
            last_is_dash = false;

            if ch == '=' {
                if equal_appeard {
                    log::warn!("more than one `=`");
                    return false;
                }
                equal_appeard = true;
                continue;
            }

            match ch {
                'a'..='z' => (),
                'A'..='Z' => (),
                '0'..='9' => (),
                '_' => (),
                _ => {
                    log::warn!("wrong char {:?}", ch);
                    return false;
                }
            }
        }
        true
    }

    fn new(s: &'a str) -> Self {
        if s.is_empty() {
            return Self::Empty;
        }
        if !s.starts_with('-') {
            return Self::NotFlag;
        }
        match s.chars().nth(1) {
            None => Self::SingleDash,
            Some('-') => {
                if s.len() == 2 {
                    return Self::DoubleDash;
                }
                if !Self::is_valid_flag(s, true) {
                    return Self::Error;
                }
                let mut flag_part = &s[2..];
                if flag_part.len() == 1 {
                    return Self::Error; // "--a" is not allowed
                }

                let equal = if let Some(equal_pos) = flag_part.chars().position(|c| c == '=') {
                    let equal = &flag_part[equal_pos + 1..];
                    flag_part = &flag_part[..equal_pos];
                    Some(equal)
                } else {
                    None
                };

                Self::Long {
                    body: flag_part,
                    equal,
                }
            }
            _ => {
                let flag_part = &s[1..];
                if !Self::is_valid_flag(s, false) {
                    return Self::Error;
                }
                if flag_part.len() == 1 {
                    Self::Short {
                        body: flag_part.chars().nth(0).unwrap(),
                        equal: None,
                    }
                } else {
                    if let Some(equal_pos) = flag_part.chars().position(|c| c == '=') {
                        let equal = Some(&flag_part[equal_pos + 1..]);
                        if equal_pos == 1 {
                            Self::Short {
                                body: flag_part.chars().nth(0).unwrap(),
                                equal,
                            }
                        } else {
                            Self::MultiShort {
                                body: flag_part[..equal_pos].chars(),
                                equal,
                            }
                        }
                    } else {
                        Self::MultiShort {
                            body: flag_part.chars(),
                            equal: None,
                        }
                    }
                }
            }
        }
    }
}

impl Arg {
    fn supplement(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Option<Vec<CompResult>> {
        let value = args.next().unwrap();
        if args.peek().is_none() {
            return Some((self.comp_options)(history, &value));
        }

        history.push_arg(self.id, value);
        None
    }
}

impl Flag {
    fn supplement(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Option<Vec<CompResult>> {
        let Some(comp_options) = self.comp_options else {
            history.push_pure_flag(self.id);
            return None;
        };

        let arg = args.next().unwrap();
        if args.peek().is_none() {
            return Some(comp_options(&history, &arg));
        }

        history.push_flag(self.id, arg);
        None
    }
}

impl Command {
    pub fn supplement(
        &self,
        args: impl Iterator<Item = String>,
        last_is_empty: bool,
    ) -> Vec<CompResult> {
        let mut history = History::default();
        self.supplement_with_history(&mut history, args, last_is_empty)
    }

    pub fn supplement_with_history(
        &self,
        history: &mut History,
        args: impl Iterator<Item = String>,
        last_is_empty: bool,
    ) -> Vec<CompResult> {
        let last_arg = if last_is_empty {
            Some(String::new())
        } else {
            None
        };

        let mut args = args.chain(last_arg.into_iter()).peekable();
        if args.peek().is_none() {
            panic!();
        }

        self.supplement_recur(true, history, &mut args)
    }

    fn flags(&self, history: &History) -> impl Iterator<Item = &Flag> {
        self.true_flags.iter().filter(|f| {
            if !f.once {
                true
            } else {
                let exists = history.find(f.id).is_some();
                if exists {
                    log::debug!("flag {:?} already exists", f.id);
                }
                !exists
            }
        })
    }

    fn supplement_recur(
        &self,
        is_first: bool,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
    ) -> Vec<CompResult> {
        let arg = args.next().unwrap();
        if is_first {
            history.push_command(self.id);
        }

        if args.peek().is_none() {
            return self.supplement_last(history, arg);
        }

        macro_rules! handle_flag {
            ($flag:expr, $equal:expr, $history:expr) => {
                if let Some(equal) = $equal {
                    if $flag.comp_options.is_none() {
                        unimplemented!("error: value for a boolean flag");
                    }
                    $history.push_flag($flag.id, equal.to_string());
                } else {
                    let res = $flag.supplement($history, args);
                    if let Some(res) = res {
                        return res;
                    }
                }
            }
        }

        match ParsedFlag::new(&arg) {
            ParsedFlag::SingleDash | ParsedFlag::DoubleDash => {
                return self.supplement_args(history, args, arg);
            }
            ParsedFlag::NotFlag => {
                let command = self.commands.iter().find(|c| arg == c.info.name);
                return match command {
                    Some(command) => command.supplement_recur(true, history, args),
                    None => self.supplement_args(history, args, arg),
                };
            }
            ParsedFlag::Long { body, equal } => {
                let flag = self.flags(history).find(|f| f.info.long == body);
                let Some(flag) = flag else { unimplemented!() };
                handle_flag!(flag, equal, history);
            }
            ParsedFlag::Short { body, equal } => {
                let flag = self.flags(history).find(|f| f.info.short == Some(body));
                let Some(flag) = flag else { unimplemented!() };
                handle_flag!(flag, equal, history);
            }
            ParsedFlag::MultiShort { body, equal } => {
                let mut body = body.peekable();
                loop {
                    let Some(ch) = body.next() else {
                        break;
                    };
                    let is_last = body.peek().is_none();
                    let flag = self.flags(history).find(|f| f.info.short == Some(ch));
                    let Some(flag) = flag else { unimplemented!() };

                    if is_last {
                        handle_flag!(flag, equal, history);
                    } else {
                        if flag.comp_options.is_some() {
                            if equal.is_some() {
                                // e.g. git commit -ma=abcd
                                unimplemented!();
                            }
                            // e.g. git commit -mabcde
                            history.push_flag(flag.id, body.collect());
                            break;
                        }

                        history.push_pure_flag(flag.id);
                    }
                }
            }
            ParsedFlag::Error | ParsedFlag::Empty => {
                unimplemented!()
            }
        }

        self.supplement_recur(false, history, args)
    }

    fn supplement_last(&self, history: &mut History, arg: String) -> Vec<CompResult> {
        let all_long_flags = self.flags(history).map(|f| CompResult {
            value: format!("--{}", f.info.long),
            description: f.info.description.to_string(),
        });

        match ParsedFlag::new(&arg) {
            ParsedFlag::Empty => {
                let cmd_iter = self.commands.iter().map(|c| CompResult {
                    value: c.info.name.to_string(),
                    description: c.info.description.to_string(),
                });
                let arg_comp = if let Some(arg_obj) = self.args.first() {
                    (arg_obj.comp_options)(history, &arg)
                } else {
                    vec![]
                };
                return cmd_iter.chain(arg_comp.into_iter()).collect();
            }
            ParsedFlag::DoubleDash => {
                return all_long_flags.collect();
            }
            ParsedFlag::SingleDash => {
                let iter = self.flags(history).filter_map(|f| {
                    if let Some(c) = f.info.short {
                        Some(CompResult {
                            value: format!("-{}", c),
                            description: f.info.description.to_string(),
                        })
                    } else {
                        None
                    }
                });
                let iter = iter.chain(all_long_flags);
                return iter.collect();
            }
            _ => unimplemented!(),
        }
    }
    fn supplement_args(
        &self,
        history: &mut History,
        args: &mut Peekable<impl Iterator<Item = String>>,
        arg: String,
    ) -> Vec<CompResult> {
        let mut args = args.chain(once(arg)).peekable();
        for arg_obj in self.args.iter() {
            let res = arg_obj.supplement(history, &mut args);
            if let Some(res) = res {
                return res;
            }
        }

        panic!("too many args");
    }
}

mod def {
    use super::*;

    pub const C_FLAG: Flag = Flag {
        id: SupplementID(line!(), "long-c"),
        info: FlagInfo {
            short: Some('c'),
            long: "long-c",
            description: "test description for flag C",
        },
        comp_options: None,
        once: true,
    };
    pub trait BFlag {
        fn comp_options(_history: &History, _arg: &str) -> Vec<CompResult> {
            vec![
                CompResult::new("flag-option1", "description of option1"),
                CompResult::new("flag-option2", "description of option2"),
            ]
        }
        fn id() -> SupplementID {
            SupplementID(line!(), "long-b")
        }
        fn generate() -> Flag {
            Flag {
                id: Self::id(),
                info: FlagInfo {
                    short: Some('b'),
                    long: "long-b",
                    description: "test description for flag B",
                },
                comp_options: Some(Self::comp_options),
                once: true,
            }
        }
    }

    pub trait AArg {
        fn comp_options(_history: &History, _arg: &str) -> Vec<CompResult> {
            vec![]
        }
        fn id() -> SupplementID {
            SupplementID(line!(), "")
        }
        fn generate() -> Arg {
            Arg {
                id: Self::id(),
                comp_options: Self::comp_options,
            }
        }
    }

    pub trait Root {
        type B: BFlag;
        type Sub: SubCommand;

        fn id() -> SupplementID {
            SupplementID(line!(), "root")
        }
        fn generate() -> Command {
            Command {
                id: Self::id(),
                true_flags: vec![Self::B::generate(), C_FLAG],
                info: CommandInfo {
                    name: "root",
                    description: "",
                },
                args: vec![],
                commands: vec![Self::Sub::generate()],
            }
        }
    }

    pub trait SubCommand {
        type B: BFlag;
        type A: AArg;
        fn id() -> SupplementID {
            SupplementID(line!(), "sub")
        }
        fn generate() -> Command {
            Command {
                id: Self::id(),
                true_flags: vec![Self::B::generate()],
                info: CommandInfo {
                    name: "sub",
                    description: "test sub description",
                },
                args: vec![Self::A::generate()],
                commands: vec![],
            }
        }
    }
}

mod my_impl {
    use super::*;

    pub struct AArg;
    impl def::AArg for AArg {
        fn comp_options(_history: &History, _arg: &str) -> Vec<CompResult> {
            vec![
                CompResult::new("arg-option1", "description of option1"),
                CompResult::new("arg-option2", "description of option2"),
            ]
        }
    }

    pub struct BFlag;
    impl def::BFlag for BFlag {}

    pub struct Root;
    impl def::Root for Root {
        type B = BFlag;
        type Sub = SubCommand;
    }

    pub struct SubCommand;
    impl def::SubCommand for SubCommand {
        type B = BFlag;
        type A = AArg;
    }
}

fn do_it(args: &str, last_is_empty: bool) {
    use def::Root;
    println!("args = {:?}", args);
    let args = args.split(' ').map(|s| s.to_owned());
    let mut history = History::default();
    let res = my_impl::Root::generate().supplement_with_history(&mut history, args, last_is_empty);
    println!("comp res = {:?}", res);
    println!("history = {:?}\n", history);
}

fn main() {
    env_logger::init();

    do_it("--long-b=option sub", true);
    do_it("-b=option sub", true);
    do_it("-cb=abcd sub", true);
    do_it("-cb abcd sub", true);
    do_it("-cbabcd sub", true);

    do_it("-", false);
    do_it("-b a -", false); // test "once"
    do_it("--", false);
    do_it("sub -", false);
    do_it("sub --", false);
    do_it("-b", true);
    do_it("sub -bmy_option", true);

    do_it("-bmy_option sub", true);

    do_it("-cbmy_option sub", true);
}
