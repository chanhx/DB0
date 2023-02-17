use ast::Span;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Internal(String),
    ColumnNotExists { name: String, span: Span },
    RelationNotExists { name: String },
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Internal(details) => details.clone(),
                Self::ColumnNotExists { name, .. } => format!("columen {} does not exists", name),
                Self::RelationNotExists { name } => format!("relation {} does not exists", name),
            }
        )
    }
}
