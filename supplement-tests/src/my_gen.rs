use clap::CommandFactory;
use std::io::Write;
use supplement::{
    error::GenerateError,
    generate::{Config, generate},
};

pub fn my_gen<C: CommandFactory>(w: &mut impl Write) -> Result<(), GenerateError> {
    let config = Config::default()
        .ignore(&["ignored-cmd"])
        .ignore(&["checkout", "flag1"])
        .ignore(&["log", "flag2"])
        .ignore(&["flag3"])
        .ignore(&["remote", "add", "git_dir"])
        .make_custom(&["bisect2", "arg"])
        .make_custom(&["bisect2", "pretty"]);

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
        let gen_content = String::from_utf8(v).unwrap();
        let file_content = include_str!("def.rs");
        assert_eq!(
            gen_content, file_content,
            "The code-gen content has changed. Please remove `src/def.rs` and build again."
        );
    }
}
