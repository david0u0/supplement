use history::*;
use supplements::*;

mod def {
    use super::*;
    use supplements::info::*;

    pub const C_FLAG: Flag = Flag {
        id: id::Flag::new(line!(), "long-c"),
        info: FlagInfo {
            short: &['c'],
            long: &["long-c", "long-c-2"],
            description: "test description for flag C",
        },
        comp_options: None,
        once: true,
    };
    pub trait BFlag {
        fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
            vec![
                Completion::new("flag-option1", "description of option1"),
                Completion::new("flag-option2", "description of option2"),
            ]
        }
        fn id() -> id::Flag {
            id::Flag::new(line!(), "long-b")
        }
        fn generate() -> Flag {
            Flag {
                id: Self::id(),
                info: FlagInfo {
                    short: &['b', 'x'],
                    long: &["long-b"],
                    description: "test description for flag B",
                },
                comp_options: Some(Self::comp_options),
                once: true,
            }
        }
    }

    pub trait AArg {
        fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
            vec![]
        }
        fn id() -> id::Arg {
            id::Arg::new(line!(), "aarg")
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

        fn id() -> id::Command {
            id::Command::new(line!(), "root")
        }
        fn generate() -> Command {
            Command {
                id: Self::id(),
                all_flags: vec![Self::B::generate(), C_FLAG],
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
        type A2: AArg;
        fn id() -> id::Command {
            id::Command::new(line!(), "sub")
        }
        fn generate() -> Command {
            Command {
                id: Self::id(),
                all_flags: vec![Self::B::generate()],
                info: CommandInfo {
                    name: "sub",
                    description: "test sub description",
                },
                args: vec![Self::A::generate(), Self::A2::generate()],
                commands: vec![],
            }
        }
    }
}

mod my_impl {
    use super::*;

    pub struct Dummy;

    impl def::AArg for Dummy {
        fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
            vec![
                Completion::new("arg-option1", "description of option1"),
                Completion::new("arg-option2", "description of option2"),
            ]
        }
    }

    impl def::BFlag for Dummy {}

    impl def::Root for Dummy {
        type B = Dummy;
        type Sub = Dummy;
    }

    impl def::SubCommand for Dummy {
        type B = Dummy;
        type A = Dummy;
        type A2 = Dummy;
    }
}
use my_impl::Dummy;

fn run(args: &str, last_is_empty: bool) -> (History, Vec<Completion>) {
    let _ = env_logger::try_init();

    let args = args.split(' ').map(|s| s.to_owned());
    let args = std::iter::once("whatever".to_owned()).chain(args);
    let mut history = History::default();
    let res =
        <Dummy as def::Root>::generate().supplement_with_history(&mut history, args, last_is_empty);
    (history, res.unwrap())
}
fn map_comp_values(arr: &[Completion]) -> Vec<&str> {
    let mut v: Vec<_> = arr.iter().map(|c| &*c.value).collect();
    v.sort();
    v
}

macro_rules! b_flag {
    ($name:ident) => {
        SingleHistory::Flag(SingleHistoryFlag {
            id: def::$name.id,
            value: String::new(),
        })
    };
}
macro_rules! flag {
    ($name:ident, $value:expr) => {
        SingleHistory::Flag(SingleHistoryFlag {
            id: <Dummy as def::$name>::id(),
            value: $value.to_owned(),
        })
    };
}
macro_rules! arg {
    ($name:ident, $value:expr) => {
        SingleHistory::Arg(SingleHistoryArg {
            id: <Dummy as def::$name>::id(),
            value: $value.to_owned(),
        })
    };
}
macro_rules! cmd {
    ($name:ident) => {
        SingleHistory::Command(SingleHistoryCommand(<Dummy as def::$name>::id()))
    };
}

#[test]
fn test_args_last() {
    let (h, r) = run("sub a1", true);
    assert_eq!(r, <Dummy as def::AArg>::comp_options(&h, ""));
    assert_eq!(
        h.into_inner(),
        vec![cmd!(Root), cmd!(SubCommand), arg!(AArg, "a1")]
    );
}

#[test]
fn test_flags_not_last() {
    let expected = (
        vec![
            cmd!(Root),
            b_flag!(C_FLAG),
            flag!(BFlag, "option"),
            cmd!(SubCommand),
        ]
        .into(),
        <Dummy as def::AArg>::comp_options(&Default::default(), ""),
    );

    let res = run("-c --long-b=option sub", true);
    assert_eq!(expected, res);
    let res = run("-c -b=option sub", true);
    assert_eq!(expected, res);
    let res = run("-cb=option sub", true);
    assert_eq!(expected, res);
    let res = run("-cb option sub", true);
    assert_eq!(expected, res);
    let res = run("-cboption sub", true);
    assert_eq!(expected, res);

    let (h, r) = run("-bc=option sub", true);
    assert_eq!(expected.1, r);
    assert_eq!(
        h.into_inner(),
        vec![cmd!(Root), flag!(BFlag, "c=option"), cmd!(SubCommand)]
    );
}

#[test]
fn test_once_flag() {
    let (h, r) = run("-", false);
    assert_eq!(h.into_inner(), vec![cmd!(Root)]);
    assert_eq!(
        map_comp_values(&r),
        vec!["--long-b", "--long-c", "--long-c-2", "-b", "-c", "-x"],
    );

    let (h, r) = run("-b option -", false);
    assert_eq!(h.into_inner(), vec![cmd!(Root), flag!(BFlag, "option")]);
    assert_eq!(map_comp_values(&r), vec!["--long-c", "--long-c-2", "-c"],);
}
