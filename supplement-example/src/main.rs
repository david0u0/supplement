#[cfg(feature = "clap-3")]
use clap3 as clap;
#[cfg(feature = "clap-4")]
use clap4 as clap;

use clap::{CommandFactory, Parser};
use std::io::stdout;
use std::process::Command;
use supplement::{Completion, CompletionGroup, History, Shell, generate, helper::id};
use supplement_example::args::Git;

mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}
use def::ID;

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

    if args.len() == 2 && args[1] == "generate" {
        log::info!("Mode #2: generate");
        generate(&mut Git::command(), Default::default(), &mut stdout()).unwrap();
        return;
    }

    let shell: Result<Shell, _> = args.get(1).map(String::as_str).unwrap_or_default().parse();
    match shell {
        Ok(shell) => {
            log::info!("Mode #1: completion");
            let args = args[2..].iter().map(String::from);
            let mut history = Default::default();
            let grp = def::CMD.supplement2(&mut history, args).unwrap();
            let ready = match grp {
                CompletionGroup::Ready(r) => {
                    // The easy path. No custom logic needed.
                    // e.g. Completing a subcommand or flag, like `git chec<TAB>`
                    // or completing something with candidate values, like `ls --color=<TAB>`
                    r
                }
                CompletionGroup::Unready { unready, id, value } => {
                    let comps = handle_comp(id, &value);
                    unready.to_ready(comps)
                }
            };
            ready.print(shell, &mut stdout()).unwrap();
        }
        Err(_) => {
            log::info!("Mode #3: parse");
            let res = Git::try_parse_from(args);
            match res {
                Ok(res) => println!("{:?}", res),
                Err(err) => println!("{err}"),
            }
        }
    }
}

// TODO: This somehow mess with the `(ctx)` part. Try to fix it.
// macro_rules! id {
//     ($($id:tt )+) => {
//         supplement::helper::id!(def $($id )+)
//     }
// }

fn handle_comp(id: ID<&History<ID>>, _value: &str) -> Vec<Completion> {
    match id {
        id!(def git_dir) => std::process::exit(1), // Exit to use default completion
        id!(def checkout file_or_commit) => {
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
        id!(def checkout files(_, ctx)) => {
            // For the second and more arguments, it can only be file
            // Let's also filter out those files we've already seen!
            let prev1: Option<&str> = ctx.file_or_commit();
            let prev2: &[String] = ctx.files();
            let prev: Vec<_> = prev1
                .into_iter()
                .chain(prev2.iter().map(|s| s.as_str()))
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
        id!(def log commit) => run_git("log --oneline -10")
            .lines()
            .map(|line| {
                let (hash, description) = line.split_once(" ").unwrap();
                Completion::new(hash, description).group("Commits")
            })
            .collect(),
    }
}
