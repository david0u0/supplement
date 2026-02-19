#[path = "src/args.rs"]
mod args;
use args::Arg;

use clap::CommandFactory;
use std::path::Path;
use supplement::{Config, generate};

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
}
