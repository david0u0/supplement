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
    rename_cmd: HashMap<Vec<String>, (bool, String)>,
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
            rename_cmd: Default::default(),
        }
    }
    /// Ignore a certain flag or subcommand during code-gen.
    ///
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

    /// Rename a certain command during code-gen.
    /// This will only change the generated name in rust code, mostly to avoid name confliction with argument.
    /// **It won't change any completion behavior.**
    ///
    /// Note that if you want to rename something that doesn't actually exist in the command definition,
    /// The `generate` function will raise an `UnprocessedConfigObj` error.
    /// ```no_run
    /// # use supplement::Config;
    /// let config = Config::default()
    ///     .rename_cmd(&["log"], "my-log")
    ///     .rename_cmd(&["remote", "set-url"], "my-set-url")
    ///     .rename_cmd(&["unexpected-thing"], "xxx"); // will cause error during code-gen
    /// ```
    pub fn rename_cmd(mut self, ids: &[&str], new_name: &str) -> Self {
        self.rename_cmd.insert(
            ids.iter().map(|x| x.to_string()).collect(),
            (false, new_name.to_string()),
        );
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
    pub(crate) fn get_cmd_name<'a>(&'a mut self, prev: &[Trace], id: &'a str) -> &'a str {
        let mut key: Vec<_> = prev.iter().map(|t| t.cmd_id.to_string()).collect();
        key.push(id.to_string());
        if let Some(t) = self.rename_cmd.get_mut(&key) {
            t.0 = true;
            t.1.as_str()
        } else {
            id
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
    pub(crate) fn unprocessed_rename_cmd(&self) -> impl Iterator<Item = &[String]> {
        self.rename_cmd.iter().filter_map(|(key, (processed, _))| {
            if *processed {
                None
            } else {
                Some(key.as_slice())
            }
        })
    }

    pub(crate) fn check_unprocessed_config(&self) -> Result<(), GenerateError> {
        let it = self
            .unprocessed_ignore()
            .chain(self.unprocessed_rename_cmd());
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
