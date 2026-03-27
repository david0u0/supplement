use clap::{Parser, ValueEnum};
use clap4 as clap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use supplement::{Seen, Supplement, helper::id_no_assoc as id};

#[derive(Parser, Debug, Supplement)]
#[clap(version)]
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
    #[clap(value_enum)]
    test_value_enum_iter: Vec<Pretty>,
}

#[derive(Parser, Debug, Clone, Supplement)]
pub enum Sub {
    Log {
        #[clap(long, value_enum)]
        pretty: Option<Pretty>, // NOTE: the `value_enum` is necessary due to lack of specialization
        commit: Commit,
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

    // To test the Pacal => camal logic
    CherryPick {
        commit: String,
    },

    RM {
        paths: Vec<PathBuf>,
    },

    #[clap(external_subcommand)]
    Other(Vec<String>),
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

#[derive(Clone, Debug)]
pub struct Commit(pub String);
impl FromStr for Commit {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Commit(s.to_string()))
    }
}

#[derive(Parser, Debug, Clone, Supplement)]
pub enum Remote {
    #[clap(name = "add")]
    MyAdd {
        url: Vec<URL>,
    },
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
    let _: Option<&str> = id.git_dir(seen); // ID implements Deref<Target = Accessor>

    match id {
        id!(GitID.git_dir(acc)) => {
            let _: Option<&str> = acc.git_dir(seen);
        }
        id!(GitID.sub(acc) SubID.Remote1.sub(remote_acc) RemoteID.MyAdd.url(add_acc)) => {
            let _: Option<&str> = acc.git_dir(seen);
            let _: u32 = remote_acc.v(seen);
            let _: u32 = remote_acc.opts.verbose(seen);
            let _: Option<&str> = remote_acc.opts.test_flat(seen);
            let _: Vec<Result<Pretty, _>> = remote_acc.opts.test_value_enum_iter(seen).collect();
            let _: Vec<Result<URL, _>> = add_acc.url(seen).collect();
        }
        id!(GitID.sub SubID.Remote2 RemoteStructID.sub(remote_acc) RemoteID.MyAdd.url(add_acc)) => {
            let _: u32 = remote_acc.v(seen);
            let _: u32 = remote_acc.opts.verbose(seen);
            let _: Option<&str> = remote_acc.opts.test_flat(seen);
            let _: Vec<Result<URL, _>> = add_acc.url(seen).collect();
        }

        id!(GitID.sub SubID.Remote1.opts(remote_acc) TestFlatID.test_flat(flat_acc)) => {
            let _: u32 = remote_acc.v(seen);
            assert_eq!(remote_acc.opts, flat_acc);
            let _: u32 = flat_acc.verbose(seen);
            let _: Option<&str> = flat_acc.test_flat(seen);
        }
        id!(GitID.sub SubID.Remote2 RemoteStructID.opts(remote_acc) TestFlatID.test_flat(flat_acc)) =>
        {
            let _: u32 = remote_acc.v(seen);
            assert_eq!(remote_acc.opts, flat_acc);
            let _: u32 = flat_acc.verbose(seen);
            let _: Option<&str> = flat_acc.test_flat(seen);
        }

        id!(GitID.sub SubID.Log.paths(log_acc)) => {
            let _: Vec<&Path> = log_acc.paths(seen).collect();
            let _: Option<Result<Pretty, _>> = log_acc.pretty(seen);
            let _: Option<Result<Commit, _>> = log_acc.commit(seen);
        }
        id!(GitID.sub SubID.RM.paths) => {}
        id!(GitID.sub(acc) SubID.Log.commit) | id!(GitID.sub(acc) SubID.CherryPick.commit) => {
            let _: Option<&str> = acc.git_dir(seen);
        }

        id!(GitID.sub SubID.Other(ext_acc)) => {
            let _: Vec<&str> = ext_acc.values(seen).collect();
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
                    RemoteStructID::X3Xsub(def(), RemoteID::X5XMyAddurl(def(), ()))
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
            RemoteID::X5XMyAddurl(def(), ())
        );

        assert_eq!(
            from_cmd::<RemoteStruct>(&["add", "url"]),
            RemoteStructID::X3Xsub(def(), RemoteID::X5XMyAddurl(def(), ()))
        );

        assert_eq!(Git::id_from_cmd(&["nonexistent"]), None);
    }

    #[test]
    fn test_gen_cmd() {
        let _ = env_logger::try_init();

        let _cmd = Git::gen_cmd();
    }
}
