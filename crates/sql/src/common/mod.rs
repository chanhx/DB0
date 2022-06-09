pub(crate) mod iter;
pub(crate) mod macros;

pub type Span = std::ops::RangeInclusive<usize>;

pub type Spanned<T> = (T, Span);
