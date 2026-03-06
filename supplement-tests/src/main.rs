use clap::CommandFactory;
use std::io::Write;
use std::path::Path;
use supplement::{Config, error::GenerateError, generate};
use supplement_tests::args::Arg;

fn my_gen(w: &mut impl Write) -> Result<(), GenerateError> {
    let config = Config::default()
        .ignore(&["ignored-cmd"])
        .ignore(&["checkout", "flag1"])
        .ignore(&["log", "flag2"])
        .ignore(&["flag3"])
        .ignore(&["remote", "add", "git_dir"])
        .make_uncertain(&["bisect2", "arg"])
        .make_uncertain(&["bisect2", "pretty"]);

    generate(&mut Arg::command(), config, w)
}

fn main() {
    env_logger::init();

    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/def.rs");
    let mut file = std::fs::File::create(file).unwrap();
    my_gen(&mut file).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gen() {
        let mut v: Vec<u8> = vec![];
        my_gen(&mut v).unwrap();
        let gen_content = String::from_utf8(v).unwrap();
        let file_content = include_str!("def.rs");
        assert_eq!(
            gen_content, file_content,
            "The code-gen content has changed. Please regenerate by `cargo run`"
        );
    }
}
