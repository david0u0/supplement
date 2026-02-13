use error::Error;
use history::*;
use supplements::*;

mod def {
    use super::*;

    pub const C_FLAG_ID: id::NoVal = id::NoVal::new(line!(), "");
    pub const C_FLAG: Flag = Flag {
        id: id::Flag::No(C_FLAG_ID),
        short: &['c'],
        long: &["long-c", "long-c-2"],
        description: "test description for flag C",
        comp_options: None,
        once: true,
    };
    pub const B_FLAG_ID: id::SingleVal = id::SingleVal::new(line!(), "");
    pub const B_FLAG: Flag = Flag {
        id: id::Flag::Single(B_FLAG_ID),
        short: &['b', 'x'],
        long: &["long-b"],
        description: "test description for flag B",
        comp_options: Some(|_history, arg| {
            let mut ret = vec![];
            if arg != "" {
                ret.push(Completion::new(arg, ""));
            }
            ret.push(Completion::new(&format!("{arg}!"), ""));
            ret
        }),
        once: true,
    };
    pub const A_ARG_ID: id::SingleVal = id::SingleVal::new(line!(), "");
    pub const A_ARG: Arg = Arg {
        id: id::Arg::Single(A_ARG_ID),
        comp_options: |_, _| {
            vec![
                Completion::new("arg-option1", ""),
                Completion::new("arg-option2", ""),
            ]
        },
        max_values: 1,
    };
    pub const ROOT_ID: id::NoVal = id::NoVal::new(line!(), "");
    pub const ROOT: Command = Command {
        id: ROOT_ID,
        all_flags: &[B_FLAG, C_FLAG],
        name: "root",
        description: "",
        args: &[A_ARG, D_ARG],
        commands: &[SUB],
    };
    pub const SUB_ID: id::NoVal = id::NoVal::new(line!(), "");
    pub const SUB: Command = Command {
        id: SUB_ID,
        all_flags: &[B_FLAG],
        name: "sub",
        description: "test sub description",
        args: &[A_ARG, A_ARG],
        commands: &[],
    };
    pub const D_ARG_ID: id::MultiVal = id::MultiVal::new(line!(), "");
    pub const D_ARG: Arg = Arg {
        id: id::Arg::Multi(D_ARG_ID),
        comp_options: |_, _| vec![Completion::new("d-arg!", "")],
        max_values: 2,
    };
}

fn try_run(args: &str, last_is_empty: bool) -> (Vec<HistoryUnit>, Result<Vec<Completion>>) {
    let _ = env_logger::try_init();

    let args = args.split(' ').map(|s| s.to_owned());
    let args = std::iter::once("whatever".to_owned()).chain(args);
    let last = if last_is_empty {
        Some(String::new())
    } else {
        None
    };
    let args = args.chain(last.into_iter());
    let mut history = History::default();
    let res = def::ROOT.supplement_with_history(&mut history, args);
    let res = res.map(|r| r.into_inner().0);
    (history.into_inner(), res)
}
fn run(args: &str, last_is_empty: bool) -> (Vec<HistoryUnit>, Vec<Completion>) {
    let (h, r) = try_run(args, last_is_empty);
    (h, r.unwrap())
}
fn map_comp_values(arr: &[Completion]) -> Vec<&str> {
    let mut v: Vec<_> = arr.iter().map(|c| &*c.value).collect();
    v.sort();
    v
}

macro_rules! no {
    ($id:ident) => {
        HistoryUnit::No(HistoryUnitNoVal {
            id: def::$id,
            count: 1,
        })
    };
}
macro_rules! single {
    ($id:ident, $value:expr) => {
        HistoryUnit::Single(HistoryUnitSingleVal {
            id: def::$id,
            value: $value.to_owned(),
        })
    };
}
macro_rules! multi {
    ($id:ident, $value:expr) => {
        HistoryUnit::Multi(HistoryUnitMultiVal {
            id: def::$id,
            values: $value.iter().map(|s| s.to_string()).collect(),
        })
    };
}

#[test]
fn test_args_last() {
    let (h, r) = run("sub a1", true);
    assert_eq!(h, vec![no!(SUB_ID), single!(A_ARG_ID, "a1")]);
    assert_eq!(map_comp_values(&r), vec!["arg-option1", "arg-option2"]);
}

#[test]
fn test_flags_not_last() {
    let expected = (
        vec![no!(C_FLAG_ID), single!(B_FLAG_ID, "option"), no!(SUB_ID)],
        (def::A_ARG.comp_options)(&Default::default(), ""),
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
    assert_eq!(h, vec![single!(B_FLAG_ID, "c=option"), no!(SUB_ID)]);
}

#[test]
fn test_once_flag() {
    let (h, r) = run("-", false);
    assert_eq!(h, vec![]);
    assert_eq!(
        map_comp_values(&r),
        vec!["--long-b", "--long-c", "--long-c-2", "-b", "-c", "-x"],
    );

    let (h, r) = run("-b option -", false);
    assert_eq!(h, vec![single!(B_FLAG_ID, "option")]);
    assert_eq!(map_comp_values(&r), vec!["--long-c", "--long-c-2", "-c"],);
}

#[test]
fn test_flags_last() {
    let expected_h = vec![no!(C_FLAG_ID)];

    let (h, r) = run("-c --long-b=x", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_comp_values(&r), vec!["--long-b=x", "--long-b=x!"]);

    let (h, r) = run("-c -b=x", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_comp_values(&r), vec!["-b=x", "-b=x!"]);

    let (h, r) = run("-cb=x", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_comp_values(&r), vec!["-cb=x", "-cb=x!"]);

    let (h, r) = run("-cbx", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_comp_values(&r), vec!["-cbx", "-cbx!"]);

    let (h, r) = run("-cb", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_comp_values(&r), vec!["-cb!"]);

    let (h, r) = run("-c", false);
    assert_eq!(expected_h, h);
    assert_eq!(map_comp_values(&r), vec!["-cb", "-cx"]);
}

#[test]
fn test_flags_supplement() {
    let expected = (
        vec![no!(C_FLAG_ID)],
        (def::B_FLAG.comp_options.unwrap())(&Default::default(), "x"),
    );

    let res = run("-c --long-b x", false);
    assert_eq!(expected, res);
    let res = run("-c -b x", false);
    assert_eq!(expected, res);
    let res = run("-cb x", false);
    assert_eq!(expected, res);

    let res = run("-c x", false);
    assert_eq!(expected.0, res.0);
    assert_eq!(
        map_comp_values(&res.1),
        vec!["arg-option1", "arg-option2", "sub"]
    );
}

#[test]
fn test_fall_back_and_var_len_arg() {
    let (h, r) = run("arg1", true);
    assert_eq!(h, vec![single!(A_ARG_ID, "arg1")]);
    assert_eq!(map_comp_values(&r), vec!["d-arg!"]);

    let (h, r) = run("arg1 d1", true);
    assert_eq!(h, vec![single!(A_ARG_ID, "arg1"), multi!(D_ARG_ID, ["d1"])]);
    assert_eq!(map_comp_values(&r), vec!["d-arg!"]);

    let expected_h = vec![single!(A_ARG_ID, "arg1"), multi!(D_ARG_ID, ["d1", "d2"])];

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
    assert_eq!(h, vec![no!(SUB_ID), single!(A_ARG_ID, "arg1")]);
    assert_eq!(map_comp_values(&r), vec!["--long-b"],);

    let (h, r) = run("sub arg1 --long-b flag1", false);
    assert_eq!(h, vec![no!(SUB_ID), single!(A_ARG_ID, "arg1")]);
    assert_eq!(map_comp_values(&r), vec!["flag1", "flag1!"],);

    let (h, r) = run("sub arg1 --long-b flag1", true);
    assert_eq!(
        h,
        vec![
            no!(SUB_ID),
            single!(A_ARG_ID, "arg1"),
            single!(B_FLAG_ID, "flag1")
        ]
    );
    assert_eq!(map_comp_values(&r), vec!["arg-option1", "arg-option2"],);
}

#[test]
fn test_flag_after_external_sub() {
    let expected_r = vec!["d-arg!"];

    let (h, r) = run("--long-b flag1 ext", true);
    assert_eq!(
        h,
        vec![single!(B_FLAG_ID, "flag1"), single!(A_ARG_ID, "ext")]
    );
    assert_eq!(map_comp_values(&r), expected_r);

    let (h, r) = run("ext --", false);
    assert_eq!(h, vec![single!(A_ARG_ID, "ext")]);
    assert_eq!(map_comp_values(&r), expected_r);

    let (h, r) = run("ext --long-b flag1", false);
    assert_eq!(
        h,
        vec![single!(A_ARG_ID, "ext"), multi!(D_ARG_ID, ["--long-b"])]
    );
    assert_eq!(map_comp_values(&r), expected_r);

    let expected_h = vec![
        single!(A_ARG_ID, "ext"),
        multi!(D_ARG_ID, ["--long-b", "flag1"]),
    ];
    let (h, r) = try_run("ext --long-b flag1", true);
    assert_eq!(h, expected_h);
    assert_eq!(r.unwrap_err(), Error::UnexpectedArg("".to_owned()));
}
