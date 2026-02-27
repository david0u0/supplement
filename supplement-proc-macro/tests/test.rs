use supplement_proc_macro::id;

mod def {
    #[derive(PartialEq, Eq, Debug)]
    pub enum ID {
        ValGitDir,
        CMDRemote(remote::ID),
    }

    pub mod remote {
        #[derive(PartialEq, Eq, Debug)]
        pub enum ID {
            CMDSetUrl(set_url::ID),
        }

        pub mod set_url {
            #[derive(PartialEq, Eq, Debug)]
            pub enum ID {
                ValUrl,
            }
        }
    }
}

#[test]
fn test_proc_macro() {
    let e = id!(def git_dir);
    assert_eq!(e, def::ID::ValGitDir);

    let e = id!(def remote set_url url);
    assert_eq!(
        e,
        def::ID::CMDRemote(def::remote::ID::CMDSetUrl(def::remote::set_url::ID::ValUrl))
    );
}
