use std::fmt::Debug;
pub mod args;
pub mod dummy;
use supplements::{Completion, CompletionGroup};

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
    #[test]
    fn test_unprocessed_conf() {
        use crate::args::Arg;
        use clap::CommandFactory;
        use supplements::error::GenerateError;
        use supplements::{Config, generate};

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
}
