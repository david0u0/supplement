use crate::id;

#[derive(Debug, Eq, PartialEq)]
pub struct HistoryUnitNoVal {
    pub id: id::NoVal,
    pub count: u32,
}
#[derive(Debug, Eq, PartialEq)]
pub struct HistoryUnitSingleVal<ID> {
    pub id: id::SingleVal<ID>,
    pub value: String,
}
#[derive(Debug, Eq, PartialEq)]
pub struct HistoryUnitMultiVal<ID> {
    pub id: id::MultiVal<ID>,
    pub values: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum HistoryUnit<ID> {
    No(HistoryUnitNoVal),
    Single(HistoryUnitSingleVal<ID>),
    Multi(HistoryUnitMultiVal<ID>),
}

pub trait Getter<ID> {
    type Ret;
    fn match_and_cast<'a>(&self, h: &'a HistoryUnit<ID>) -> Option<&'a Self::Ret>;
}
impl<ID> Getter<ID> for id::NoVal {
    type Ret = HistoryUnitNoVal;
    fn match_and_cast<'a>(&self, h: &'a HistoryUnit<ID>) -> Option<&'a Self::Ret> {
        match h {
            HistoryUnit::No(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}
impl<ID: PartialEq> Getter<ID> for id::SingleVal<ID> {
    type Ret = HistoryUnitSingleVal<ID>;
    fn match_and_cast<'a>(&self, h: &'a HistoryUnit<ID>) -> Option<&'a Self::Ret> {
        match h {
            HistoryUnit::Single(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}
impl<ID: PartialEq> Getter<ID> for id::MultiVal<ID> {
    type Ret = HistoryUnitMultiVal<ID>;
    fn match_and_cast<'a>(&self, h: &'a HistoryUnit<ID>) -> Option<&'a Self::Ret> {
        match h {
            HistoryUnit::Multi(h) if &h.id == self => Some(h),
            _ => None,
        }
    }
}

/// A structures that records all seen args/flags/commands, along with their value if they have some.
/// You can search in the history by their IDs using the `find` function.
#[derive(Debug, Eq, PartialEq)]
pub struct History<ID>(Vec<HistoryUnit<ID>>);
impl<ID: PartialEq + std::fmt::Debug> History<ID> {
    pub fn new() -> Self {
        History(vec![])
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
    pub(crate) fn push_single_val(&mut self, id: id::SingleVal<ID>, value: String) {
        log::debug!("push single val {:?} {}", id, value);
        for h in self.0.iter_mut() {
            match h {
                HistoryUnit::Single(h) if h.id == id => {
                    log::info!(
                        "push single val {:?}: {} where old value exists: {}",
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
    pub(crate) fn push_multi_val(&mut self, id: id::MultiVal<ID>, value: String) {
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

    pub(crate) fn push_valued(&mut self, id: id::Valued<ID>, value: String) {
        match id {
            id::Valued::Single(id) => self.push_single_val(id, value),
            id::Valued::Multi(id) => self.push_multi_val(id, value),
        }
    }

    /// Find the history of flags/args/commands by their ID.
    /// Based on the type of ID, the returned `HistoryUnit` will contain different types of value.
    /// - `id::NoVal`: An integer that represents how many times it's seen in the CLI command
    /// - `id::SingleVal`: A single string
    /// - `id::MultiVal`: A vector of string
    ///
    /// ```no_run
    /// use supplements::{History, id};
    /// let history = History::<()>::new();
    ///
    /// let id = id::NoVal::new(0);
    /// let c: u32 = history.find(&id).unwrap().count;
    ///
    /// let id = id::SingleVal::new(());
    /// let v: &String = &history.find(&id).unwrap().value;
    ///
    /// let id = id::MultiVal::new(());
    /// let v: &[String] = &history.find(&id).unwrap().values;
    /// ```
    pub fn find<I: Getter<ID>>(&self, id: &I) -> Option<&I::Ret> {
        for h in self.0.iter() {
            let h = id.match_and_cast(h);
            if h.is_some() {
                return h;
            }
        }
        None
    }

    #[doc(hidden)]
    pub fn from_vec(value: Vec<HistoryUnit<ID>>) -> Self {
        History(value)
    }
    #[doc(hidden)]
    pub fn into_inner(self) -> Vec<HistoryUnit<ID>> {
        self.0
    }
}
