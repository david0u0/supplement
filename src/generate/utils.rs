use clap::{Arg, Command, builder::StyledStr};

pub(super) fn flags(p: &Command) -> impl Iterator<Item = &Arg> {
    p.get_arguments()
        .filter(|a| !a.get_num_args().expect("built").takes_values() && !a.is_positional())
}

pub(super) fn escape_help(help: &StyledStr) -> String {
    help.to_string().replace('\n', " ")
}
