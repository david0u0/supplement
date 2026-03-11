use supplement_tests::{args::Arg, my_gen::my_gen};

fn main() {
    my_gen::<Arg>(&mut std::io::stdout()).unwrap();
}
