# Supplement
> Shell-agnostic, extensible CLI completion for Rust 💊

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
Say you have this awesome clap definition, and want to use supplement to make it even more awesome.

```rs
use clap::{CommandFactory, Parser, ValueEnum};

#[derive(Parser, Debug)]
pub struct Git {
    #[clap(long)]
    pub git_dir: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    pub sub: SubCommand,
}
// more definition...
```

You can now edit `build.rs` to generate the scaffold code (see [supplement-example/build.rs](supplement-example/build.rs)), and in `main.rs`, utilize the generated code like this:

```rs
use supplement::*;

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
                id_enum!(def git_dir) => {
                    let comps: Vec<Completion> = complete_git_dir(history, value);
                    unready.to_ready(comps)
                }
                id_enum!(def remote set_url name) => {
                    unimplemented!("logic for `git remote set-url <TAB>`");
                }
                _ => unimplemented!("Some more custom logic...")
            }
        }
    };

    // Print fish-style completion to stdout.
    ready.print(Shell::Fish, &mut std::io::stdout()).unwrap()
}
```

Note that, if you missed some implementation, it's a *compile time error*. So just relex and let Rust get your back 💪

And after implementing everything, compile it to binary file and create a shell completion file to tell the shell how to use the binary. For example, in `fish` shell you should have:

```fish
# Put this to /usr/share/fish/completions/git.fish or  ~/.config/fish/completions/git.fish

function __do_completion
    set cmd (commandline -j)
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
