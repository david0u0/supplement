#[path = "src/args.rs"]
mod args;
use args::Arg;

use clap::CommandFactory;
use std::io::Write;
use std::path::Path;
use supplements::{generate, generate_default};

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let file = Path::new(&out_dir).join("definition.rs");
    let mut f = std::fs::File::create(file).unwrap();
    generate(&mut Arg::command(), Default::default(), &mut f).unwrap();

    let file = Path::new(&out_dir).join("dummy_impl.rs");
    let mut f = std::fs::File::create(file).unwrap();
    writeln!(f, "use super::*;").unwrap();
    generate_default(&mut Arg::command(), Default::default(), &mut f).unwrap();
}
