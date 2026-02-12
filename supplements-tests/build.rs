#[path = "src/args.rs"]
mod args;
use args::Arg;

use clap::CommandFactory;
use std::io::Write;
use std::path::Path;
use supplements::{Config, generate, generate_default};

fn main() {
    let config = Config::default()
        .ignore(&["ignored-cmd"])
        .ignore(&["checkout", "flag1"])
        .ignore(&["log", "flag2"])
        .ignore(&["flag3"]);

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let file = Path::new(&out_dir).join("definition.rs");
    let mut f = std::fs::File::create(file).unwrap();
    generate(&mut Arg::command(), config.clone(), &mut f).unwrap();

    let file = Path::new(&out_dir).join("dummy_impl.rs");
    let mut f = std::fs::File::create(file).unwrap();
    writeln!(f, "use super::*;").unwrap();
    generate_default(&mut Arg::command(), config, &mut f).unwrap();
}
