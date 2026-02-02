use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ThisTest {
    #[clap(help = "help for p1")]
    P1,
    P2,
}

#[derive(Parser, Debug)]
pub struct Root {
    #[clap(short = 't', long, help = "help for \"t\"")]
    pub this_test: Vec<ThisTest>,
    #[clap(short = 'a', long)]
    pub another_test: bool,
    #[clap(short = 'y', long, overrides_with = "yet_another_test")]
    pub yet_another_test: u32,

    #[clap(subcommand)]
    pub sub: Option<SubCommand>,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    Sub1 {
        #[clap(short = 's', long, help = "help for \"s\"")]
        sub_test: Option<ThisTest>,
    },
    #[clap(about = "help for \"sub2\"")]
    Sub2 { sub_test: String },
}
