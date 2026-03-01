#[path = "src/args.rs"]
mod args;
use args::Arg;

#[path = "src/generate.rs"]
mod generate;

use std::path::Path;

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let file = Path::new(&out_dir).join("definition.rs");
    let mut f = std::fs::File::create(file).unwrap();

    // Ignore error. Otherwise it's hard to debug generate Error.
    let _ = generate::my_gen::<Arg>(&mut f);
}
