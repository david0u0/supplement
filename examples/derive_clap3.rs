#[cfg(feature = "clap-3")]
use clap3 as clap;
#[cfg(feature = "clap-4")]
use clap4 as clap;

use clap::Parser;
use std::io::stdout;
use std::iter::once;
use std::path::{Path, PathBuf};
use std::process::Command;
use supplement::completion::{Ready, Unready};
use supplement::{Completion, CompletionGroup, Seen, Shell, Supplement, helper::id_no_assoc as id};

mod args {
    // Here list some not-so-trivial stuff in the definition, and they're all supported by `supplement`.
    // - `--git-dir` is global
    // - `files` in `checkout` can be infinitely long
    // - There is an external subcommand `Ext` for alias
    // - `--color` and `--pretty` in `log` have *possible values*. For example, `color` should be one of `auto` `always` or `never`
    // - `--pretty` in `log` has `num_args = 0..=1, default_value = "short, default_missing_value = "full", require_equals = true`... which just means it behaves differently with or without the equal sign
    //     + `qit log` is the same as `qit log --pretty=short`
    //     + `qit log --pretty` is the same as `qit log --pretty=full`
    //     + `qit log --pretty full` will not supply the argument `full` to `--pretty`

    use super::*;
    use clap::Parser;
    pub use clap3 as clap;

    #[derive(Parser, Debug, Supplement)]
    pub struct Git {
        #[clap(long)]
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
            #[clap(long)]
            exclude: Option<String>,
            #[clap(long)]
            graph: bool,
            #[clap(long, possible_values(&["oneline", "short", "full"]))]
            pretty: Option<String>,
            #[clap(long, possible_values(&["always", "never", "auto"]))]
            color: Option<String>,
            commit: Option<String>,
        },

        #[clap(external_subcommand)]
        Ext(#[allow(unused)] Vec<String>),
    }
}

use args::*;

/// NOTE: These can be avoided with `#![feature(more_qualified_paths)]` + [`supplement::helper::id`]
type GitID = <Git as Supplement>::ID;
type SubID = <Sub as Supplement>::ID;

fn run_git(args: &str) -> String {
    let out = Command::new("git")
        .args(args.split(" "))
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(out).unwrap()
}
fn get_commits(limit: usize) -> impl Iterator<Item = Completion> {
    let cmd = format!("log --oneline -{limit}");
    let lines: Vec<_> = run_git(&cmd).lines().map(str::to_string).collect();
    lines.into_iter().map(|l| {
        let (hash, description) = l.split_once(" ").unwrap();
        Completion::new(hash, description).group("Commits")
    })
}
fn get_files() -> impl Iterator<Item = Completion> {
    let cmd = "status --porcelain";
    let lines: Vec<_> = run_git(cmd).lines().map(str::to_string).collect();
    lines.into_iter().map(|l| {
        let (_, file) = l.rsplit_once(" ").unwrap();
        Completion::new(file, "").group("Modified file")
    })
}
fn get_alias() -> Vec<(String, String)> {
    run_git("config --get-regexp ^alias\\.")
        .lines()
        .map(|line| {
            let line = &line["alias.".len()..];
            let (before, after) = line.split_once(" ").unwrap();
            (before.to_string(), after.to_string())
        })
        .collect()
}

fn main() {
    env_logger::init();
    let args: Vec<_> = std::env::args().collect();
    log::info!("args = {:?}", args);

    let shell: Result<Shell, _> = args.get(1).map(String::as_str).unwrap_or_default().parse();
    let Ok(shell) = shell else {
        log::info!("Mode #1: parse");
        let res = Git::try_parse_from(args).unwrap_or_else(|e| panic!("{}", e));
        println!("{:?}", res);
        return;
    };

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
            handle_comp(unready, id, seen, &value, 0)
        }
    };
    ready.print(shell, &mut stdout()).unwrap();
}

fn handle_comp(unready: Unready, id: GitID, mut seen: Seen, val: &str, alias_len: usize) -> Ready {
    let comps = match id {
        id!(GitID.git_dir) | id!(GitID.sub SubID.Log.exclude) => std::process::exit(1), // Exit to use default completion
        id!(GitID.sub SubID.Checkout.file_or_commit) => {
            // For the first argument, it can either be a git commit or a file
            get_commits(10).chain(get_files()).collect()
        }
        id!(GitID.sub(root_accessor) SubID.Checkout.files(chk_accessor)) => {
            // For the second and more arguments, it can only be file
            // Let's also filter out those files we've already seen!
            let _git_dir: Option<&Path> = root_accessor.git_dir(&seen); // This is only for demo
            let prev1: Option<&str> = chk_accessor.file_or_commit(&seen);
            let prev2 = chk_accessor.files(&seen); // impl Iterator<Item = &Path>
            let prev: Vec<&str> = prev1
                .into_iter()
                .chain(prev2.filter_map(|p: &Path| p.to_str()))
                .collect();

            get_files()
                .filter(|comp| !prev.contains(&&*comp.value))
                .collect()
        }
        id!(GitID.sub SubID.Log.commit(log_accessor)) => {
            let pretty: Option<&str> = log_accessor.pretty(&seen);

            // let's say, if pretty is "oneline", we show for more commits
            let limit = if pretty == Some("oneline") { 100 } else { 10 };
            get_commits(limit).collect()
        }
        id!(GitID.sub SubID.Ext(ext_accessor)) if ext_accessor.values(&seen).len() == 0 => {
            // The first external subcommand, show aliases
            get_alias()
                .into_iter()
                .map(|(b, a)| Completion::new(b, a).group("Alias"))
                .collect()
        }
        id!(GitID.sub SubID.Ext(ext_accessor)) => {
            // The arguments of external subcommand
            let mut args = ext_accessor.values(&seen);
            let first = args.next().unwrap();
            if alias_len != 0 {
                let cmd = args.nth(alias_len - 1).unwrap();
                log::error!("Unknown command '{cmd}' when resolving alias '{first}'");
                std::process::exit(1);
            }

            let aliases = get_alias();
            let Some((_, after)) = aliases.iter().find(|a| a.0 == first) else {
                log::error!("unknown alias {first}");
                std::process::exit(1);
            };
            let after = after.split(' ');
            let alias_len = after.clone().count();

            let args = once("qit").chain(after).chain(args).chain(once(val));
            let args: Vec<_> = args.map(String::from).collect();
            log::info!("Begin alias completion with {args:?}");

            // use `supplement_with_seen` to preserve root args/flags
            let grp = Git::gen_cmd()
                .supplement_with_seen(&mut seen, args.into_iter())
                .unwrap();
            return match grp {
                CompletionGroup::Ready(r) => r,
                CompletionGroup::Unready { unready, id, value } => {
                    handle_comp(unready, id, seen, &value, alias_len)
                }
            };
        }

        _ => unimplemented!(),
    };

    unready.to_ready(comps)
}
