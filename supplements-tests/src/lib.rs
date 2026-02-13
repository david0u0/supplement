pub mod args;
pub mod dummy;
use supplements::completion::CompletionGroup;

pub fn map_comps(comps: &CompletionGroup) -> Vec<&str> {
    let mut ret: Vec<_> = comps.inner().0.iter().map(|c| c.value.as_str()).collect();
    ret.sort();
    ret
}

#[cfg(test)]
mod test {
    #[test]
    fn test_unprocessed_conf() {
        use crate::args::Arg;
        use clap::CommandFactory;
        use supplements::error::GenerateError;
        use supplements::{Config, generate, generate_default};

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

        let err = generate_default(&mut Arg::command(), cfg, &mut s).unwrap_err();
        do_assrt(err);
    }
}
