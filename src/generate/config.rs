use super::Trace;
use crate::error::GenerateError;
use std::collections::HashMap;

/// An object to configure how the code-gen should work
/// e.g. to ignore some certain flags or subcommands.
/// ```no_run
/// # use supplement::{Config, generate};
/// # #[cfg(feature = "clap-3")]
/// # use clap3 as clap;
/// # #[cfg(feature = "clap-4")]
/// # use clap4 as clap;
///
/// let config: Config = Default::default();
/// let mut cmd = clap::Command::new("git");
/// generate(&mut cmd, config, &mut std::io::stdout()).unwrap();
/// ```
#[derive(Clone)]
pub struct Config {
    ignore: HashMap<Vec<String>, bool>,
    strict: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
impl Config {
    pub fn new() -> Self {
        Config {
            strict: true,
            ignore: Default::default(),
        }
    }
    /// Ignore a certain flag or subcommand during code-gen.
    /// Note that if you want to ignore something that doesn't actually exist in the command definition,
    /// The `generate` function will raise an `UnprocessedConfigObj` error.
    /// ```no_run
    /// # use supplement::Config;
    /// let config = Config::default()
    ///     .ignore(&["log", "pretty"]) // ignore the `git log --pretty` flag
    ///     .ignore(&["branch"]) // ignore the `git branch` command
    ///     .ignore(&["unexpected-thing"]); // will cause error during code-gen
    /// ```
    pub fn ignore(mut self, ids: &[&str]) -> Self {
        self.ignore
            .insert(ids.iter().map(|x| x.to_string()).collect(), false);
        self
    }

    pub(crate) fn is_ignored(&mut self, prev: &[Trace], id: &str) -> bool {
        let mut key: Vec<_> = prev.iter().map(|t| t.cmd_id.to_string()).collect();
        key.push(id.to_string());
        if let Some(t) = self.ignore.get_mut(&key) {
            *t = true;
            true
        } else {
            false
        }
    }
    pub(crate) fn unprocessed_ignore(&self) -> impl Iterator<Item = &[String]> {
        self.ignore.iter().filter_map(|(key, processed)| {
            if *processed {
                None
            } else {
                Some(key.as_slice())
            }
        })
    }

    pub(crate) fn check_unprocessed_config(&self) -> Result<(), GenerateError> {
        let mut it = self.unprocessed_ignore().peekable();
        if it.peek().is_none() {
            return Ok(());
        }

        Err(GenerateError::UnprocessedConfigObj(
            it.map(|x| x.to_vec()).collect(),
        ))
    }
    pub fn strict(mut self, yes: bool) -> Self {
        self.strict = yes;
        self
    }
    pub fn is_strict(&self) -> bool {
        self.strict
    }
}
