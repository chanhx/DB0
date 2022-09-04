use parser::Span;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    DuplicateColumn { span: Span, details: String },
    Internal(String),
    MultiplePrimaryKey { span: Span, details: String },
    UndefinedColumn { span: Span, details: String },
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
                Self::DuplicateColumn { details, .. } => details.clone(),
                Self::Internal(details) => details.clone(),
                Self::MultiplePrimaryKey { details, .. } => details.clone(),
                Self::UndefinedColumn { details, .. } => details.clone(),
                Self::ColumnNotExists { name, .. } => format!("columen {} does not exists", name),
                Self::RelationNotExists { name } => format!("relation {} does not exists", name),
            }
        )
    }
}
