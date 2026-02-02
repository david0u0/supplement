use clap::{CommandFactory, Parser};
use supplements::{Completion, History};
use supplements_example::args::Root;

mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}

struct Dummy;

impl def::FlagYetAnotherTest for Dummy {}
impl def::Cmd for Dummy {
    type IFlagYetAnotherTest = Dummy;
    type ICmdSub1 = Dummy;
    type ICmdSub2 = Dummy;
}
impl def::sub1::Cmd for Dummy {}
impl def::sub2::ArgSubTest for Dummy {
    fn comp_options(_history: &History, _arg: &str) -> Vec<Completion> {
        vec![
            Completion::new("arg-value-1", ""),
            Completion::new("arg-value-2", ""),
        ]
    }
}
impl def::sub2::Cmd for Dummy {
    type IArgSubTest = Dummy;
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

    let res = <Dummy as def::Cmd>::generate().supplement(args.into_iter(), false);
    println!("{:?}", res);
}
