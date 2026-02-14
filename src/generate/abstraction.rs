#[cfg(all(feature = "clap-3", feature = "clap-4"))]
compile_error!("Only one of the clap features can be enabled");

#[cfg(feature = "clap-3")]
use clap3 as clap;
#[cfg(feature = "clap-4")]
use clap4 as clap;

#[cfg(feature = "clap-3")]
pub type ClapCommand<'a> = &'a mut clap::Command<'static>;
#[cfg(feature = "clap-4")]
pub type ClapCommand<'a> = &'a mut clap::Command;

pub use clap::ArgAction;

pub(crate) struct CommandMut<'a>(pub ClapCommand<'a>);

#[cfg(feature = "clap-3")]
#[derive(Clone, Copy)]
pub(crate) struct Command<'a>(&'a clap::Command<'static>);
#[cfg(feature = "clap-4")]
#[derive(Clone, Copy)]
pub(crate) struct Command<'a>(&'a clap::Command);

#[cfg(feature = "clap-3")]
#[derive(Clone, Copy)]
pub(crate) struct Arg<'a>(&'a clap::Arg<'static>, bool);
#[cfg(feature = "clap-4")]
#[derive(Clone, Copy)]
pub(crate) struct Arg<'a>(&'a clap::Arg);

#[cfg(feature = "clap-3")]
pub type PossibleValue = clap::builder::PossibleValue<'static>;
#[cfg(feature = "clap-4")]
pub type PossibleValue = clap::builder::PossibleValue;

#[cfg(feature = "clap-3")]
pub type Id = str;
#[cfg(feature = "clap-4")]
pub type Id = clap::Id;

impl<'a> CommandMut<'a> {
    pub fn build(&mut self) {
        self.0.build()
    }
    pub fn to_const(self) -> Command<'a> {
        Command(self.0)
    }
}

impl<'a> Command<'a> {
    pub fn get_arguments(&self) -> impl Iterator<Item = Arg<'a>> {
        #[cfg(feature = "clap-3")]
        {
            let positional_count = self.0.get_arguments().filter(|a| a.is_positional()).count();
            let mut cur = 0;
            self.0.get_arguments().map(move |a| {
                if a.is_positional() {
                    cur += 1;
                    let is_last_positional = cur == positional_count;
                    Arg(a, is_last_positional)
                } else {
                    Arg(a, false)
                }
            })
        }
        #[cfg(feature = "clap-4")]
        {
            self.0.get_arguments().map(|a| Arg(a))
        }
    }
    pub fn get_subcommands(&self) -> impl Iterator<Item = Command<'a>> {
        self.0.get_subcommands().map(|c| Command(c))
    }
    pub fn get_name(&self) -> &'a str {
        self.0.get_name()
    }
    pub fn get_about(&self) -> Option<String> {
        self.0.get_about().map(|s| s.to_string())
    }
    pub fn is_allow_external_subcommands_set(&self) -> bool {
        self.0.is_allow_external_subcommands_set()
    }
}

impl<'a> Arg<'a> {
    pub fn is_positional(&self) -> bool {
        self.0.is_positional()
    }
    pub fn get_id(&self) -> &'a Id {
        self.0.get_id()
    }
    pub fn get_max_num_args(&self) -> usize {
        #[cfg(feature = "clap-3")]
        {
            if !self.takes_values() {
                return 0;
            }
            // "get_num_vals" is often None even if it should take a vector,
            // so we have to make some compromise:
            // If this is the last arg, make it infinite long, otherwise make it 1
            let default = if self.1 { std::usize::MAX } else { 1 };
            self.0.get_num_vals().unwrap_or(default)
        }
        #[cfg(feature = "clap-4")]
        {
            self.0.get_num_args().expect("built").max_values()
        }
    }
    pub fn get_min_num_args(&self) -> usize {
        #[cfg(feature = "clap-3")]
        {
            if !self.takes_values() { 0 } else { 1 }
        }
        #[cfg(feature = "clap-4")]
        {
            self.0.get_num_args().expect("built").min_values()
        }
    }
    pub fn is_require_equals_set(&self) -> bool {
        self.0.is_require_equals_set()
    }
    pub fn get_possible_values(&self) -> Vec<PossibleValue> {
        #[cfg(feature = "clap-3")]
        {
            if let Some(pvs) = self.0.get_possible_values() {
                return pvs.to_vec();
            }
        }
        self.0
            .get_value_parser()
            .possible_values()
            .map(|p| p.collect())
            .unwrap_or_default()
    }
    pub fn takes_values(&self) -> bool {
        #[cfg(feature = "clap-3")]
        {
            self.0.is_takes_value_set()
        }
        #[cfg(feature = "clap-4")]
        {
            self.0.get_num_args().expect("built").takes_values()
        }
    }
    pub fn is_global_set(&self) -> bool {
        self.0.is_global_set()
    }
    pub fn get_short_and_visible_aliases(&self) -> Option<Vec<char>> {
        self.0.get_short_and_visible_aliases()
    }
    pub fn get_long_and_visible_aliases(&self) -> Option<Vec<&'a str>> {
        self.0.get_long_and_visible_aliases()
    }
    pub fn get_action(&self) -> &ArgAction {
        self.0.get_action()
    }
    pub fn get_help(&self) -> String {
        self.0.get_help().unwrap_or_default().to_string()
    }
}
