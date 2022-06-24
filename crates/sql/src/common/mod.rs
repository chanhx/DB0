pub(crate) mod iter;
pub(crate) mod macros;
mod types;

pub type Span = std::ops::RangeInclusive<usize>;

pub type Spanned<T> = (T, Span);

pub use types::{DataType, JoinType};
