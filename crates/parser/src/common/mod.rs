mod multi_peekable;

pub type Span = std::ops::RangeInclusive<usize>;
pub type Spanned<T> = (T, Span);

pub(crate) use multi_peekable::{MultiPeek, MultiPeekable};
