mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}

struct Dummy;

impl def::FlagThisTest for Dummy {}
impl def::Cmd for Dummy {
    type IFlagThisTest = Dummy;
}

fn main() {
    env_logger::init();

    let args: Vec<_> = std::env::args().collect();
    log::info!("args = {:?}", args);

    let res = <Dummy as def::Cmd>::generate().supplement(args.into_iter(), false);
    println!("{:?}", res);
}
