#[path = "src/args.rs"]
mod args;
#[path = "src/my_gen.rs"]
mod my_gen;

use args::Arg;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/def.rs");

    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/def.rs");
    if file.exists() {
        // Don't generate the file if it exists.
        // So that we can still manually change that file and test the change.
        return;
    }

    // Fall back to code-gen
    let mut f = std::fs::File::create(file).unwrap();
    my_gen::my_gen::<Arg>(&mut f).unwrap();
}
