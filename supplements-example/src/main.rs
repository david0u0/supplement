#[cfg(feature = "clap-3")]
use clap3 as clap;
#[cfg(feature = "clap-4")]
use clap4 as clap;

use clap::{CommandFactory, Parser};
use std::io::stdout;
use std::process::Command;
use supplements::{Completion, History, Shell, generate, generate_default};
use supplements_example::args::Git;

mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}

use def::Supplements;

fn run_git(args: &str) -> String {
    let out = Command::new("git")
        .args(args.split(" "))
        .output()
        .unwrap()
        .stdout;
    String::from_utf8(out).unwrap()
}
impl def::FlagGitDir for Supplements {} // default implementation
impl def::cmd_checkout::ArgFileOrCommit for Supplements {
    /// For the first argument, it can either be a git commit or a file
    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
        let mut ret = vec![];
        for line in run_git("log --oneline -10").lines() {
            let (hash, description) = line.split_once(" ").unwrap();
            ret.push(Completion::new(hash, description).group("Commits"));
        }
        for line in run_git("status --porcelain").lines() {
            let (_, file) = line.rsplit_once(" ").unwrap();
            ret.push(Completion::new(file, "").group("Modified file"));
        }
        ret
    }
}
impl def::cmd_checkout::ArgFiles for Supplements {
    /// For the second and more arguments, it can only be file
    /// Let's also filter out those files we've already seen!
    fn comp_options(history: &History, _arg: &str) -> Vec<Completion> {
        let prev1 = history
            .find(def::cmd_checkout::ID_ARG_FILES)
            .into_iter()
            .flat_map(|x| x.values.iter());
        let prev2 = history
            .find(def::cmd_checkout::ID_ARG_FILE_OR_COMMIT)
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
}
impl def::cmd_log::ArgCommit for Supplements {
    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
        run_git("log --oneline -10")
            .lines()
            .map(|line| {
                let (hash, description) = line.split_once(" ").unwrap();
                Completion::new(hash, description).group("Commits")
            })
            .collect()
    }
}

fn main() {
    env_logger::init();

    let args: Vec<_> = std::env::args().collect();
    log::info!("args = {:?}", args);

    if args.len() == 2 && args[1] == "generate" {
        generate(&mut Git::command(), Default::default(), &mut stdout()).unwrap();
        generate_default(&mut Git::command(), Default::default(), &mut stdout()).unwrap();
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

    let shell: Shell = args.get(1).unwrap().parse().unwrap();

    let args = args[2..].iter().map(String::from);
    let comps = def::CMD.supplement(args).unwrap();
    comps.print(shell, &mut std::io::stdout()).unwrap();
}
