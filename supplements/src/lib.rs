use std::fs;
use std::path::{MAIN_SEPARATOR_STR, Path};

pub mod error;
pub mod id;
pub mod info;

mod generate;
pub mod history;
mod utils;
pub use generate::generate;
pub use history::History;
pub use utils::*;

pub(crate) mod arg_context;
pub(crate) mod parsed_flag;

pub type Result<T = ()> = std::result::Result<T, error::Error>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Completion {
    pub value: String,
    pub description: String,
}
impl Completion {
    pub fn new(value: &str, description: &str) -> Self {
        Completion {
            value: value.to_owned(),
            description: description.to_owned(),
        }
    }
    pub fn files(arg: &str) -> Vec<Self> {
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
            Ok(paths) => paths,
            Err(err) => {
                log::warn!("error reading current directory: {:?}", err);
                return vec![];
            }
        };

        paths
            .filter_map(|p| {
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
                let file_name = file_name.to_string_lossy();
                if file_name.starts_with(arg) {
                    let file_name = format!("{}{}", file_name, trailing);
                    Some(Completion::new(&file_name, ""))
                } else {
                    None
                }
            })
            .collect()
    }
}
