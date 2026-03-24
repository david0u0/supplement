use clap::{Parser, ValueEnum};
use clap4 as clap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use supplement::{Seen, Supplement, helper::id_no_assoc as id};

#[derive(Parser, Debug, Supplement)]
pub struct Git {
    #[clap(long, global = true)]
    git_dir: Option<String>,
    #[clap(subcommand)]
    sub: Sub,
}

#[derive(Parser, Debug, Clone, Supplement)]
pub struct TestFlat {
    #[clap(long)]
    verbose: bool,
    #[clap(long)]
    test_flat: Option<String>,
}

#[derive(Parser, Debug, Clone, Supplement)]
pub enum Sub {
    Log {
        #[clap(long)]
        exclude: Option<String>,
        #[clap(long, value_enum)]
        pretty: Option<Pretty>, // NOTE: the `value_enum` is necessary due to lack of specialization
        paths: Vec<PathBuf>,
    },
    Remote1 {
        #[clap(short)]
        v: bool,
        #[clap(flatten)]
        opts: TestFlat,
        #[clap(subcommand)]
        sub: Remote,
    },
    Remote2(RemoteStruct),
}

#[derive(Parser, Debug, Clone, Supplement)]
pub struct RemoteStruct {
    #[clap(short)]
    v: bool,
    #[clap(flatten)]
    opts: TestFlat,
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

// TODO: use `id` once `more_qualified_paths` becomes stable
type GitID = <Git as Supplement>::ID;
type SubID = <Sub as Supplement>::ID;
type RemoteID = <Remote as Supplement>::ID;
type RemoteStructID = <RemoteStruct as Supplement>::ID;
type TestFlatID = <TestFlat as Supplement>::ID;

pub fn handle_id(seen: &Seen, id: <Git as Supplement>::ID) {
    match id {
        id!(GitID.git_dir(ctx)) | id!(GitID.sub(ctx) SubID.Log.exclude) => {
            let _: Option<&str> = ctx.git_dir(seen);
        }
        id!(GitID.sub(ctx) SubID.Remote1.sub(remote_ctx) RemoteID.Add.url(add_ctx)) => {
            let _: Option<&str> = ctx.git_dir(seen);
            let _: u32 = remote_ctx.v(seen);
            let _: u32 = remote_ctx.opts.verbose(seen);
            let _: Option<&str> = remote_ctx.opts.test_flat(seen);
            let _: Option<Result<URL, _>> = add_ctx.url(seen);
        }
        id!(GitID.sub SubID.Remote2 RemoteStructID.sub(remote_ctx) RemoteID.Add.url(add_ctx)) => {
            let _: u32 = remote_ctx.v(seen);
            let _: u32 = remote_ctx.opts.verbose(seen);
            let _: Option<&str> = remote_ctx.opts.test_flat(seen);
            let _: Option<Result<URL, _>> = add_ctx.url(seen);
        }

        id!(GitID.sub SubID.Remote1.opts(remote_ctx) TestFlatID.test_flat(flat_ctx)) => {
            let _: u32 = remote_ctx.v(seen);
            assert_eq!(remote_ctx.opts, flat_ctx);
            let _: u32 = flat_ctx.verbose(seen);
            let _: Option<&str> = flat_ctx.test_flat(seen);
        }
        id!(GitID.sub SubID.Remote2 RemoteStructID.opts(remote_ctx) TestFlatID.test_flat(flat_ctx)) =>
        {
            let _: u32 = remote_ctx.v(seen);
            assert_eq!(remote_ctx.opts, flat_ctx);
            let _: u32 = flat_ctx.verbose(seen);
            let _: Option<&str> = flat_ctx.test_flat(seen);
        }

        id!(GitID.sub SubID.Log.paths(log_ctx)) => {
            let _: Vec<&Path> = log_ctx.paths(seen).collect();
            let _: Option<Result<Pretty, _>> = log_ctx.pretty(seen);
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    fn def<T: Default>() -> T {
        T::default()
    }

    fn from_cmd_root(cmd: &[&str]) -> GitID {
        from_cmd::<Git>(cmd)
    }
    fn from_cmd<T: Supplement>(cmd: &[&str]) -> T::ID {
        let (id, num) = T::id_from_cmd(cmd).unwrap();
        log::debug!("from_cmd result: {id:?} {num}");
        id.unwrap()
    }

    #[test]
    fn test_id_from_cmd() {
        let _ = env_logger::try_init();

        assert_eq!(from_cmd_root(&["git_dir"]), GitID::X7Xgit_dir(def(), ()));

        assert_eq!(
            from_cmd_root(&["remote2", "add", "url"]),
            GitID::X3Xsub(
                def(),
                SubID::X7XRemote2(
                    (),
                    RemoteStructID::X3Xsub(def(), RemoteID::X3XAddurl(def(), ()))
                )
            )
        );

        assert_eq!(
            from_cmd_root(&["remote2", "test_flat"]),
            GitID::X3Xsub(
                def(),
                SubID::X7XRemote2(
                    (),
                    RemoteStructID::X4Xopts(def(), TestFlatID::X9Xtest_flat(def(), ()))
                )
            )
        );

        assert_eq!(
            from_cmd::<Remote>(&["add", "url"]),
            RemoteID::X3XAddurl(def(), ())
        );

        assert_eq!(
            from_cmd::<RemoteStruct>(&["add", "url"]),
            RemoteStructID::X3Xsub(def(), RemoteID::X3XAddurl(def(), ()))
        );

        assert_eq!(Git::id_from_cmd(&["nonexistent"]), None);
    }

    #[test]
    fn test_gen_cmd() {
        let _ = env_logger::try_init();

        let _cmd = Git::gen_cmd();
    }
}
