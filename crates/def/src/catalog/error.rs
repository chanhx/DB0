use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    DatabaseAlreadyExists { name: String },
    DatabaseNotExists { name: String },
    TableAlreadyExists { name: String },
    TableNotExists { name: String },
    ColumnNotExists { name: String },
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::DatabaseAlreadyExists { name } =>
                    format!(r#"database "{}" already exists"#, name),
                Self::DatabaseNotExists { name } =>
                    format!(r#"database "{}" does not exist"#, name),
                Self::TableAlreadyExists { name } => format!(r#"table "{}" already exists"#, name),
                Self::TableNotExists { name } => format!(r#"table "{}" does not exist"#, name),
                Self::ColumnNotExists { name } => format!(r#"column "{}" does not exist"#, name),
            }
        )
    }
}
