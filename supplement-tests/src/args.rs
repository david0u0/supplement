use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
pub struct Arg {
    #[clap(long, global = true)]
    pub flag3: Option<std::path::PathBuf>,

    #[clap(long, global = true)]
    pub git_dir: Option<std::path::PathBuf>,

    #[clap(long)]
    pub external: Option<String>,

    #[clap(subcommand)]
    pub sub: SubCommand,
}
#[derive(Parser, Debug)]
pub enum SubCommand {
    Bisect {
        arg: Bisect,
    },
    Checkout {
        #[clap(long)]
        flag1: Option<String>, // ignored
        file_or_commit: Option<String>,
        files: Vec<std::path::PathBuf>,
    },
    #[clap(about = "log")]
    Log {
        #[clap(long)]
        graph: bool,
        #[clap(long, num_args = 0..=1, default_value = None, default_missing_value = "full", require_equals = true)]
        pretty: Option<Pretty>,
        commit: Option<String>,

        #[clap(long)]
        flag1: Option<String>,
        #[clap(long)]
        flag2: bool, // ignored
    },
    #[clap(about = "log")]
    IgnoredCmd {
        arg: Option<String>,
    },

    Remote {
        #[clap(subcommand)]
        sub: Option<Remote>,
    },

    #[clap(external_subcommand)]
    Other(#[allow(unused)] Vec<String>),
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

#[derive(Parser, Debug)]
pub enum Remote {
    Add {
        #[clap(long)]
        tags: bool,
        name: String,
    },
    Remove,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Bisect {
    Good,
    Bad,
}
