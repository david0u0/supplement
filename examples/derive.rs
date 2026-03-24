#![feature(more_qualified_paths)] // TODO: remove this when it's stable

#[cfg(feature = "clap-3")]
use clap3 as clap;
#[cfg(feature = "clap-4")]
use clap4 as clap;

use clap::Parser;
use std::io::stdout;
use std::path::{Path, PathBuf};
use std::process::Command;
use supplement::{Completion, CompletionGroup, Seen, Shell, Supplement, helper::id_derived as id};

mod args {
    // Here list some not-so-trivial stuff in the definition, and they're all supported by `supplement`.
    // - `--git-dir` is global
    // - `files` in `checkout` can be infinitely long
    // - `--color` and `--pretty` in `log` have *possible values*. For example, `color` should be one of `auto` `always` or `never`
    // - `--pretty` in `log` has `num_args = 0..=1, default_value = "short, default_missing_value = "full", require_equals = true`... which just means it behaves differently with or without the equal sign
    //     + `qit log` is the same as `qit log --pretty=short`
    //     + `qit log --pretty` is the same as `qit log --pretty=full`
    //     + `qit log --pretty full` will not supply the argument `full` to `--pretty`

    use super::*;
    use clap::{Parser, ValueEnum};

    #[derive(Parser, Debug, Supplement)]
    pub struct Git {
        #[clap(long)] // TODO: make this global
        pub git_dir: Option<PathBuf>,
        #[clap(subcommand)]
        pub sub: Sub,
    }
    #[derive(Parser, Debug, Supplement)]
    pub enum Sub {
        Checkout {
            file_or_commit: Option<String>,
            files: Vec<PathBuf>,
        },

        #[clap(about = "log")]
        Log {
            #[clap(short, long)]
            graph: bool,
            #[clap(short, long, num_args = 0..=1, default_value = "short", default_missing_value = "full", require_equals = true, value_enum)]
            pretty: Option<Pretty>,
            #[clap(short, long, default_value = "auto", value_enum)]
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
}

use args::*;
type ID = <Git as Supplement>::ID;

fn run_git(args: &str) -> String {
    let out = Command::new("git")
        .args(args.split(" "))
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(out).unwrap()
}

fn main() {
    env_logger::init();
    let args: Vec<_> = std::env::args().collect();
    log::info!("args = {:?}", args);

    let shell: Result<Shell, _> = args.get(1).map(String::as_str).unwrap_or_default().parse();
    match shell {
        Err(_) => {
            log::info!("Mode #1: parse");
            let res = Git::try_parse_from(args);
            match res {
                Ok(res) => println!("{:?}", res),
                Err(err) => println!("{err}"),
            }
        }
        Ok(shell) => {
            log::info!("Mode #2: completion");
            let args = args[2..].iter().map(String::from);
            let (seen, grp) = Git::supplement(args).unwrap();
            let ready = match grp {
                CompletionGroup::Ready(r) => {
                    // The easy path. No custom logic needed.
                    // e.g. Completing a subcommand or flag, like `git chec<TAB>`
                    // or completing something with candidate values, like `ls --color=<TAB>`
                    r
                }
                CompletionGroup::Unready { unready, id, value } => {
                    let comps = handle_comp(id, &seen, &value);
                    unready.to_ready(comps)
                }
            };
            ready.print(shell, &mut stdout()).unwrap();
        }
    }
}

fn handle_comp(id: ID, seen: &Seen, _value: &str) -> Vec<Completion> {
    match id {
        id!(Git.git_dir) => std::process::exit(1), // Exit to use default completion
        id!(Git.sub Sub.Checkout.file_or_commit) => {
            // For the first argument, it can either be a git commit or a file
            let mut comps = vec![];
            for line in run_git("log --oneline -10").lines() {
                let (hash, description) = line.split_once(" ").unwrap();
                comps.push(Completion::new(hash, description).group("Commits"));
            }
            for line in run_git("status --porcelain").lines() {
                let (_, file) = line.rsplit_once(" ").unwrap();
                comps.push(Completion::new(file, "").group("Modified file"));
            }
            comps
        }
        id!(Git.sub(root_ctx) Sub.Checkout.files(chk_ctx)) => {
            // For the second and more arguments, it can only be file
            // Let's also filter out those files we've already seen!
            let _git_dir: Option<&Path> = root_ctx.git_dir(seen); // This is only for demo
            let prev1: Option<&str> = chk_ctx.file_or_commit(seen);
            let prev2 = chk_ctx.files(seen); // impl Iterator<Item = &Path>
            let prev: Vec<&str> = prev1
                .into_iter()
                .chain(prev2.filter_map(|p: &Path| p.to_str()))
                .collect();

            run_git("status --porcelain")
                .lines()
                .filter_map(|line| {
                    let (_, file) = line.rsplit_once(" ").unwrap();
                    if prev.contains(&file) {
                        None
                    } else {
                        Some(Completion::new(file, "").group("Modified file"))
                    }
                })
                .collect()
        }
        id!(Git.sub Sub.Log.commit(log_ctx)) => {
            let pretty: Option<Result<Pretty, String>> = log_ctx.pretty(seen);

            // let's say, if pretty is "oneline", we search for all commits
            let cmd = if pretty == Some(Ok(Pretty::Oneline)) {
                "log --oneline"
            } else {
                "log --oneline -10"
            };
            run_git(cmd)
                .lines()
                .map(|line| {
                    let (hash, description) = line.split_once(" ").unwrap();
                    Completion::new(hash, description).group("Commits")
                })
                .collect()
        }
    }
}
