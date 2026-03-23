use error::Error;
use seen::*;
use supplement::completion::CompletionGroup;
use supplement::*;

mod def {
    use super::*;
    use supplement::core::*;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID {
        A,
        B,
        D,
        OPT2,
        E,
    }

    pub const C_FLAG_ID: id::NoVal = id::NoVal::new(line!());
    pub const C_FLAG: Flag<ID> = Flag {
        ty: flag_type::Type::new_bool(C_FLAG_ID),
        short: &['c'],
        long: &["long-c", "long-c-2"],
        description: "test description for flag C",
        once: true,
    };
    pub const B_FLAG_ID: id::SingleVal = id::SingleVal::new(line!());
    pub const B_FLAG: Flag<ID> = Flag {
        ty: flag_type::Type::new_valued(
            Some(ID::B),
            B_FLAG_ID.into(),
            CompleteWithEqual::NoNeed,
            &[],
        ),
        short: &['b', 'x'],
        long: &["long-b"],
        description: "test description for flag B",
        once: true,
    };
    pub const A_ARG_ID: id::SingleVal = id::SingleVal::new(line!());
    pub const A_ARG: Arg<ID> = Arg {
        id: Some(ID::A),
        seen_id: A_ARG_ID.into(),
        max_values: 1,
        possible_values: &[],
    };
    pub const E_ARG_ID: id::SingleVal = id::SingleVal::new(line!());
    pub const E_ARG: Arg<ID> = Arg {
        id: Some(ID::E),
        seen_id: E_ARG_ID.into(),
        max_values: 1,
        possible_values: &[("ext1", "")],
    };
    pub const ROOT: Command<ID> = Command {
        all_flags: &[B_FLAG, C_FLAG, OPT_FLAG],
        name: "root",
        description: "",
        args: &[E_ARG, D_ARG],
        commands: &[SUB],
    };
    pub const SUB: Command<ID> = Command {
        all_flags: &[B_FLAG, OPT2_FLAG],
        name: "sub",
        description: "test sub description",
        args: &[A_ARG, A_ARG],
        commands: &[],
    };
    pub const D_ARG_ID: id::MultiVal = id::MultiVal::new(line!());
    pub const D_ARG: Arg<ID> = Arg {
        id: Some(ID::D),
        seen_id: D_ARG_ID.into(),
        max_values: 2,
        possible_values: &[("p1", "")],
    };

    pub const OPT_FLAG_ID: id::SingleVal = id::SingleVal::new(line!());
    pub const OPT_FLAG: Flag<ID> = Flag {
        ty: flag_type::Type::new_valued(
            None,
            OPT_FLAG_ID.into(),
            CompleteWithEqual::Optional,
            &[("opt1", ""), ("opt2", "")],
        ),
        short: &['o'],
        long: &["opt"],
        description: "test description for flag OPT",
        once: true,
    };

    pub const OPT2_FLAG_ID: id::SingleVal = id::SingleVal::new(line!());
    pub const OPT2_FLAG: Flag<ID> = Flag {
        ty: flag_type::Type::new_valued(
            Some(ID::OPT2),
            OPT2_FLAG_ID.into(),
            CompleteWithEqual::Optional,
            &[("opt3", ""), ("opt4", "")],
        ),
        short: &['o'],
        long: &["opt"],
        description: "test description for flag OPT",
        once: true,
    };
}
use def::ID;

fn try_run(args: &str, last_is_empty: bool) -> (Vec<SeenUnit>, Result<CompletionGroup<ID>>) {
    let _ = env_logger::try_init();

    let args = args.split(' ').map(|s| s.to_owned());
    let args = std::iter::once("whatever".to_owned()).chain(args);
    let last = if last_is_empty {
        Some(String::new())
    } else {
        None
    };
    let args = args.chain(last);
    let mut seen = Seen::new();
    let res = def::ROOT.supplement_with_seen(&mut seen, args);
    (seen.into_inner(), res)
}
fn run(args: &str, last_is_empty: bool) -> (Vec<SeenUnit>, CompletionGroup<ID>) {
    let (h, r) = try_run(args, last_is_empty);
    (h, r.unwrap())
}

fn map_comps(comps: &[Completion]) -> Vec<&str> {
    let mut v: Vec<_> = comps.iter().map(|c| c.value.as_str()).collect();
    v.sort();
    v
}

fn map_unready(grp: &CompletionGroup<ID>) -> (ID, &str, Vec<&str>, &str) {
    match grp {
        CompletionGroup::Unready { unready, id, value } => {
            let preexist = map_comps(&unready.preexist);
            (*id, value, preexist, &unready.prefix)
        }
        _ => panic!("{:?} is ready", grp),
    }
}

fn map_comp_values(grp: &CompletionGroup<ID>) -> Vec<&str> {
    let v = match grp {
        CompletionGroup::Ready(r) => r.inner().0,
        _ => panic!("{:?} is unready", grp),
    };
    map_comps(v)
}

macro_rules! no {
    ($id:ident) => {
        SeenUnit::No(SeenUnitNoVal {
            id: def::$id,
            count: 1,
        })
    };
}
macro_rules! single {
    ($id:ident, $value:expr) => {
        SeenUnit::Single(SeenUnitSingleVal {
            id: def::$id,
            value: $value.to_owned(),
        })
    };
}
macro_rules! multi {
    ($id:ident, $value:expr) => {
        SeenUnit::Multi(SeenUnitMultiVal {
            id: def::$id,
            values: $value.iter().map(|s| s.to_string()).collect(),
        })
    };
}

#[test]
fn test_args_last() {
    let (h, r) = run("sub a1", true);
    assert_eq!(h, vec![single!(A_ARG_ID, "a1")]);
    assert_eq!(map_unready(&r), (ID::A, "", vec![], ""));
}

#[test]
fn test_flags_not_last() {
    let expected_h = vec![no!(C_FLAG_ID), single!(B_FLAG_ID, "option")];
    let expected_r = (ID::A, "", vec![], "");

    let (h, r) = run("-c --long-b=option sub", true);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), expected_r);
    let (h, r) = run("-c -b=option sub", true);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), expected_r);
    let (h, r) = run("-cb=option sub", true);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), expected_r);
    let (h, r) = run("-cb option sub", true);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), expected_r);
    let (h, r) = run("-cboption sub", true);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), expected_r);

    let (h, r) = run("-bc=option sub", true);
    assert_eq!(h, vec![single!(B_FLAG_ID, "c=option")]);
    assert_eq!(map_unready(&r), expected_r);
}

#[test]
fn test_once_flag() {
    let (h, r) = run("-", false);
    assert_eq!(h, vec![]);
    assert_eq!(
        map_comp_values(&r),
        vec!["--long-b", "--long-c", "--opt", "--opt="],
    );

    let (h, r) = run("-b option -", false);
    assert_eq!(h, vec![single!(B_FLAG_ID, "option")]);
    assert_eq!(map_comp_values(&r), vec!["--long-c", "--opt", "--opt="]);
}

#[test]
fn test_flags_last() {
    let expected_h = vec![no!(C_FLAG_ID)];

    let (h, r) = run("-c --long-b=x", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_unready(&r), (ID::B, "x", vec![], "--long-b="));

    let (h, r) = run("-c -b=x", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_unready(&r), (ID::B, "x", vec![], "-b="));

    let (h, r) = run("-cb=x", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_unready(&r), (ID::B, "x", vec![], "-cb="));

    let (h, r) = run("-cbx", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_unready(&r), (ID::B, "x", vec![], "-cb"));

    let (h, r) = run("-cb", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_unready(&r), (ID::B, "", vec![], "-cb"));

    let (h, r) = run("-c", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_comp_values(&r), vec!["-cb", "-co", "-co="]);
}

#[test]
fn test_flags_supplement() {
    let expected_h = vec![no!(C_FLAG_ID)];
    let expected_r = (ID::B, "x", vec![], "");

    let (h, r) = run("-c --long-b x", false);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), expected_r);

    let (h, r) = run("-c -b x", false);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), expected_r);

    let (h, r) = run("-cb x", false);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), expected_r);

    let (h, r) = run("-c x", false);
    assert_eq!(h, expected_h);
    assert_eq!(map_unready(&r), (ID::E, "x", vec!["ext1", "sub"], ""));
}

#[test]
fn test_fall_back_and_var_len_arg() {
    let (h, r) = run("arg1", true);
    assert_eq!(h, vec![single!(E_ARG_ID, "arg1")]);
    assert_eq!(map_unready(&r), (ID::D, "", vec!["p1"], ""));

    let (h, r) = run("arg1 d1", true);
    assert_eq!(h, vec![single!(E_ARG_ID, "arg1"), multi!(D_ARG_ID, ["d1"])]);
    assert_eq!(map_unready(&r), (ID::D, "", vec!["p1"], ""));

    let expected_h = vec![single!(E_ARG_ID, "arg1"), multi!(D_ARG_ID, ["d1", "d2"])];

    let (h, r) = try_run("arg1 d1 d2", true);
    assert_eq!(h, expected_h);
    assert_eq!(r.unwrap_err(), Error::UnexpectedArg("".to_owned()));

    let (h, r) = try_run("arg1 d1 d2 d3", true);
    assert_eq!(h, expected_h);
    assert_eq!(r.unwrap_err(), Error::UnexpectedArg("d3".to_owned()));
}

#[test]
fn test_flag_after_args() {
    let (h, r) = run("sub arg1 --", false);
    assert_eq!(h, vec![single!(A_ARG_ID, "arg1")]);
    assert_eq!(map_comp_values(&r), vec!["--long-b", "--opt", "--opt="],);

    let (h, r) = run("sub arg1 --long-b flag1", false);
    assert_eq!(h, vec![single!(A_ARG_ID, "arg1")]);
    assert_eq!(map_unready(&r), (ID::B, "flag1", vec![], ""));

    let (h, r) = run("sub arg1 --long-b flag1", true);
    assert_eq!(
        h,
        vec![single!(A_ARG_ID, "arg1"), single!(B_FLAG_ID, "flag1")]
    );
    assert_eq!(map_unready(&r), (ID::A, "", vec![], ""));
}

#[test]
fn test_flag_after_external_sub() {
    let (h, r) = run("--long-b flag1 ext", true);
    assert_eq!(
        h,
        vec![single!(B_FLAG_ID, "flag1"), single!(E_ARG_ID, "ext")]
    );
    assert_eq!(map_unready(&r), (ID::D, "", vec!["p1"], ""));

    let (h, r) = run("ext --", false);
    assert_eq!(h, vec![single!(E_ARG_ID, "ext")]);
    assert_eq!(map_unready(&r), (ID::D, "--", vec!["p1"], ""));

    let (h, r) = run("ext --long-b flag1", false);
    assert_eq!(
        h,
        vec![single!(E_ARG_ID, "ext"), multi!(D_ARG_ID, ["--long-b"])]
    );
    assert_eq!(map_unready(&r), (ID::D, "flag1", vec!["p1"], ""));

    let expected_h = vec![
        single!(E_ARG_ID, "ext"),
        multi!(D_ARG_ID, ["--long-b", "flag1"]),
    ];
    let (h, r) = try_run("ext --long-b flag1", true);
    assert_eq!(h, expected_h);
    assert_eq!(r.unwrap_err(), Error::UnexpectedArg("".to_owned()));
}

#[test]
fn test_optional_flag() {
    let (h, r) = run("--opt=", false);
    assert_eq!(h, vec![]);
    assert_eq!(map_comp_values(&r), vec!["--opt=opt1", "--opt=opt2"]);

    let expected_r = (ID::E, "", vec!["ext1", "sub"], "");

    let (h, r) = run("--opt=xxx", true);
    assert_eq!(h, vec![single!(OPT_FLAG_ID, "xxx")]);
    assert_eq!(map_unready(&r), expected_r);

    let (h, r) = run("--opt", true);
    assert_eq!(h, vec![single!(OPT_FLAG_ID, "")]);
    assert_eq!(map_unready(&r), expected_r);

    let (h, r) = run("--opt sub", true);
    assert_eq!(h, vec![single!(OPT_FLAG_ID, "")]);
    assert_eq!(map_unready(&r), (ID::A, "", vec![], ""));

    // test short flags

    let (h, r) = run("-oba", false);
    assert_eq!(h, vec![single!(OPT_FLAG_ID, "")]);
    assert_eq!(map_unready(&r), (ID::B, "a", vec![], "-ob"));

    let (h, r) = run("-oba", true);
    assert_eq!(h, vec![single!(OPT_FLAG_ID, ""), single!(B_FLAG_ID, "a")]);
    assert_eq!(map_unready(&r), expected_r);

    let (h, r) = run("-ob a", false);
    assert_eq!(h, vec![single!(OPT_FLAG_ID, "")]);
    assert_eq!(map_unready(&r), (ID::B, "a", vec![], ""));

    let (h, r) = run("-c", false);
    assert_eq!(h, vec![no!(C_FLAG_ID)]);
    assert_eq!(map_comp_values(&r), vec!["-cb", "-co", "-co="]);

    let (h, r) = run("-co", false);
    assert_eq!(h, vec![no!(C_FLAG_ID)]);
    assert_eq!(map_comp_values(&r), vec!["-co=opt1", "-co=opt2"]);

    let (h, r) = try_run("-oz", false);
    assert_eq!(h, vec![single!(OPT_FLAG_ID, "")]);
    assert_eq!(r.unwrap_err(), Error::FlagNotFound("z".to_owned()));
}

#[test]
fn test_custom_with_possible() {
    let (h, r) = run("sub -", false);
    assert_eq!(h, vec![]);
    assert_eq!(map_comp_values(&r), vec!["--long-b", "--opt", "--opt="]);

    let (h, r) = run("sub --opt=x", false);
    assert_eq!(h, vec![]);
    assert_eq!(
        map_unready(&r),
        (ID::OPT2, "x", vec!["opt3", "opt4"], "--opt=")
    );

    let (h, r) = run("sub -o", false);
    assert_eq!(h, vec![]);
    assert_eq!(map_unready(&r), (ID::OPT2, "", vec!["opt3", "opt4"], "-o="));
}
