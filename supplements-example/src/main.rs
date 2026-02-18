#[cfg(feature = "clap-3")]
use clap3 as clap;
#[cfg(feature = "clap-4")]
use clap4 as clap;

use clap::{CommandFactory, Parser};
use std::io::stdout;
use std::process::Command;
use supplements::{Completion, CompletionGroup, History, Shell, generate};
use supplements_example::args::Git;

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
        generate(&mut Git::command(), Default::default(), &mut stdout()).unwrap();
        return;
    }

    if args.get(1).map(|s| s.as_str()) == Some("parse") {
        let res = Git::try_parse_from(args[1..].iter());
        match res {
            Ok(res) => println!("{:?}", res),
            Err(err) => println!("{err}"),
        }
        return;
    }

    let shell: Result<Shell, _> = args.get(1).unwrap().parse();
    match shell {
        Err(_) => {
            let res = Git::try_parse_from(args);
            match res {
                Ok(res) => println!("{:?}", res),
                Err(err) => println!("{err}"),
            }
        }
        Ok(shell) => {
            let args = args[2..].iter().map(String::from);
            let mut h = History::new();
            let grp = def::CMD.supplement_with_history(&mut h, args).unwrap();
            let ready = match grp {
                CompletionGroup::Ready(r) => r,
                CompletionGroup::Unready { unready, id, value } => {
                    let comps = handle_comp(h, id, &value);
                    unready.to_ready(comps)
                }
            };
            ready.print(shell, &mut stdout()).unwrap();
        }
    }
}

fn handle_comp(history: History<ID>, id: ID, value: &str) -> Vec<Completion> {
    use def::cmd_checkout::ID as CheckoutID;
    use def::cmd_log::ID as LogID;

    match id {
        ID::GitDir => Completion::files(value).collect(),
        ID::Checkout(CheckoutID::FileOrCommit) => {
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
        ID::Checkout(CheckoutID::Files) => {
            // For the second and more arguments, it can only be file
            // Let's also filter out those files we've already seen!
            let prev1 = history
                .find(&def::cmd_checkout::ID_ARG_FILES)
                .into_iter()
                .flat_map(|x| x.values.iter());
            let prev2 = history
                .find(&def::cmd_checkout::ID_ARG_FILE_OR_COMMIT)
                .map(|x| &x.value);
            let prev: Vec<_> = prev1.chain(prev2.into_iter()).collect();
            run_git("status --porcelain")
                .lines()
                .filter_map(|line| {
                    let (_, file) = line.rsplit_once(" ").unwrap();
                    if prev.iter().any(|p| *p == file) {
                        None
                    } else {
                        Some(Completion::new(file, "").group("Modified file"))
                    }
                })
                .collect()
        }
        ID::Log(LogID::Commit) => run_git("log --oneline -10")
            .lines()
            .map(|line| {
                let (hash, description) = line.split_once(" ").unwrap();
                Completion::new(hash, description).group("Commits")
            })
            .collect(),
        ID::Log(LogID::Color | LogID::Pretty) => {
            unreachable!(); // TODO: don't need id for these
        }
    }
}
