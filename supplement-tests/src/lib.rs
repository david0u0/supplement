use std::fmt::Debug;
pub mod args;
use supplement::{Completion, CompletionGroup};

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
    mod def {
        include!("def.rs");
    }

    use super::*;
    use def::Ctx as Ctx1;
    use def::ID;
    use supplement::{Result, helper::id};

    fn run(cmd: &str) -> Result<CompletionGroup<def::ID>> {
        let cmd = cmd.split(" ").map(|s| s.to_string());
        let (_, grp) = def::CMD.supplement(cmd)?;
        Ok(grp)
    }

    #[test]
    fn test_unprocessed_conf() {
        use crate::args::Arg;
        use clap::CommandFactory;
        use supplement::error::GenerateError;
        use supplement::{Config, generate};

        fn do_assrt(err: GenerateError) {
            let v = match err {
                GenerateError::UnprocessedConfigObj(v) => v,
                _ => panic!(),
            };
            assert_eq!(v, vec![vec!["some-cmd".to_owned(), "pretty".to_owned()]]);
        }

        let mut s: Vec<u8> = vec![];
        let cfg = Config::new().ignore(&["some-cmd", "pretty"]);
        let err = generate(&mut Arg::command(), cfg.clone(), &mut s).unwrap_err();
        do_assrt(err);
    }
    #[test]
    fn test_gen_uncertain() {
        use crate::args::Arg;
        use clap::CommandFactory;
        use supplement::error::GenerateError;
        use supplement::{Config, generate};

        let mut s: Vec<u8> = vec![];
        let cfg = Config::new().make_uncertain(&["log", "commit"]);
        let err = generate(&mut Arg::command(), cfg.clone(), &mut s).unwrap_err();
        let is_match = matches!(err, GenerateError::AlreadyUncertain(s) if s == "commit");
        assert!(is_match);

        let cfg = Config::new().make_uncertain(&["log", "graph"]);
        let err = generate(&mut Arg::command(), cfg.clone(), &mut s).unwrap_err();
        let is_match = matches!(err, GenerateError::UncertainWithoutValue(s) if s == "graph");
        assert!(is_match);
    }

    #[test]
    fn test_simple() {
        let comps = run("git -").unwrap();
        assert_eq!(vec!["--external", "--git-dir"], map_ready(&comps));

        let comps = run("git g").unwrap();
        assert_eq!(
            map_unready(&comps),
            (
                id!(def @ext(Ctx1)),
                "g",
                vec!["bisect", "bisect2", "checkout", "log", "remote"],
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

        // test possible value for arg
        let comps = run("git bisect x").unwrap();
        assert_eq!(vec!["bad", "good"], map_ready(&comps));

        // test ignoring global flags
        let comps = run("git remote add -").unwrap();
        assert_eq!(vec!["--tags"], map_ready(&comps));
    }

    #[test]
    fn test_made_uncertain() {
        use def::bisect2::Ctx as Ctx2;
        let comps = run("git bisect2 x").unwrap();
        assert_eq!(
            map_unready(&comps),
            (
                id!(def bisect2 arg(Ctx1, Ctx2)),
                "x",
                vec!["bad", "good"],
                ""
            )
        );

        let comps = run("git bisect2 --pretty=z").unwrap();
        assert_eq!(
            map_unready(&comps),
            (
                id!(def bisect2 pretty(Ctx1, Ctx2)),
                "z",
                vec!["full", "oneline", "short"],
                "--pretty="
            )
        );
    }

    #[test]
    fn test_ctx() {
        let comps = run("git --git-dir=").unwrap();
        match comps {
            CompletionGroup::Unready { id, .. } => match id {
                id!(def git_dir) => {}
                _ => panic!(),
            },
            _ => panic!(),
        }
    }
}
