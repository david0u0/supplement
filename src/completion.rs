use std::fs;
use std::io::Result as IoResult;
use std::io::Write;
use std::path::{MAIN_SEPARATOR_STR, Path};

/// The object to represent a single completion result.
/// For example, if you type `git <TAB>` in command-line, the result should be:
/// ```no_run
/// # use supplements::Completion;
/// vec![
///     Completion::new("checkout", "description...").group("command"),
///     Completion::new("log", "description...").group("command"),
///     Completion::new("branch", "description...").group("command"),
///     // more subcommands here...
/// ];
/// ```
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Completion {
    pub value: String,
    pub description: String,
    pub group: Option<&'static str>,
}
impl Completion {
    pub fn new(value: &str, description: &str) -> Self {
        Completion {
            value: value.to_owned(),
            description: description.to_owned(),
            group: None,
        }
    }
    pub fn value<F: FnOnce(&str) -> String>(mut self, val: F) -> Self {
        self.value = val(&self.value);
        self
    }
    pub fn group(mut self, group: &'static str) -> Self {
        self.group = Some(group);
        self
    }
    /// Generate completion by file. e.g.
    /// - `ls <TAB>` - everything under current directory
    /// - `ls xyz<TAB>` - everything under current directory
    /// - `ls xyz/<TAB>` - everything under `xyz` directory
    /// - `ls another/xyz<TAB>` - everything under `another` directory
    /// - `ls /xyz<TAB>` - everything under `/` directory
    pub fn files(arg: &str) -> impl Iterator<Item = Completion> {
        let path = Path::new(arg);
        let (arg_dir, dir) = match arg {
            "" => (Path::new(""), Path::new("./")),
            "/" => (Path::new("/"), Path::new("/")),
            _ => {
                let arg_dir = if arg.ends_with(MAIN_SEPARATOR_STR) {
                    // path like xyz/ will have `parent() == Some("")`, but we want Some("xyz")
                    path
                } else {
                    path.parent().unwrap()
                };

                let dir = if arg_dir == Path::new("") {
                    Path::new("./")
                } else {
                    arg_dir
                };
                (arg_dir, dir)
            }
        };
        log::debug!("arg_dir = {:?}, dir = {:?}", arg_dir, dir);
        let paths = match fs::read_dir(dir) {
            Ok(paths) => Some(paths),
            Err(err) => {
                log::warn!("error reading current directory: {:?}", err);
                None
            }
        };

        paths.into_iter().flatten().filter_map(|p| {
            let p = match p {
                Ok(p) => p.path(),
                Err(err) => {
                    log::warn!("error reading current directory: {:?}", err);
                    return None;
                }
            };
            let Some(file_name) = p.file_name() else {
                return None;
            };
            let file_name = arg_dir.join(file_name);
            let trailing = if file_name.is_dir() { "/" } else { "" };
            let file_name = format!("{}{}", file_name.to_string_lossy(), trailing);
            Some(Completion::new(&file_name, ""))
        })
    }
}

/// Enum to represent different shell. Use `str::parse` to create it.
/// ```rust
/// use supplements::Shell;
/// let shell: Shell = "fish".parse().unwrap();
/// assert_eq!(shell, Shell::Fish);
/// ```
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Shell {
    Zsh,
    Fish,
    Bash,
}
impl std::str::FromStr for Shell {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ret = match s {
            "zsh" => Shell::Zsh,
            "fish" => Shell::Fish,
            "bash" => Shell::Bash,
            _ => return Err(format!("Unknown shell {}", s)),
        };

        Ok(ret)
    }
}

/// The object to represent multiple completion results.
/// It's not supposed to be created by user of this library, but instead should only be returned by
/// `Command::supplements` function,
/// and is solely used to print out those completion results.
#[derive(PartialEq, Eq, Debug)]
pub struct Ready {
    arg: String,
    comps: Vec<Completion>,
}
impl Ready {
    #[doc(hidden)]
    pub fn inner(&self) -> (&[Completion], &str) {
        (&self.comps, &self.arg)
    }
    #[doc(hidden)]
    pub fn into_inner(self) -> (Vec<Completion>, String) {
        (self.comps, self.arg)
    }

    pub fn print(&self, shell: Shell, w: &mut impl Write) -> IoResult<()> {
        match shell {
            Shell::Bash => {
                for comp in self.comps.iter() {
                    if !comp.value.starts_with(&self.arg) {
                        continue; // If there are multiple candates, bash will not complete :(
                    }
                    writeln!(w, "{}", comp.value)?; // Bash doesn't allow description
                }
            }
            Shell::Fish => {
                for comp in self.comps.iter() {
                    let desc = match (comp.description.as_str(), comp.group) {
                        ("", None) => "",
                        ("", Some(g)) => g,
                        (desc, _) => desc,
                    };
                    writeln!(w, "{}\t{}", comp.value, desc)?
                }
            }
            Shell::Zsh => {
                let mut groups: Vec<(&str, Vec<&Completion>)> = vec![];
                for comp in self.comps.iter() {
                    let group = comp.group.unwrap_or("option");
                    let (_, group_vec) =
                        if let Some(group_vec) = groups.iter_mut().find(|(s, _)| *s == group) {
                            group_vec
                        } else {
                            groups.push((group, vec![]));
                            groups.last_mut().unwrap()
                        };
                    group_vec.push(comp);
                }

                // Group with fewer count shows first. If the same, sort by group name
                groups.sort_by_key(|(k, v)| (v.len(), *k));

                for (group, comps) in groups.into_iter() {
                    writeln!(w, "{}", group)?;
                    for comp in comps.into_iter() {
                        if comp.description.is_empty() {
                            writeln!(w, "\t{}\t{}", comp.value, comp.value)?
                        } else {
                            writeln!(
                                w,
                                "\t{}\t{} -- {}",
                                comp.value, comp.value, comp.description
                            )?
                        }
                    }
                }
                writeln!(w, "END")?;
            }
        }
        Ok(())
    }
}

/// The object to represent an unready completion results.
/// You can't use it to print out completion directly,
/// but instead should convert it to `Ready` (by `to_ready`) first.
#[derive(PartialEq, Eq, Debug)]
pub struct Unready {
    arg: String,
    #[doc(hidden)]
    pub preexist: Vec<Completion>,
    #[doc(hidden)]
    pub prefix: String,
}
impl Unready {
    pub(crate) fn new(prefix: String, arg: String) -> Self {
        Unready {
            prefix,
            arg,
            preexist: vec![],
        }
    }
    pub(crate) fn preexist(mut self, preexist: Vec<Completion>) -> Self {
        self.preexist = preexist;
        self
    }

    /// Building a `Ready` completion based on the `Unready` one.
    /// You have to provide a vector of `Completion`, which represents you're custom completion logic.
    ///
    /// ```no_run
    /// use supplements::completion::{Ready, Unready, Completion, Shell};
    /// # type ID = u32;
    /// # fn create_unready() -> Unready {
    /// #     unimplemented!()
    /// # }
    ///
    /// let unready: Unready = create_unready();
    ///
    /// // `my_comps` is generated by some logic specific to your APP!
    /// let my_comps = vec![Completion::new("custom-value", "custom-description")];
    ///
    /// let ready: Ready = unready.to_ready(my_comps);
    /// ready.print(Shell::Fish, &mut std::io::stdout()).unwrap();
    /// ```
    ///
    /// NOTE that the end result may not be 100% identical to the vector.
    /// For example, if you're completing a flag value `ls --color=<TAB>`,
    /// the vector provided here should be `[auto, never, always]`.
    /// But due to the nature of completion, the `Ready` object created by this function actually contains
    /// `[--color=auto, --color=never, --color=always]`
    pub fn to_ready(mut self, comps: Vec<Completion>) -> Ready {
        log::info!("to_ready: {:?} with {:?}", self, comps);
        let prefix = self.prefix;
        self.preexist.extend(
            comps
                .into_iter()
                .map(|c| c.value(|c| format!("{prefix}{c}"))),
        );
        Ready {
            arg: self.arg,
            comps: self.preexist,
        }
    }
    pub(crate) fn to_ready_grp<ID>(self, comps: Vec<Completion>) -> CompletionGroup<ID> {
        let ready = self.to_ready(comps);
        CompletionGroup::Ready(ready)
    }
}

/// The object to represent completion results.
/// It's not supposed to be created by user of this library, but instead should only be returned by
/// `Command::supplements` function.
/// ```no_run
/// use supplements::{core::Command, Shell};
/// # type ID = u32;
/// # use supplements::completion::{CompletionGroup, Completion};
/// # fn create_cmd() -> Command<ID> {
/// #     unimplemented!()
/// # }
/// let cmd: Command<ID> = create_cmd();
/// let (_history, grp) = cmd
///     .supplement(["git".to_owned(), "log".to_owned()].into_iter())
///     .unwrap();
/// let ready = match grp {
///     CompletionGroup::Ready(ready) => ready,
///     CompletionGroup::Unready { id, value, unready } => {
///         unready
///             .to_ready(vec![Completion::new("value", "description")])
///     }
/// };
/// ready.print(Shell::Fish, &mut std::io::stdout()).unwrap();
/// ```
#[derive(PartialEq, Eq, Debug)]
pub enum CompletionGroup<ID> {
    Ready(Ready),
    Unready {
        unready: Unready,
        id: ID,
        value: String,
    },
}
impl<ID> CompletionGroup<ID> {
    pub(crate) fn new_ready(comps: Vec<Completion>, arg: String) -> Self {
        CompletionGroup::Ready(Ready { comps, arg })
    }
    pub(crate) fn new_unready(id: ID, prefix: String, arg: String, value: Option<String>) -> Self {
        let value = value.unwrap_or_else(|| arg.clone());
        CompletionGroup::Unready {
            unready: Unready::new(prefix, arg),
            id,
            value,
        }
    }
}
