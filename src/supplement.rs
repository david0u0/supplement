use crate::clap::{Command as ClapCommand, CommandFactory};
use crate::core::Command;
use std::fmt::Debug;

pub trait Supplement: CommandFactory {
    type ID: Debug;
    fn id_from_cmd(cmd: &[&str]) -> Option<(Option<Self::ID>, u32)>;
    fn gen_cmd() -> Command<Self::ID> {
        // TODO: try not to rely on CommandFactory?
        gen_cmd::<Self>(Self::command(), vec![])
    }
}

fn gen_cmd<Root: Supplement>(cmd: ClapCommand, trace: Vec<String>) -> Command<Root::ID> {
    panic!()
}
