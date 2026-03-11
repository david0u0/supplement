use super::Trace;
use crate::error::GenerateError;
use std::collections::HashMap;

fn to_trace(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}
fn not_processed(map: HashMap<Vec<String>, MayBeProcessed>) -> impl Iterator<Item = Vec<String>> {
    map.into_iter().filter_map(
        |(key, processed)| {
            if processed.0 { None } else { Some(key) }
        },
    )
}

#[derive(Debug, Clone)]
struct MayBeProcessed(bool);
impl MayBeProcessed {
    fn new() -> Self {
        MayBeProcessed(false)
    }
    fn process(&mut self) {
        self.0 = true;
    }
}

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
    ignore: HashMap<Vec<String>, MayBeProcessed>,
    custom: HashMap<Vec<String>, MayBeProcessed>,
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
            custom: Default::default(),
        }
    }
    /// Ignore a flag or subcommand during code-gen.
    /// Note that if you want to ignore something that doesn't actually exist in the command definition,
    /// The [`crate::generate`] function will raise a [`GenerateError::UnprocessedConfigObj`] error.
    /// ```no_run
    /// # use supplement::Config;
    /// let config = Config::default()
    ///     .ignore(&["log", "pretty"]) // ignore the `git log --pretty` flag
    ///     .ignore(&["branch"]) // ignore the `git branch` command
    ///     .ignore(&["unexpected-thing"]); // will cause error during code-gen
    /// ```
    pub fn ignore(mut self, ids: &[&str]) -> Self {
        self.ignore.insert(to_trace(ids), MayBeProcessed::new());
        self
    }

    pub(crate) fn is_ignored(&mut self, prev: &[Trace], id: &str) -> bool {
        let mut key: Vec<_> = prev.iter().map(|t| t.cmd_id.to_string()).collect();
        key.push(id.to_string());
        if let Some(t) = self.ignore.get_mut(&key) {
            t.process();
            true
        } else {
            false
        }
    }

    /// Make a flag or argument *"custom"* during code-gen.
    /// This means it will give a [`crate::CompletionGroup::Unready`] after completion,
    /// and user can then apply custom logic.
    /// For mor detail, check the doc of [`crate::id`].
    ///
    /// Error when calling [`crate::generate`]:
    /// - If the path doesn't exist, raise a [`GenerateError::UnprocessedConfigObj`] error.
    /// - If it is already custom, raise a [`GenerateError::AlreadyCustom`] error.
    /// - If the flag has no value, raise a [`GenerateError::CustomWithoutValue`] error.
    ///
    /// ```no_run
    /// # use supplement::Config;
    /// let config = Config::default()
    ///     .make_custom(&["log", "pretty"]) // make `git log --pretty` custom
    ///     .make_custom(&["unexpected-thing"]) // Error: path not found
    ///     .make_custom(&["commit", "message"]) // Error: already custom
    ///     .make_custom(&["log", "graph"]); // Error: flag without value
    /// ```
    pub fn make_custom(mut self, ids: &[&str]) -> Self {
        self.custom.insert(to_trace(ids), MayBeProcessed::new());
        self
    }

    pub(crate) fn is_custom(&mut self, prev: &[Trace], id: &str) -> bool {
        let mut key: Vec<_> = prev.iter().map(|t| t.cmd_id.to_string()).collect();
        key.push(id.to_string());
        if let Some(t) = self.custom.get_mut(&key) {
            t.process();
            true
        } else {
            false
        }
    }

    pub(crate) fn check_unprocessed_config(self) -> Result<(), GenerateError> {
        let Config {
            strict: _,
            ignore,
            custom,
        } = self;
        let it = not_processed(ignore).chain(not_processed(custom));

        let mut it = it.peekable();
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
