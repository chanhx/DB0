use {crate::common::Span, common::pub_fields_struct};

#[derive(Debug)]
pub struct Identifier(pub String, pub Span);

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(test)]
pub(crate) fn identifier_from_str(s: &str) -> Identifier {
    Identifier(s.to_string(), 0..=s.len() - 1)
}

pub_fields_struct! {
    #[derive(Debug, PartialEq)]
    struct ColumnRef {
        column: Identifier,
        table: Option<Identifier>,
    }
}
