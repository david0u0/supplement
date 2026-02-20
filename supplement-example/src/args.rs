// Here list some not-so-trivial stuff in the definition, and they're all supported by `supplement`.
// - `--git-dir` is global
// - `files` in `checkout` can be infinitely long
// - `--color` and `--pretty` in `log` have *possible values*. For example, `color` should be one of `auto` `always` or `never`
// - `--pretty` in `log` has `num_args = 0..=1, default_value = "short, default_missing_value = "full", require_equals = true`... which just means it behaves differently with or without the equal sign
//     + `qit log` is the same as `qit log --pretty=short`
//     + `qit log --pretty` is the same as `qit log --pretty=full`
//     + `qit log --pretty full` will not supply the argument `full` to `--pretty`

use clap::{Parser, ValueEnum};
use clap4 as clap;

#[derive(Parser, Debug)]
pub struct Git {
    #[clap(long, global = true)]
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
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Pretty {
    #[clap(help = "<sha1> <title line>")]
    Oneline,
    #[clap(help = "<sha1> / <author> / <title line>)")]
    Short,
    #[clap(help = "<sha1> / <author> / <committer> / <title> / <commit msg>")]
    Full,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Color {
    Always,
    Never,
    Auto,
}
