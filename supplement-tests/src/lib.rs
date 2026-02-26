use std::fmt::Debug;
use supplement::Result;
pub mod args;
pub mod conflict_name;
use supplement::{Completion, CompletionGroup};

mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}

pub fn run(cmd: &str) -> Result<CompletionGroup<def::ID>> {
    let cmd = cmd.split(" ").map(|s| s.to_string());
    let (_, grp) = def::CMD.supplement(cmd)?;
    Ok(grp)
}

fn map_comps(comps: &[Completion]) -> Vec<&str> {
    let mut v: Vec<_> = comps.iter().map(|c| c.value.as_str()).collect();
    v.sort();
    v
}

pub fn map_ready<ID: Debug>(grp: &CompletionGroup<ID>) -> Vec<&str> {
    let v = match grp {
        CompletionGroup::Ready(r) => r.inner().0,
        _ => panic!("{:?} is unready", grp),
    };
    map_comps(v)
}

pub fn map_unready<ID: Debug + Copy>(grp: &CompletionGroup<ID>) -> (ID, &str, Vec<&str>, &str) {
    match grp {
        CompletionGroup::Unready { unready, id, value } => {
            let preexist = map_comps(&unready.preexist);
            (*id, value, preexist, &unready.prefix)
        }
        _ => panic!("{:?} is ready", grp),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::args::Arg;
    use clap::CommandFactory;
    use def::ID;
    use supplement::error::GenerateError;
    use supplement::{Config, generate};

    #[test]
    fn test_unprocessed_conf() {
        fn do_assrt(err: GenerateError) {
            let v = match err {
                GenerateError::UnprocessedConfigObj(v) => v,
                _ => panic!(),
            };
            assert_eq!(v, vec![vec!["some-cmd".to_owned(), "pretty".to_owned()]]);
        }

        let mut s: Vec<u8> = vec![];
        let cfg = Config::new().ignore(&["some-cmd", "pretty"]);

        let err = generate(&mut Arg::command(), cfg, &mut s).unwrap_err();
        do_assrt(err);
    }

    #[test]
    fn test_simple() {
        let comps = run("git -").unwrap();
        assert_eq!(vec!["--git-dir"], map_ready(&comps));

        let comps = run("git g").unwrap();
        assert_eq!(
            map_unready(&comps),
            (
                ID::External,
                "g",
                vec!["bisect", "checkout", "log", "remote"],
                ""
            )
        );

        let comps = run("git remote g").unwrap();
        assert_eq!(map_ready(&comps), vec!["add", "remove"]);

        let comps = run("git log -").unwrap();
        assert_eq!(
            vec!["--flag1", "--git-dir", "--graph", "--pretty", "--pretty="],
            map_ready(&comps)
        );

        let comps = run("git checkout -").unwrap();
        assert_eq!(vec!["--git-dir"], map_ready(&comps));

        let comps = run("git log --pretty=").unwrap();
        assert_eq!(
            vec!["--pretty=full", "--pretty=oneline", "--pretty=short"],
            map_ready(&comps)
        );

        let comps = run("git bisect x").unwrap();
        assert_eq!(vec!["bad", "good"], map_ready(&comps));
    }

    #[test]
    fn test_rename() {
        let comps = run("git remote add x").unwrap();
        let id = ID::Remote(def::remote::ID::MyAdd(def::remote::my_add::ID::Remote));
        assert_eq!(map_unready(&comps), (id, "x", vec![], ""));
    }
}
