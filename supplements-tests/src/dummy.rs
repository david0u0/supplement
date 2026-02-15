use supplements::Result;
use supplements::completion::CompletionGroup;

mod def {
    include!(concat!(env!("OUT_DIR"), "/definition.rs"));
}
use def::*;
mod dummy_impl {
    include!(concat!(env!("OUT_DIR"), "/dummy_impl.rs"));
}

pub fn run(cmd: &str) -> Result<CompletionGroup> {
    let cmd = cmd.split(" ").map(|s| s.to_string());
    def::CMD.supplement(cmd)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::map_comps;

    #[test]
    fn test_simple() {
        let comps = run("git -").unwrap();
        assert_eq!(vec!["--git-dir"], map_comps(&comps));

        let comps = run("git g").unwrap();
        assert_eq!(vec!["checkout", "log"], map_comps(&comps));

        let comps = run("git log -").unwrap();
        assert_eq!(
            vec!["--flag1", "--git-dir", "--graph", "--pretty", "--pretty="],
            map_comps(&comps)
        );

        let comps = run("git checkout -").unwrap();
        assert_eq!(vec!["--git-dir"], map_comps(&comps));
    }
}
