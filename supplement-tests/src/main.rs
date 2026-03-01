use supplement_tests::args::Arg;
use supplement_tests::generate::my_gen;

fn main() {
    env_logger::init();
    let mut v: Vec<u8> = vec![];
    my_gen::<Arg>(&mut v).unwrap();
}
