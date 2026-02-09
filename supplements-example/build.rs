#[cfg(feature = "clap-4")]
#[path = "src/args.rs"]
mod args;

#[cfg(feature = "clap-3")]
#[path = "src/args_clap3.rs"]
mod args;

#[cfg(feature = "clap-3")]
use clap3 as clap;
#[cfg(feature = "clap-4")]
use clap4 as clap;

use args::Git;
use clap::CommandFactory;
use std::path::Path;
use supplements::generate;

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let file = Path::new(&out_dir).join("definition.rs");
    let mut f = std::fs::File::create(file).unwrap();
    generate(&mut Git::command(), &mut f).unwrap();
}
