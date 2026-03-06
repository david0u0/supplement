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
    use supplement::{Result, helper::id};
    type History = supplement::History<ID>;

    fn run<'a>(history: &'a mut History, cmd: &str) -> Result<CompletionGroup<ID<&'a History>>> {
        let cmd = cmd.split(" ").map(|s| s.to_string());
        def::CMD.supplement2(history, cmd)
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
        let mut h = History::new();
        let comps = run(&mut h, "git -").unwrap();
        assert_eq!(vec!["--external", "--git-dir"], map_ready(&comps));

        let mut h = History::new();
        let comps = run(&mut h, "git g").unwrap();
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

        let mut h = History::new();
        let comps = run(&mut h, "git remote g").unwrap();
        assert_eq!(map_ready(&comps), vec!["add", "remove"]);

        let mut h = History::new();
        let comps = run(&mut h, "git log -").unwrap();
        assert_eq!(
            vec!["--flag1", "--git-dir", "--graph", "--pretty", "--pretty="],
            map_ready(&comps)
        );

        let mut h = History::new();
        let comps = run(&mut h, "git checkout -").unwrap();
        assert_eq!(vec!["--git-dir", "-b"], map_ready(&comps));

        let mut h = History::new();
        let comps = run(&mut h, "git log --pretty=").unwrap();
        assert_eq!(
            vec!["--pretty=full", "--pretty=oneline", "--pretty=short"],
            map_ready(&comps)
        );

        // test possible value for arg
        let mut h = History::new();
        let comps = run(&mut h, "git bisect x").unwrap();
        assert_eq!(vec!["bad", "good"], map_ready(&comps));

        // test ignoring global flags
        let mut h = History::new();
        let comps = run(&mut h, "git remote add -").unwrap();
        assert_eq!(vec!["--tags"], map_ready(&comps));
    }

    #[test]
    fn test_made_uncertain() {
        let mut h = History::new();
        let comps = run(&mut h, "git bisect2 x").unwrap();
        let (id, comps) = map_unready(&comps);
        assert!(matches!(id, id!(def bisect2 arg)));
        assert_eq!(comps, ("x", vec!["bad", "good"], ""));

        let mut h = History::new();
        let comps = run(&mut h, "git bisect2 --pretty=z").unwrap();
        let (id, comps) = map_unready(&comps);
        assert!(matches!(id, id!(def bisect2 pretty)));
        assert_eq!(comps, ("z", vec!["full", "oneline", "short"], "--pretty="));
    }

    #[test]
    fn test_ctx() {
        let mut h = History::new();
        let comps = run(&mut h, "git --external e --git-dir=").unwrap();
        let (id, _) = map_unready(&comps);
        match id {
            id!(def checkout files(root, _)) | id!(def git_dir(root)) => {
                assert_eq!(root.val_git_dir(), None);
                assert_eq!(root.val_external(), &["e"]);
            }
            _ => panic!("id is {id:?}"),
        }

        let mut h = History::new();
        let comps = run(&mut h, "git --git-dir mydir checkout ww xx yy zz").unwrap();
        let (id, _) = map_unready(&comps);
        match id {
            id!(def checkout files(root, chk)) => {
                assert_eq!(root.val_git_dir(), Some("mydir"));
                assert_eq!(chk.val_file_or_commit(), Some("ww"));
                assert_eq!(chk.val_files(), &["xx", "yy"]);
                assert_eq!(chk.val_b(), 0);
            }
            _ => panic!("id is {id:?}"),
        }

        let mut h = History::new();
        let comps = run(&mut h, "git --external e time for some ext").unwrap();
        let (id, _) = map_unready(&comps);
        match id {
            id!(def @ext(root)) => {
                assert_eq!(root.val_git_dir(), None);
                assert_eq!(root.val_external(), &["e"]);
                assert_eq!(root.external(), &["time", "for", "some"]);
            }
            _ => panic!("id is {id:?}"),
        }
    }
}
