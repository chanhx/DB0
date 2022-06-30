pub(crate) mod iter;

pub type Span = std::ops::RangeInclusive<usize>;

pub type Spanned<T> = (T, Span);
