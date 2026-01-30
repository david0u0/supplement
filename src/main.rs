use supplements::*;

mod def {
    use super::*;

    pub const C_FLAG: Flag = Flag {
        id: SupplementID::new(line!(), "long-c"),
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
            SupplementID::new(line!(), "long-b")
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
            SupplementID::new(line!(), "")
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
            SupplementID::new(line!(), "root")
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
            SupplementID::new(line!(), "sub")
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

    do_it("-bc=abcd sub", true);
    //
    //do_it("-", false);
    //do_it("-b a -", false); // test "once"
    //do_it("--", false);
    //do_it("sub -", false);
    //do_it("sub --", false);
    //do_it("-b", true);
    //do_it("sub -bmy_option", true);
    //
    //do_it("-bmy_option sub", true);
    //
    //do_it("-cbmy_option sub", true);
}
