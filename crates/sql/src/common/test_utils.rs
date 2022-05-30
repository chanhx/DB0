use crate::stmt::Identifier;

#[cfg(test)]
pub(crate) fn identifier_from_str(s: &str) -> Identifier {
    Identifier(s.to_string(), 0..=s.len() - 1)
}
