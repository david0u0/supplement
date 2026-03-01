use clap::CommandFactory;
use std::io::Write;
use supplement::{Config, error::GenerateError, generate};

pub fn my_gen<C: CommandFactory>(w: &mut impl Write) -> Result<(), GenerateError> {
    let config = Config::default()
        .ignore(&["ignored-cmd"])
        .ignore(&["checkout", "flag1"])
        .ignore(&["log", "flag2"])
        .ignore(&["flag3"])
        .ignore(&["remote", "add", "git_dir"]);

    generate(&mut C::command(), config, w)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::args::Arg;

    #[test]
    fn test_gen() {
        let mut v: Vec<u8> = vec![];
        my_gen::<Arg>(&mut v).unwrap();
    }
}
