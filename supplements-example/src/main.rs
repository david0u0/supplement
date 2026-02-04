use clap::{CommandFactory, Parser};
use supplements::{Completion, History};
use supplements_example::args::Root;

mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}

use def::Supplements;

impl def::FlagYetAnotherTest for Supplements {
    fn comp_options(history: &History, _arg: &str) -> Vec<Completion> {
        if let Some(last) = history.find_last(Self::ID) {
            let last: u32 = last.value.parse().unwrap();
            return vec![Completion::new(&(last + 1).to_string(), "")];
        }
        vec![]
    }
}
impl def::sub2::ArgArgTestOpt for Supplements {
    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
        vec![
            Completion::new("opt-value-1", ""),
            Completion::new("opt-value-2", ""),
        ]
    }
}
impl def::sub2::ArgArgTestVec for Supplements {
    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
        vec![
            Completion::new("vec-value-1", ""),
            Completion::new("vec-value-2", ""),
        ]
    }
}
impl def::External for Supplements {
    fn comp_options(history: &History, _arg: &str) -> Vec<Completion> {
        if history.find(Self::ID).is_some() {
            return vec![Completion::new("external-arg", "")];
        }
        vec![
            Completion::new("external-1", ""),
            Completion::new("external-2", ""),
        ]
    }
}

fn main() {
    env_logger::init();

    let args: Vec<_> = std::env::args().collect();
    log::info!("args = {:?}", args);

    if args.len() == 2 && args[1] == "generate" {
        supplements::generate(&mut Root::command(), &mut std::io::stdout()).unwrap();
        return;
    }

    if args.get(1).map(|s| s.as_str()) == Some("parse") {
        let res = Root::try_parse_from(args[1..].iter());
        match res {
            Ok(res) => println!("{:?}", res),
            Err(err) => println!("{err}"),
        }
        return;
    }

    let mut args = args.into_iter();
    args.next();
    let res = def::CMD.supplement(args, false).unwrap();
    for c in res.iter() {
        println!("{}\t{}", c.value, c.description);
    }
}
