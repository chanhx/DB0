pub(crate) mod iter;
pub(crate) mod macros;

#[cfg(test)]
pub(crate) mod test_utils;

pub type Span = std::ops::RangeInclusive<usize>;
