# Supplement
> Shell-agnostic, extensible CLI completion for Rust 💊

[![Crates.io](https://img.shields.io/crates/v/supplement?style=flat-square)](https://crates.io/crates/supplement)
[![Crates.io](https://img.shields.io/crates/d/supplement?style=flat-square)](https://crates.io/crates/supplement)
![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
[![Doc](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/supplement)

**supplement** is a Rust library that generates completion scaffolds as Rust code.

Give it a [`clap`](https://github.com/clap-rs/clap) object, and instead of spitting out shell files that you later have to manually edit, it spits out Rust! supplement is:
- **Shell-agnostic**
- **Powerful** - Some features are not widely supported in every shell, and `supplement` comes to the rescue.
- **Stop modifying generated files** - Instead, *extend* it with Rust's enum system.
- **Easy to test and debug** - Functions and objects in a modern programming language, instead of some shell script black sorcery.
- **It's Rust 🦀**

## Install
Add one line in Cargo.toml. By default, it uses `clap` 4, but you can make it use `clap` 3 with features.
```toml
[dependencies]
supplement = "0.1"
# Or, to use clap 3
supplement = { version = "0.1", default-features = false, features = ["clap-3"] }
# Or, disable the code-gen feature completely
supplement = { version = "0.1", default-features = false }
```

## Quick start
Say you have some awesome clap definition, and want to use supplement to make it even more awesome.

<details>
<summary>
Click to show clap definition
</summary>

```rs
// src/args.rs

use clap::{CommandFactory, Parser, ValueEnum};

#[derive(Parser, Debug)]
pub struct Git {
    #[clap(long)]
    pub git_dir: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    pub sub: SubCommand,
}
#[derive(Parser, Debug)]
pub enum SubCommand {
    Checkout {
        file_or_commit: Option<String>,
        files: Vec<std::path::PathBuf>,
    },

    #[clap(about = "log")]
    Log {
        #[clap(short, long)]
        graph: bool,
        #[clap(short, long, num_args = 0..=1, default_value = "short", default_missing_value = "full", require_equals = true)]
        pretty: Option<Pretty>,
        #[clap(short, long, default_value = "auto")]
        color: Color,
        commit: Option<String>,
    },
}

// more definition...
```
</details>

You can call `supplement::generate` to generate the completion file (preferably in `build.rs`):

<details>
<summary>
Click to show build.rs
</summary>

```rs
#[path = "src/args.rs"]
mod args; // path to the clap definition

use args::Git;
use clap::CommandFactory;
use std::path::Path;
use supplement::generate;

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let file = Path::new(&out_dir).join("definition.rs");
    let mut f = std::fs::File::create(file).unwrap();

    // Code-gen to a file like target/debug/build/myapp-3b7bb95d1e25522b/out/definition.rs
    generate(&mut Git::command(), Default::default(), &mut f).unwrap();
}
```
</details>

### Implementation

Utilize the generated code in `main.rs`.
Note that, if you missed some implementation, it's a *compile time error*.
So just relex and let Rust get your back 💪

```rs
use supplement::{*, helper::id};

mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs")); // This is generated
}

fn main() {
    // `args` looks like ["the-binary-name", "git", "log", "--graph"]
    // so we should skip the first arg
    let args = std::env::args().skip(1);
    let (history, grp) = def::CMD.supplement(args).unwrap();
    let ready = match grp {
        CompletionGroup::Ready(ready) => {
            // The easy path. No custom logic needed.
            // e.g. Completing a subcommand or flag, like `git chec<TAB>`
            // or completing something with candidate values, like `ls --color=<TAB>`
            ready
        }
        CompletionGroup::Unready { unready, id, value } => {
            // The hard path. You should write completion logic for each possible variant.
            match id {
                id!(def git_dir) => {
                    let comps: Vec<Completion> = complete_git_dir(history, value);
                    unready.to_ready(comps)
                }
                id!(def log commit) => {
                    unimplemented!("logic for `git log <TAB>`");
                }
                _ => unimplemented!("Some more custom logic...")
            }
        }
    };

    // Print fish-style completion to stdout.
    ready.print(Shell::Fish, &mut std::io::stdout()).unwrap()
}
```

### Install to system

After implementing everything, compile it to binary file, and create a shell completion script to tell the shell how to use the binary.
Here's an example for [Fish](https://github.com/fish-shell/fish-shell) shell:

```fish
# Put this to /usr/share/fish/completions/git.fish or  ~/.config/fish/completions/git.fish

function __do_completion
    set cmd (commandline -c)
    set cmd_arr (string split ' ' $cmd)
    if [ -z "$cmd_arr[-1]" ]
        # preserve the last white space
        echo $cmd "''" | xargs path/to/your/binary
    else
        echo $cmd | xargs path/to/your/binary
    end
end

complete -k -c git -x -a "(__do_completion)"
```

The scripts for all supported shells can be found in [supplement-example/shell](supplement-example/shell).

A complete example can be found in [supplement-example](supplement-example).
