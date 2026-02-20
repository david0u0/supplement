use supplement::Result;
use supplement::completion::CompletionGroup;

mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}

pub fn run(cmd: &str) -> Result<CompletionGroup<def::ID>> {
    let cmd = cmd.split(" ").map(|s| s.to_string());
    let (_, grp) = def::CMD.supplement(cmd)?;
    Ok(grp)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{map_ready, map_unready};

    #[test]
    fn test_simple() {
        let comps = run("git -").unwrap();
        assert_eq!(vec!["--git-dir"], map_ready(&comps));

        let comps = run("git g").unwrap();
        assert_eq!(
            map_unready(&comps),
            (
                def::ID::External,
                "g",
                vec!["checkout", "log", "remote"],
                ""
            )
        );

        let comps = run("git log -").unwrap();
        assert_eq!(
            vec!["--flag1", "--git-dir", "--graph", "--pretty", "--pretty="],
            map_ready(&comps)
        );

        let comps = run("git checkout -").unwrap();
        assert_eq!(vec!["--git-dir"], map_ready(&comps));

        let comps = run("git log --pretty=").unwrap();
        assert_eq!(
            vec!["--pretty=full", "--pretty=oneline", "--pretty=short"],
            map_ready(&comps)
        );
    }
}
