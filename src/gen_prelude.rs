pub use crate::core::*;
pub use crate::history::History;

pub trait HistoryBearer<'a, ID> {
    type Ret;
    fn bear(self, h: &'a History<ID>) -> Self::Ret;
}

impl<'a> HistoryBearer<'a, u32> for u32 {
    type Ret = u32;
    fn bear(self, _: &'a History<u32>) -> Self::Ret {
        self
    }
}
