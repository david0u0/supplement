use clap::Parser;

#[derive(Parser, Debug)]
pub struct ArgConflict {
    pub one: Option<String>,
    #[clap(subcommand)]
    pub sub: SubCommand,
}
#[derive(Parser, Debug)]
pub struct ArgNoConflict {
    #[clap(long)]
    pub three: Option<String>,

    #[clap(subcommand)]
    pub sub: SubCommand,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    Two { arg: String },
    One { arg: String },
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::CommandFactory;
    use supplement::error::GenerateError;
    use supplement::{Config, generate};

    fn do_assrt(expected: &str, err: GenerateError) {
        let name = match err {
            GenerateError::CmdNameConflict(name) => name,
            _ => panic!(),
        };
        assert_eq!(name, expected);
    }

    #[test]
    fn test_conflict_name() {
        let mut s: Vec<u8> = vec![];
        let err = generate(&mut ArgConflict::command(), Config::new(), &mut s).unwrap_err();
        do_assrt("one", err);

        let cfg = Config::new().rename_cmd(&["one"], "my-one");
        generate(&mut ArgConflict::command(), cfg, &mut s).expect("rename should fix conflict");
    }

    #[test]
    fn test_rename_conflict() {
        let mut s: Vec<u8> = vec![];
        generate(&mut ArgNoConflict::command(), Default::default(), &mut s)
            .expect("no conflict before rename");

        let cfg = Config::new().rename_cmd(&["two"], "three");
        let err = generate(&mut ArgNoConflict::command(), cfg, &mut s).unwrap_err();
        do_assrt("three", err);

        let cfg = Config::new().rename_cmd(&["two"], "one");
        let err = generate(&mut ArgNoConflict::command(), cfg, &mut s).unwrap_err();
        do_assrt("one", err);
    }
}
