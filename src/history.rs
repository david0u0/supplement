use crate::id;

#[derive(Debug, Eq, PartialEq)]
pub struct HistoryUnitNoVal {
    pub id: id::NoVal,
    pub count: u32,
}
#[derive(Debug, Eq, PartialEq)]
pub struct HistoryUnitSingleVal {
    pub id: id::SingleVal,
    pub value: String,
}
#[derive(Debug, Eq, PartialEq)]
pub struct HistoryUnitMultiVal {
    pub id: id::MultiVal,
    pub values: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HistoryUnit {
    No(HistoryUnitNoVal),
    Single(HistoryUnitSingleVal),
    Multi(HistoryUnitMultiVal),
}

pub trait ID: Copy {
    type Ret;
    fn match_and_cast(self, h: &HistoryUnit) -> Option<&Self::Ret>;
}
impl ID for id::NoVal {
    type Ret = HistoryUnitNoVal;
    fn match_and_cast(self, h: &HistoryUnit) -> Option<&Self::Ret> {
        match h {
            HistoryUnit::No(h) if h.id == self => Some(h),
            _ => None,
        }
    }
}
impl ID for id::SingleVal {
    type Ret = HistoryUnitSingleVal;
    fn match_and_cast(self, h: &HistoryUnit) -> Option<&Self::Ret> {
        match h {
            HistoryUnit::Single(h) if h.id == self => Some(h),
            _ => None,
        }
    }
}
impl ID for id::MultiVal {
    type Ret = HistoryUnitMultiVal;
    fn match_and_cast(self, h: &HistoryUnit) -> Option<&Self::Ret> {
        match h {
            HistoryUnit::Multi(h) if h.id == self => Some(h),
            _ => None,
        }
    }
}

/// A structures that records all seen args/flags/commands, along with their value if they have some
/// You can search in the history by their IDs using the `find` function
#[derive(Default, Debug, Eq, PartialEq)]
pub struct History(Vec<HistoryUnit>);
impl History {
    pub fn into_inner(self) -> Vec<HistoryUnit> {
        self.0
    }

    pub(crate) fn push_no_val(&mut self, id: id::NoVal) {
        log::debug!("push no value {:?}", id);
        for h in self.0.iter_mut() {
            match h {
                HistoryUnit::No(h) if h.id == id => {
                    h.count += 1;
                    return;
                }
                _ => (),
            }
        }

        self.0
            .push(HistoryUnit::No(HistoryUnitNoVal { id, count: 1 }));
    }
    pub(crate) fn push_single_val(&mut self, id: id::SingleVal, value: String) {
        log::debug!("push single val {:?} {}", id, value);
        for h in self.0.iter_mut() {
            match h {
                HistoryUnit::Single(h) if h.id == id => {
                    log::info!(
                        "push single val {:?} {} where old value exists: {}",
                        id,
                        value,
                        h.value
                    );
                    h.value = value;
                    return;
                }
                _ => (),
            }
        }

        self.0
            .push(HistoryUnit::Single(HistoryUnitSingleVal { id, value }));
    }
    pub(crate) fn push_multi_val(&mut self, id: id::MultiVal, value: String) {
        log::debug!("push multi val {:?} {}", id, value);
        for h in self.0.iter_mut() {
            match h {
                HistoryUnit::Multi(h) if h.id == id => {
                    h.values.push(value);
                    return;
                }
                _ => (),
            }
        }

        let values = vec![value];
        self.0
            .push(HistoryUnit::Multi(HistoryUnitMultiVal { id, values }));
    }

    pub(crate) fn push_arg(&mut self, id: id::Arg, value: String) {
        match id {
            id::Arg::Single(id) => self.push_single_val(id, value),
            id::Arg::Multi(id) => self.push_multi_val(id, value),
        }
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
}

impl From<Vec<HistoryUnit>> for History {
    fn from(value: Vec<HistoryUnit>) -> Self {
        History(value)
    }
}
