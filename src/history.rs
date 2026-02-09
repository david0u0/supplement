use crate::id;

#[derive(Debug, Eq, PartialEq)]
pub struct SingleHistoryFlag {
    pub id: id::Flag,
    pub value: String,
}
#[derive(Debug, Eq, PartialEq)]
pub struct SingleHistoryCommand(pub id::Command);
#[derive(Debug, Eq, PartialEq)]
pub struct SingleHistoryArg {
    pub id: id::Arg,
    pub value: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum SingleHistory {
    Flag(SingleHistoryFlag),
    Command(SingleHistoryCommand),
    Arg(SingleHistoryArg),
}

pub trait ID: Copy {
    type Ret;
    fn match_and_cast(self, h: &SingleHistory) -> Option<&Self::Ret>;
}
impl ID for id::Flag {
    type Ret = SingleHistoryFlag;
    fn match_and_cast(self, h: &SingleHistory) -> Option<&Self::Ret> {
        match h {
            SingleHistory::Flag(h) if h.id == self => Some(h),
            _ => None,
        }
    }
}
impl ID for id::Arg {
    type Ret = SingleHistoryArg;
    fn match_and_cast(self, h: &SingleHistory) -> Option<&Self::Ret> {
        match h {
            SingleHistory::Arg(h) if h.id == self => Some(h),
            _ => None,
        }
    }
}
impl ID for id::Command {
    type Ret = SingleHistoryCommand;
    fn match_and_cast(self, h: &SingleHistory) -> Option<&Self::Ret> {
        match h {
            SingleHistory::Command(h) if h.0 == self => Some(h),
            _ => None,
        }
    }
}

#[derive(Default, Debug, Eq, PartialEq)]
pub struct History(Vec<SingleHistory>);
impl History {
    pub fn into_inner(self) -> Vec<SingleHistory> {
        self.0
    }

    pub fn push_arg(&mut self, id: id::Arg, value: String) {
        log::debug!("push arg {:?} {}", id, value);
        self.0
            .push(SingleHistory::Arg(SingleHistoryArg { id, value }));
    }
    pub fn push_flag(&mut self, id: id::Flag, value: String) {
        log::debug!("push flag {:?}: {}", id, value);
        self.0
            .push(SingleHistory::Flag(SingleHistoryFlag { id, value }));
    }
    pub fn push_pure_flag(&mut self, id: id::Flag) {
        log::debug!("push pure flag {:?}", id);
        self.0.push(SingleHistory::Flag(SingleHistoryFlag {
            id,
            value: String::new(),
        }));
    }
    pub fn push_command(&mut self, id: id::Command) {
        log::debug!("push command {:?}", id);
        self.0
            .push(SingleHistory::Command(SingleHistoryCommand(id)));
    }

    pub fn find_last<I: ID>(&self, id: I) -> Option<&I::Ret> {
        for h in self.0.iter().rev() {
            let h = id.match_and_cast(h);
            if h.is_some() {
                return h;
            }
        }
        None
    }
    pub fn find<I: ID>(&self, id: I) -> Option<&I::Ret> {
        for h in self.0.iter() {
            let h = id.match_and_cast(h);
            if h.is_some() {
                return h;
            }
        }
        None
    }
    pub fn find_all<'a, I: ID>(&'a self, ids: &'a [I]) -> impl Iterator<Item = &'a I::Ret> {
        self.0.iter().filter_map(|h| {
            for id in ids.iter() {
                if let Some(h) = id.match_and_cast(h) {
                    return Some(h);
                }
            }
            None
        })
    }
}

impl From<Vec<SingleHistory>> for History {
    fn from(value: Vec<SingleHistory>) -> Self {
        History(value)
    }
}
