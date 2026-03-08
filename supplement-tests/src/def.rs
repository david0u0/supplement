type GlobalID = ID;
use supplement::gen_prelude::*;

const ID_VAL_GIT_DIR: id::SingleVal<GlobalID> = id::SingleVal::new(ID::ValGitDir(Ctx(())));
const VAL_GIT_DIR: Flag<GlobalID> = Flag {
    short: &[],
    long: &["git-dir"],
    description: "",
    once: false,
    ty: flag_type::Type::new_valued(ID_VAL_GIT_DIR.into(), CompleteWithEqual::NoNeed, &[]),
};
const ID_VAL_EXTERNAL: id::MultiVal<GlobalID> = id::MultiVal::new(ID::ValExternal(Ctx(())));
const VAL_EXTERNAL: Flag<GlobalID> = Flag {
    short: &[],
    long: &["external"],
    description: "",
    once: false,
    ty: flag_type::Type::new_valued(ID_VAL_EXTERNAL.into(), CompleteWithEqual::NoNeed, &[]),
};
const ID_EXTERNAL: id::MultiVal<GlobalID> = id::MultiVal::new(ID::External(Ctx(())));
const EXTERNAL: Arg<GlobalID> = Arg {
    id: ID_EXTERNAL.into(),
    max_values: 18446744073709551615,
    possible_values: &[],
};
pub mod bisect {
    use super::GlobalID as GlobalID;
    use supplement::gen_prelude::*;

    const ID_VAL_ARG: id::SingleVal<GlobalID> = id::SingleVal::new_certain(line!());
    const VAL_ARG: Arg<GlobalID> = Arg {
        id: ID_VAL_ARG.into(),
        max_values: 1,
        possible_values: &[("good", ""), ("bad", "")],
    };
    pub(super) const CMD: Command<GlobalID> = Command {
        name: "bisect",
        description: "",
        all_flags: &[super::VAL_GIT_DIR],
        args: &[VAL_ARG],
        commands: &[],
    };
}
pub mod bisect2 {
    use super::GlobalID as GlobalID;
    use supplement::gen_prelude::*;

    const ID_VAL_PRETTY: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDBisect2(ID::ValPretty(super::Ctx(()), Ctx(()))));
    const VAL_PRETTY: Flag<GlobalID> = Flag {
        short: &[],
        long: &["pretty"],
        description: "",
        once: true,
        ty: flag_type::Type::new_valued(ID_VAL_PRETTY.into(), CompleteWithEqual::Optional, &[("oneline", "<sha1> <title line>"), ("short", "<sha1> / <author> / <title line>)"), ("full", "<sha1> / <author> / <committer> / <title> / <commit msg>")]),
    };
    const ID_VAL_ARG: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDBisect2(ID::ValArg(super::Ctx(()), Ctx(()))));
    const VAL_ARG: Arg<GlobalID> = Arg {
        id: ID_VAL_ARG.into(),
        max_values: 1,
        possible_values: &[("good", ""), ("bad", "")],
    };
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Ctx<H>(H);
    impl Ctx<&History<GlobalID>> {
        #[allow(dead_code)]
        pub fn val_arg(&self) -> Option<&str> {
            self.0.find(ID_VAL_ARG).map(|x| x.value.as_ref())
        }
        #[allow(dead_code)]
        pub fn val_pretty(&self) -> Option<&str> {
            self.0.find(ID_VAL_PRETTY).map(|x| x.value.as_ref())
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID<H = ()> {
        ValArg(super::Ctx<H>, Ctx<H>),
        ValPretty(super::Ctx<H>, Ctx<H>),
    }
    impl ID<()> {
        #[allow(dead_code)]
        pub fn with_ctx(self, h: &History<GlobalID>) -> ID<&History<GlobalID>> {
            match self {
                ID::ValArg(..) => ID::ValArg(super::Ctx(h), Ctx(h)),
                ID::ValPretty(..) => ID::ValPretty(super::Ctx(h), Ctx(h)),
            }
        }
    }
    pub(super) const CMD: Command<GlobalID> = Command {
        name: "bisect2",
        description: "",
        all_flags: &[VAL_PRETTY, super::VAL_GIT_DIR],
        args: &[VAL_ARG],
        commands: &[],
    };
}
pub mod checkout {
    use super::GlobalID as GlobalID;
    use supplement::gen_prelude::*;

    const ID_VAL_B: id::NoVal = id::NoVal::new_certain(line!());
    const VAL_B: Flag<GlobalID> = Flag {
        short: &['b'],
        long: &[],
        description: "Create new branch",
        once: true,
        ty: flag_type::Type::new_bool(ID_VAL_B),
    };
    const ID_VAL_FILE_OR_COMMIT: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDCheckout(ID::ValFileOrCommit(super::Ctx(()), Ctx(()))));
    const VAL_FILE_OR_COMMIT: Arg<GlobalID> = Arg {
        id: ID_VAL_FILE_OR_COMMIT.into(),
        max_values: 1,
        possible_values: &[],
    };
    const ID_VAL_FILES: id::MultiVal<GlobalID> = id::MultiVal::new(super::ID::CMDCheckout(ID::ValFiles(super::Ctx(()), Ctx(()))));
    const VAL_FILES: Arg<GlobalID> = Arg {
        id: ID_VAL_FILES.into(),
        max_values: 18446744073709551615,
        possible_values: &[],
    };
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Ctx<H>(H);
    impl Ctx<&History<GlobalID>> {
        #[allow(dead_code)]
        pub fn val_file_or_commit(&self) -> Option<&str> {
            self.0.find(ID_VAL_FILE_OR_COMMIT).map(|x| x.value.as_ref())
        }
        #[allow(dead_code)]
        pub fn val_files(&self) -> &[String] {
            self.0.find(ID_VAL_FILES).map(|x| x.values.as_slice()).unwrap_or_default()
        }
        #[allow(dead_code)]
        pub fn val_b(&self) -> u32 {
            self.0.find(ID_VAL_B).map(|x| x.count).unwrap_or_default()
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID<H = ()> {
        ValFileOrCommit(super::Ctx<H>, Ctx<H>),
        ValFiles(super::Ctx<H>, Ctx<H>),
    }
    impl ID<()> {
        #[allow(dead_code)]
        pub fn with_ctx(self, h: &History<GlobalID>) -> ID<&History<GlobalID>> {
            match self {
                ID::ValFileOrCommit(..) => ID::ValFileOrCommit(super::Ctx(h), Ctx(h)),
                ID::ValFiles(..) => ID::ValFiles(super::Ctx(h), Ctx(h)),
            }
        }
    }
    pub(super) const CMD: Command<GlobalID> = Command {
        name: "checkout",
        description: "",
        all_flags: &[VAL_B, super::VAL_GIT_DIR],
        args: &[VAL_FILE_OR_COMMIT, VAL_FILES],
        commands: &[],
    };
}
pub mod log {
    use super::GlobalID as GlobalID;
    use supplement::gen_prelude::*;

    const ID_VAL_GRAPH: id::NoVal = id::NoVal::new_certain(line!());
    const VAL_GRAPH: Flag<GlobalID> = Flag {
        short: &[],
        long: &["graph"],
        description: "",
        once: true,
        ty: flag_type::Type::new_bool(ID_VAL_GRAPH),
    };
    const ID_VAL_PRETTY: id::SingleVal<GlobalID> = id::SingleVal::new_certain(line!());
    const VAL_PRETTY: Flag<GlobalID> = Flag {
        short: &[],
        long: &["pretty"],
        description: "",
        once: true,
        ty: flag_type::Type::new_valued(ID_VAL_PRETTY.into(), CompleteWithEqual::Optional, &[("oneline", "<sha1> <title line>"), ("short", "<sha1> / <author> / <title line>)"), ("full", "<sha1> / <author> / <committer> / <title> / <commit msg>")]),
    };
    const ID_VAL_FLAG1: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDLog(ID::ValFlag1(super::Ctx(()), Ctx(()))));
    const VAL_FLAG1: Flag<GlobalID> = Flag {
        short: &[],
        long: &["flag1"],
        description: "",
        once: true,
        ty: flag_type::Type::new_valued(ID_VAL_FLAG1.into(), CompleteWithEqual::NoNeed, &[]),
    };
    const ID_VAL_COMMIT: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDLog(ID::ValCommit(super::Ctx(()), Ctx(()))));
    const VAL_COMMIT: Arg<GlobalID> = Arg {
        id: ID_VAL_COMMIT.into(),
        max_values: 1,
        possible_values: &[],
    };
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Ctx<H>(H);
    impl Ctx<&History<GlobalID>> {
        #[allow(dead_code)]
        pub fn val_commit(&self) -> Option<&str> {
            self.0.find(ID_VAL_COMMIT).map(|x| x.value.as_ref())
        }
        #[allow(dead_code)]
        pub fn val_graph(&self) -> u32 {
            self.0.find(ID_VAL_GRAPH).map(|x| x.count).unwrap_or_default()
        }
        #[allow(dead_code)]
        pub fn val_pretty(&self) -> Option<&str> {
            self.0.find(ID_VAL_PRETTY).map(|x| x.value.as_ref())
        }
        #[allow(dead_code)]
        pub fn val_flag1(&self) -> Option<&str> {
            self.0.find(ID_VAL_FLAG1).map(|x| x.value.as_ref())
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID<H = ()> {
        ValCommit(super::Ctx<H>, Ctx<H>),
        ValFlag1(super::Ctx<H>, Ctx<H>),
    }
    impl ID<()> {
        #[allow(dead_code)]
        pub fn with_ctx(self, h: &History<GlobalID>) -> ID<&History<GlobalID>> {
            match self {
                ID::ValCommit(..) => ID::ValCommit(super::Ctx(h), Ctx(h)),
                ID::ValFlag1(..) => ID::ValFlag1(super::Ctx(h), Ctx(h)),
            }
        }
    }
    pub(super) const CMD: Command<GlobalID> = Command {
        name: "log",
        description: "log",
        all_flags: &[VAL_GRAPH, VAL_PRETTY, VAL_FLAG1, super::VAL_GIT_DIR],
        args: &[VAL_COMMIT],
        commands: &[],
    };
}
pub mod remote {
    use super::GlobalID as GlobalID;
    use supplement::gen_prelude::*;

    pub mod add {
        use super::super::GlobalID as GlobalID;
        use supplement::gen_prelude::*;

        const ID_VAL_TAGS: id::NoVal = id::NoVal::new_certain(line!());
        const VAL_TAGS: Flag<GlobalID> = Flag {
            short: &[],
            long: &["tags"],
            description: "",
            once: true,
            ty: flag_type::Type::new_bool(ID_VAL_TAGS),
        };
        const ID_VAL_NAME: id::SingleVal<GlobalID> = id::SingleVal::new(super::super::ID::CMDRemote(super::ID::CMDAdd(ID::ValName(super::super::Ctx(()), super::Ctx(()), Ctx(())))));
        const VAL_NAME: Arg<GlobalID> = Arg {
            id: ID_VAL_NAME.into(),
            max_values: 1,
            possible_values: &[],
        };
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct Ctx<H>(H);
        impl Ctx<&History<GlobalID>> {
            #[allow(dead_code)]
            pub fn val_name(&self) -> Option<&str> {
                self.0.find(ID_VAL_NAME).map(|x| x.value.as_ref())
            }
            #[allow(dead_code)]
            pub fn val_tags(&self) -> u32 {
                self.0.find(ID_VAL_TAGS).map(|x| x.count).unwrap_or_default()
            }
        }
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum ID<H = ()> {
            ValName(super::super::Ctx<H>, super::Ctx<H>, Ctx<H>),
        }
        impl ID<()> {
            #[allow(dead_code)]
            pub fn with_ctx(self, h: &History<GlobalID>) -> ID<&History<GlobalID>> {
                match self {
                    ID::ValName(..) => ID::ValName(super::super::Ctx(h), super::Ctx(h), Ctx(h)),
                }
            }
        }
        pub(super) const CMD: Command<GlobalID> = Command {
            name: "add",
            description: "",
            all_flags: &[VAL_TAGS],
            args: &[VAL_NAME],
            commands: &[],
        };
    }
    pub mod remove {
        use super::super::GlobalID as GlobalID;
        use supplement::gen_prelude::*;

        pub(super) const CMD: Command<GlobalID> = Command {
            name: "remove",
            description: "",
            all_flags: &[super::super::VAL_GIT_DIR],
            args: &[],
            commands: &[],
        };
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Ctx<H>(H);
    impl Ctx<&History<GlobalID>> {
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID<H = ()> {
        CMDAdd(add::ID<H>),
    }
    impl ID<()> {
        #[allow(dead_code)]
        pub fn with_ctx(self, h: &History<GlobalID>) -> ID<&History<GlobalID>> {
            match self {
                ID::CMDAdd(id) => ID::CMDAdd(id.with_ctx(h)),
            }
        }
    }
    pub(super) const CMD: Command<GlobalID> = Command {
        name: "remote",
        description: "",
        all_flags: &[super::VAL_GIT_DIR],
        args: &[],
        commands: &[add::CMD, remove::CMD],
    };
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ctx<H>(H);
impl Ctx<&History<GlobalID>> {
    #[allow(dead_code)]
    pub fn external(&self) -> &[String] {
        self.0.find(ID_EXTERNAL).map(|x| x.values.as_slice()).unwrap_or_default()
    }
    #[allow(dead_code)]
    pub fn val_git_dir(&self) -> Option<&str> {
        self.0.find(ID_VAL_GIT_DIR).map(|x| x.value.as_ref())
    }
    #[allow(dead_code)]
    pub fn val_external(&self) -> &[String] {
        self.0.find(ID_VAL_EXTERNAL).map(|x| x.values.as_slice()).unwrap_or_default()
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ID<H = ()> {
    External(Ctx<H>),
    ValGitDir(Ctx<H>),
    ValExternal(Ctx<H>),
    CMDBisect2(bisect2::ID<H>),
    CMDCheckout(checkout::ID<H>),
    CMDLog(log::ID<H>),
    CMDRemote(remote::ID<H>),
}
impl ID<()> {
    #[allow(dead_code)]
    pub fn with_ctx(self, h: &History<GlobalID>) -> ID<&History<GlobalID>> {
        match self {
            ID::External(..) => ID::External(Ctx(h)),
            ID::ValGitDir(..) => ID::ValGitDir(Ctx(h)),
            ID::ValExternal(..) => ID::ValExternal(Ctx(h)),
            ID::CMDBisect2(id) => ID::CMDBisect2(id.with_ctx(h)),
            ID::CMDCheckout(id) => ID::CMDCheckout(id.with_ctx(h)),
            ID::CMDLog(id) => ID::CMDLog(id.with_ctx(h)),
            ID::CMDRemote(id) => ID::CMDRemote(id.with_ctx(h)),
        }
    }
}
pub const CMD: Command<GlobalID> = Command {
    name: "supplement-tests",
    description: "",
    all_flags: &[VAL_GIT_DIR, VAL_EXTERNAL],
    args: &[EXTERNAL],
    commands: &[bisect::CMD, bisect2::CMD, checkout::CMD, log::CMD, remote::CMD],
};
