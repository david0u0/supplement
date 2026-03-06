pub use crate::core::*;
pub use crate::history::History;

pub trait HistoryBearer<'a, ID> {
    type Ret;
    fn bear(self, h: &'a History<ID>) -> Self::Ret;
}
