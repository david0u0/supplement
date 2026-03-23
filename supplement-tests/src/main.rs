use supplement::Supplement;
use supplement_tests::{args::Arg, args_simple::Git, my_gen::my_gen};

fn main() {
    my_gen::<Arg>(&mut std::io::stdout()).unwrap();

    println!("{:?}", Git::gen_cmd());
}
