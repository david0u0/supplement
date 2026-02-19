#[cfg(all(feature = "clap-3", feature = "clap-4"))]
compile_error!("Only one of the clap features can be enabled");

#[cfg(feature = "clap-4")]
pub mod args;
#[cfg(feature = "clap-3")]
pub mod args_clap3;
#[cfg(feature = "clap-3")]
pub use args_clap3 as args;
