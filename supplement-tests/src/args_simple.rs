use clap::{Parser, ValueEnum};
use std::str::FromStr;
use supplement::{Supplement, helper::id_derived as id};

#[derive(Parser, Debug, Supplement)]
pub struct Git {
    #[clap(long)]
    git_dir: Option<String>,
    #[clap(subcommand)]
    sub: Sub,
}

#[derive(Parser, Debug, Clone, Supplement)]
pub enum Sub {
    Log {
        #[clap(long)]
        pretty: Option<Pretty>,
        paths: Vec<Path>,
    },
    Remote1 {
        #[clap(long)]
        verbose: bool,
        #[clap(subcommand)]
        sub: Remote,
    },
    Remote2(RemoteStruct),
}

#[derive(Parser, Debug, Clone, Supplement)]
pub struct RemoteStruct {
    #[clap(long)]
    verbose: bool,
    #[clap(subcommand)]
    sub: Remote,
}

#[derive(Clone, Debug)]
pub struct URL(pub String);
impl FromStr for URL {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(URL(s.to_string()))
    }
}
#[derive(Clone, Debug)]
pub struct Path(pub String);
impl FromStr for Path {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Path(s.to_string()))
    }
}

#[derive(Parser, Debug, Clone, Supplement)]
pub enum Remote {
    Add { url: URL },
    Delete,
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

pub fn handle_id(id: <Git as Supplement>::ID) {
    match id {
        id!(Git.git_dir(ctx)) => {
            let _: Option<String> = ctx.git_dir();
        }
        id!(Git.sub(ctx) Sub.Remote1.verbose(remote_ctx)) => {
            let _: Option<String> = ctx.git_dir();
            let _: u32 = remote_ctx.verbose();
        }
        id!(Git.sub(ctx) Sub.Remote2 RemoteStruct.verbose(remote_ctx)) => {
            let _: Option<String> = ctx.git_dir();
            let _: u32 = remote_ctx.verbose();
        }
        id!(Git.sub(ctx) Sub.Remote1.sub(remote_ctx) Remote.Add.url(add_ctx)) => {
            let _: Option<String> = ctx.git_dir();
            let _: u32 = remote_ctx.verbose();
            let _: Option<URL> = add_ctx.url();
        }
        id!(Git.sub Sub.Remote2 RemoteStruct.sub(remote_ctx) Remote.Add.url(add_ctx)) => {
            let _: u32 = remote_ctx.verbose();
            let _: Option<URL> = add_ctx.url();
        }

        id!(Git.sub Sub.Log.paths(log_ctx)) => {
            let _: Vec<Path> = log_ctx.paths().collect();
            let _: Option<Pretty> = log_ctx.pretty();
        }
        id!(Git.sub Sub.Log.pretty(log_ctx)) => {
            let _: Vec<Path> = log_ctx.paths().collect();
            let _: Option<Pretty> = log_ctx.pretty();
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;
    type GitID = <Git as Supplement>::ID;
    type SubID = <Sub as Supplement>::ID;
    type RemoteID = <Remote as Supplement>::ID;
    type RemoteStructID = <RemoteStruct as Supplement>::ID;

    fn def<T: Default>() -> T {
        T::default()
    }

    #[test]
    fn test_id_from_cmd() {
        assert_eq!(
            Git::id_from_cmd(&["git_dir"]).unwrap(),
            GitID::X7Xgit_dir(def())
        );

        assert_eq!(
            Git::id_from_cmd(&["remote1", "verbose"]).unwrap(),
            GitID::X3Xsub(def(), SubID::X7XRemote1verbose(def()))
        );

        assert_eq!(
            Git::id_from_cmd(&["remote2", "add", "url"]).unwrap(),
            GitID::X3Xsub(
                def(),
                SubID::X7XRemote2(
                    (),
                    RemoteStructID::X3Xsub(def(), RemoteID::X3XAddurl(def()))
                )
            )
        );

        assert_eq!(
            Remote::id_from_cmd(&["add", "url"]).unwrap(),
            RemoteID::X3XAddurl(def())
        );

        assert_eq!(
            RemoteStruct::id_from_cmd(&["add", "url"]).unwrap(),
            RemoteStructID::X3Xsub(def(), RemoteID::X3XAddurl(def()))
        );

        assert_eq!(Git::id_from_cmd(&["nonexistent"]), None);
    }
}
