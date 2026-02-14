use clap4 as clap;

use clap::{Parser, ValueEnum};

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
        #[clap(short, long, num_args = 0..=1, default_value = None, default_missing_value = "full", require_equals = true)]
        pretty: Option<Pretty>,
        #[clap(short, long, num_args = 0..=1, default_value = "auto", default_missing_value = "always", require_equals = true)]
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
