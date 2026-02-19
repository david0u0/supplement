use clap3 as clap;

use clap::Parser;

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
        #[clap(long)]
        graph: bool,
        #[clap(long, possible_values(&["oneline", "short", "full"]))]
        pretty: Option<String>,
        commit: Option<String>,
    },
}
