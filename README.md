# Supplements
> Shell-agnostic, extensible CLI completion for Rust 💊

**supplements** is a Rust library that generates completion scaffold as Rust code.

Give it a [`clap`](https://github.com/clap-rs/clap) object, and instead of spitting out shell files that you later have to manually edit, it spits out Rust! supplements is:
- **Shell-agnostic**
- **Powerful** - Some features are not widely supported in every shell, and `supplements` comes to rescue
- **Stop modifying generated files** - Instead, *extend* it with Rust's trait system
- **Easy to test** - Functions and objects in a modern programming language, instead of some shell script black sorcery.
- **It's Rust 🦀**

## Install
Add one line in Cargo.toml. By default, it uses `clap` 4, but you can make it use `clap` 3 with feature.
```toml
[dependencies]
supplements = "0.1"
# Or, to use clap 3
supplements = { version = "0.1", default-features = false, features = ["clap-3"] }
# Or, disable the code generate feature completely
supplements = { version = "0.1", default-features = false }
```

## Get started
1. Have a `clap` definition
2. Generate the `supplements` definition (preferably in `build.rs`)
3. Compile the binary
4. Put a simple shell script in place to tell the shell how to use your binary

Let's break it down step by step. Say you have this awesome clap definition, and want to use supplements to make it even more awsome.

```rs
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
    Log {
        #[clap(long)]
        graph: bool,
        commit: Option<String>,
    },
}
```

You can now edit the `build.rs` to generate the definition file:

```rs
#[path = "src/args.rs"]
mod args;
use clap::CommandFactory;
use supplements::{generate, generate_default};

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let file = std::path::Path::new(&out_dir).join("definition.rs");
    let mut f = std::fs::File::create(file).unwrap();
    generate(&mut args::Git::command(), Default::default(), &mut f).unwrap();

    // Generate default impl. You may not need this.
    generate_default(&mut args::Git::command(), Default::default(), &mut f).unwrap();
}
```

And use it in `main.rs`:

```rs
mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}
use def::Supplements;

impl def::FlagGitDir for Supplements {} // default completion (with files)
impl def::checkout::ArgFileOrCommit for Supplements {
    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
        unimplemented!(); // your custom completion
    }
}

// Some more custom completion logic...

fn main() {
    // `args` looks like ["supplements-example", "git", "log", "--graph"]
    // so we should skip the first arg
    let args = std::env::args().skip(1);
    let comps = def::CMD.supplement(args).unwrap();
    let shell = supplements::Shell::Fieh; // Assume we only use fish shell
    comps.print(shell, &mut std:io::stdout).unwrap()
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
        echo fish $cmd "''" | xargs path/to/your/binary
    else
        echo fish $cmd | xargs path/to/your/binary
    end
end

complete -k -c git -x -a "(__do_completion)"
```

A complete example can be found in [supplements-example](supplements-example)
