![Supplement](logo.svg)

> Shell-agnostic, extensible CLI completion for Rust 💊

[![Crates.io](https://img.shields.io/crates/v/supplement?style=flat-square)](https://crates.io/crates/supplement)
[![Crates.io](https://img.shields.io/crates/d/supplement?style=flat-square)](https://crates.io/crates/supplement)
![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
[![Build Status](https://img.shields.io/github/actions/workflow/status/david0u0/supplement/ci.yml?branch=master&style=flat-square)](https://github.com/david0u0/supplement/actions/workflows/ci.yml?query=branch%3Amaster)
[![Doc](https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square)](https://docs.rs/supplement)

**supplement** is a Rust library that derives completion scaffolds as Rust code.

Give it a [`clap`](https://github.com/clap-rs/clap) object, and instead of spitting out shell files that you later have to manually edit, it spits out Rust! supplement is:
- **Shell-agnostic**
- **No more generated code** - instead, *derive* them.
- **Easy to test and debug** - Functions and objects in a modern programming language, instead of some shell script black sorcery.
- **Context-aware** - Track the "parsed CLI values" for you so you don't have to.
- **It's Rust 🦀**

## Install
Add one line in Cargo.toml. By default, it uses `clap` 4, but you can make it use `clap` 3 with features.
```toml
[dependencies]
supplement = "0.2"
# Or, to use clap 3
supplement = { version = "0.2", default-features = false, features = ["clap-3"] }
```

## Quick start
Say you have some awesome clap definition, and want to use supplement to make it even more awesome. Derive trait `Supplement` for your definitions.

```rs
// src/args.rs

use clap::{CommandFactory, Parser, ValueEnum};
use supplement::Supplement;

#[derive(Parser, Debug, Supplement)]
pub struct Git {
    #[clap(long, global)]
    pub git_dir: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    pub sub: Sub,
}
#[derive(Parser, Debug, Supplement)]
pub enum Sub {
    Checkout {
        file_or_commit: Option<String>,
        files: Vec<std::path::PathBuf>,
    },

    #[clap(about = "log")]
    Log {
        #[clap(short, long)]
        graph: bool,
        #[clap(short, long, num_args = 0..=1, default_value = "short", default_missing_value = "full", require_equals = true, value_enum)]
        pretty: Option<Pretty>,
        #[clap(short, long, default_value = "auto", value_enum)]
        color: Color,
        commit: Option<String>,
    },
}

// more definition...
```

### Implementation

Call function `Supplement::supplement` and start implementing your completion logic.
Note that, if you missed some scenario, it's a *compile time error*. So just relex and let Rust get your back 💪

```rs
use supplement::{*, helper::id};

fn main() {
    // `args` looks like ["the-binary-name", "git", "log", "--graph"]
    // so we should skip the first arg
    let args = std::env::args().skip(1);
    let (seen, grp) = Git::supplement(args).unwrap();
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
                id!(Git.git_dir) => {
                    let comps: Vec<Completion> = complete_git_dir(seen, value);
                    unready.to_ready(comps)
                }
                id!(Git.sub Sub.Log.commit) => {
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
    set cur "$cmd_arr[-1]"
    if [ -z "$cur" ]
        # preserve the last white space
        echo $cmd "''" | xargs path/to/your/binary
    else
        echo $cmd | xargs path/to/your/binary
    end

    if [ "$status" != "0" ]
        # fall back to default completion
        complete -C "'' $cur"
    end
end

complete -k -c git -x -a "(__do_completion)"
```

The scripts for all supported shells can be found in [examples/shell](examples/shell).

A complete example can be found in [examples/README.md](examples/README.md).
