use {
    common::pub_fields_struct,
    std::fmt::{Display, Formatter, Result},
};

pub type Span = std::ops::RangeInclusive<usize>;

#[derive(Debug, Clone)]
pub struct Spanned<T>(pub T, pub Span);

impl<T: PartialEq> PartialEq for Spanned<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Display> Display for Spanned<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.0.fmt(f)
    }
}

pub type Identifier = Spanned<String>;

// #[cfg(test)]
pub fn identifier_from_str(s: &str) -> Identifier {
    Spanned(s.to_string(), 0..=s.len() - 1)
}

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct ColumnRef {
        column: Identifier,
        table: Option<Identifier>,
    }
}
