use std::fmt::Display;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    DatabaseAlreadyExists { name: String },
    DatabaseDoesNotExist { name: String },
    TableAlreadyExists { name: String },
    TableDoesNotExist { name: String },
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
                Self::DatabaseDoesNotExist { name } =>
                    format!(r#"database "{}" does not exist"#, name),
                Self::TableAlreadyExists { name } => format!(r#"table "{}" already exists"#, name),
                Self::TableDoesNotExist { name } => format!(r#"table "{}" does not exist"#, name),
            }
        )
    }
}
