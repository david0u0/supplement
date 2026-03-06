type GlobalID = ID;
use supplement::gen_prelude::*;

pub const ID_VAL_GIT_DIR: id::SingleVal<GlobalID> = id::SingleVal::new(ID::ValGitDir(Ctx));
const VAL_GIT_DIR: Flag<GlobalID> = Flag {
    short: &[],
    long: &["git-dir"],
    description: "",
    once: false,
    ty: flag_type::Type::new_valued(ID_VAL_GIT_DIR.into(), CompleteWithEqual::NoNeed, &[]),
};
pub const ID_VAL_EXTERNAL: id::MultiVal<GlobalID> = id::MultiVal::new(ID::ValExternal(Ctx));
const VAL_EXTERNAL: Flag<GlobalID> = Flag {
    short: &[],
    long: &["external"],
    description: "",
    once: false,
    ty: flag_type::Type::new_valued(ID_VAL_EXTERNAL.into(), CompleteWithEqual::NoNeed, &[]),
};
pub const ID_EXTERNAL: id::MultiVal<GlobalID> = id::MultiVal::new(ID::External(Ctx));
const EXTERNAL: Arg<GlobalID> = Arg {
    id: ID_EXTERNAL.into(),
    max_values: 18446744073709551615,
    possible_values: &[],
};
pub mod bisect {
    use super::GlobalID as GlobalID;
    use supplement::gen_prelude::*;

    pub const ID_VAL_ARG: id::SingleVal<GlobalID> = id::SingleVal::new_certain(line!());
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

    pub const ID_VAL_PRETTY: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDBisect2(ID::ValPretty(super::Ctx, Ctx)));
    const VAL_PRETTY: Flag<GlobalID> = Flag {
        short: &[],
        long: &["pretty"],
        description: "",
        once: true,
        ty: flag_type::Type::new_valued(ID_VAL_PRETTY.into(), CompleteWithEqual::Optional, &[("oneline", "<sha1> <title line>"), ("short", "<sha1> / <author> / <title line>)"), ("full", "<sha1> / <author> / <committer> / <title> / <commit msg>")]),
    };
    pub const ID_VAL_ARG: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDBisect2(ID::ValArg(super::Ctx, Ctx)));
    const VAL_ARG: Arg<GlobalID> = Arg {
        id: ID_VAL_ARG.into(),
        max_values: 1,
        possible_values: &[("good", ""), ("bad", "")],
    };
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Ctx;
    impl Ctx {
        pub fn arg(self, h: &History<GlobalID>) -> Option<&str> {
            h.find(ID_VAL_ARG).map(|x| x.value.as_ref())
        }
        pub fn pretty(self, h: &History<GlobalID>) -> Option<&str> {
            h.find(ID_VAL_PRETTY).map(|x| x.value.as_ref())
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID {
        ValArg(super::Ctx, Ctx),
        ValPretty(super::Ctx, Ctx),
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

    pub const ID_VAL_FILE_OR_COMMIT: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDCheckout(ID::ValFileOrCommit(super::Ctx, Ctx)));
    const VAL_FILE_OR_COMMIT: Arg<GlobalID> = Arg {
        id: ID_VAL_FILE_OR_COMMIT.into(),
        max_values: 1,
        possible_values: &[],
    };
    pub const ID_VAL_FILES: id::MultiVal<GlobalID> = id::MultiVal::new(super::ID::CMDCheckout(ID::ValFiles(super::Ctx, Ctx)));
    const VAL_FILES: Arg<GlobalID> = Arg {
        id: ID_VAL_FILES.into(),
        max_values: 18446744073709551615,
        possible_values: &[],
    };
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Ctx;
    impl Ctx {
        pub fn file_or_commit(self, h: &History<GlobalID>) -> Option<&str> {
            h.find(ID_VAL_FILE_OR_COMMIT).map(|x| x.value.as_ref())
        }
        pub fn files(self, h: &History<GlobalID>) -> &[String] {
            h.find(ID_VAL_FILES).map(|x| x.values.as_slice()).unwrap_or_default()
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID {
        ValFileOrCommit(super::Ctx, Ctx),
        ValFiles(super::Ctx, Ctx),
    }
    pub(super) const CMD: Command<GlobalID> = Command {
        name: "checkout",
        description: "",
        all_flags: &[super::VAL_GIT_DIR],
        args: &[VAL_FILE_OR_COMMIT, VAL_FILES],
        commands: &[],
    };
}
pub mod log {
    use super::GlobalID as GlobalID;
    use supplement::gen_prelude::*;

    pub const ID_VAL_GRAPH: id::NoVal = id::NoVal::new_certain(line!());
    const VAL_GRAPH: Flag<GlobalID> = Flag {
        short: &[],
        long: &["graph"],
        description: "",
        once: true,
        ty: flag_type::Type::new_bool(ID_VAL_GRAPH),
    };
    pub const ID_VAL_PRETTY: id::SingleVal<GlobalID> = id::SingleVal::new_certain(line!());
    const VAL_PRETTY: Flag<GlobalID> = Flag {
        short: &[],
        long: &["pretty"],
        description: "",
        once: true,
        ty: flag_type::Type::new_valued(ID_VAL_PRETTY.into(), CompleteWithEqual::Optional, &[("oneline", "<sha1> <title line>"), ("short", "<sha1> / <author> / <title line>)"), ("full", "<sha1> / <author> / <committer> / <title> / <commit msg>")]),
    };
    pub const ID_VAL_FLAG1: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDLog(ID::ValFlag1(super::Ctx, Ctx)));
    const VAL_FLAG1: Flag<GlobalID> = Flag {
        short: &[],
        long: &["flag1"],
        description: "",
        once: true,
        ty: flag_type::Type::new_valued(ID_VAL_FLAG1.into(), CompleteWithEqual::NoNeed, &[]),
    };
    pub const ID_VAL_COMMIT: id::SingleVal<GlobalID> = id::SingleVal::new(super::ID::CMDLog(ID::ValCommit(super::Ctx, Ctx)));
    const VAL_COMMIT: Arg<GlobalID> = Arg {
        id: ID_VAL_COMMIT.into(),
        max_values: 1,
        possible_values: &[],
    };
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct Ctx;
    impl Ctx {
        pub fn commit(self, h: &History<GlobalID>) -> Option<&str> {
            h.find(ID_VAL_COMMIT).map(|x| x.value.as_ref())
        }
        pub fn graph(self, h: &History<GlobalID>) -> u32 {
            h.find(ID_VAL_GRAPH).map(|x| x.count).unwrap_or_default()
        }
        pub fn pretty(self, h: &History<GlobalID>) -> Option<&str> {
            h.find(ID_VAL_PRETTY).map(|x| x.value.as_ref())
        }
        pub fn flag1(self, h: &History<GlobalID>) -> Option<&str> {
            h.find(ID_VAL_FLAG1).map(|x| x.value.as_ref())
        }
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID {
        ValCommit(super::Ctx, Ctx),
        ValFlag1(super::Ctx, Ctx),
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

        pub const ID_VAL_TAGS: id::NoVal = id::NoVal::new_certain(line!());
        const VAL_TAGS: Flag<GlobalID> = Flag {
            short: &[],
            long: &["tags"],
            description: "",
            once: true,
            ty: flag_type::Type::new_bool(ID_VAL_TAGS),
        };
        pub const ID_VAL_NAME: id::SingleVal<GlobalID> = id::SingleVal::new(super::super::ID::CMDRemote(super::ID::CMDAdd(ID::ValName(super::super::Ctx, super::Ctx, Ctx))));
        const VAL_NAME: Arg<GlobalID> = Arg {
            id: ID_VAL_NAME.into(),
            max_values: 1,
            possible_values: &[],
        };
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub struct Ctx;
        impl Ctx {
            pub fn name(self, h: &History<GlobalID>) -> Option<&str> {
                h.find(ID_VAL_NAME).map(|x| x.value.as_ref())
            }
            pub fn tags(self, h: &History<GlobalID>) -> u32 {
                h.find(ID_VAL_TAGS).map(|x| x.count).unwrap_or_default()
            }
        }
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        pub enum ID {
            ValName(super::super::Ctx, super::Ctx, Ctx),
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
    pub struct Ctx;
    impl Ctx {
    }
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum ID {
        CMDAdd(add::ID),
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
pub struct Ctx;
impl Ctx {
    pub fn git_dir(self, h: &History<GlobalID>) -> Option<&str> {
        h.find(ID_VAL_GIT_DIR).map(|x| x.value.as_ref())
    }
    pub fn external(self, h: &History<GlobalID>) -> &[String] {
        h.find(ID_VAL_EXTERNAL).map(|x| x.values.as_slice()).unwrap_or_default()
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ID {
    External(Ctx),
    ValGitDir(Ctx),
    ValExternal(Ctx),
    CMDBisect2(bisect2::ID),
    CMDCheckout(checkout::ID),
    CMDLog(log::ID),
    CMDRemote(remote::ID),
}
pub const CMD: Command<GlobalID> = Command {
    name: "supplement-tests",
    description: "",
    all_flags: &[VAL_GIT_DIR, VAL_EXTERNAL],
    args: &[EXTERNAL],
    commands: &[bisect::CMD, bisect2::CMD, checkout::CMD, log::CMD, remote::CMD],
};
