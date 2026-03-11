pub mod args;
pub mod my_gen;
use std::fmt::Debug;
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

pub fn map_unready<ID: Debug + Copy>(grp: &CompletionGroup<ID>) -> (ID, (&str, Vec<&str>, &str)) {
    match grp {
        CompletionGroup::Unready { unready, id, value } => {
            let preexist = map_comps(&unready.preexist);
            (*id, (value, preexist, &unready.prefix))
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
    use def::ID;
    use supplement::{Result, helper::id, Seen};

    fn run_with_seen(cmd: &str) -> Result<(Seen<ID>, CompletionGroup<ID>)> {
        let cmd = cmd.split(" ").map(|s| s.to_string());
        def::CMD.supplement(cmd)
    }
    fn run(cmd: &str) -> Result<CompletionGroup<ID>> {
        let (_, c) = run_with_seen(cmd)?;
        Ok(c)
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
        let (id, comps) = map_unready(&comps);
        assert!(matches!(id, id!(def @ext)));
        assert_eq!(
            comps,
            (
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
        assert_eq!(vec!["--git-dir", "-b"], map_ready(&comps));

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
        let comps = run("git bisect2 x").unwrap();
        let (id, comps) = map_unready(&comps);
        assert!(matches!(id, id!(def bisect2 arg)));
        assert_eq!(comps, ("x", vec!["bad", "good"], ""));

        let comps = run("git bisect2 --pretty=z").unwrap();
        let (id, comps) = map_unready(&comps);
        assert!(matches!(id, id!(def bisect2 pretty)));
        assert_eq!(comps, ("z", vec!["full", "oneline", "short"], "--pretty="));
    }

    #[test]
    fn test_ctx() {
        let (h, comps) = run_with_seen("git --external e --git-dir=").unwrap();
        let (id, _) = map_unready(&comps);
        match id.with_seen(&h) {
            id!(def(root) checkout files) | id!(def(root) git_dir) => {
                assert_eq!(root.val_git_dir(), None);
                assert_eq!(root.val_external(), &["e"]);
            }
            _ => panic!("id is {id:?}"),
        }

        let (h, comps) = run_with_seen("git --git-dir mydir checkout ww xx yy zz").unwrap();
        let (id, _) = map_unready(&comps);
        match id.with_seen(&h) {
            id!(def(root) checkout(chk) files) => {
                assert_eq!(root.val_git_dir(), Some("mydir"));
                assert_eq!(chk.val_file_or_commit(), Some("ww"));
                assert_eq!(chk.val_files(), &["xx", "yy"]);
                assert_eq!(chk.val_b(), 0);
            }
            _ => panic!("id is {id:?}"),
        }

        let (h, comps) = run_with_seen("git --external e time for some ext").unwrap();
        let (id, _) = map_unready(&comps);
        match id.with_seen(&h) {
            id!(def(root) @ext) => {
                assert_eq!(root.val_git_dir(), None);
                assert_eq!(root.val_external(), &["e"]);
                assert_eq!(root.external(), &["time", "for", "some"]);
            }
            _ => panic!("id is {id:?}"),
        }
    }
}
