pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IndexOutOfRange {
        index: usize,
    },
    IO {
        source: std::io::Error,
    },
    Internal {
        details: String,
        source: Option<Box<dyn std::error::Error>>,
    },
    SpaceNotEnough,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::IndexOutOfRange { index } => format!("index {} out of range", index),
                Self::IO { source } => format!("IO error: {}", source),
                Self::Internal { details, source } => {
                    format!(
                        "internal error: {}{}",
                        details,
                        match source {
                            Some(err) => format!(", source: {}", err),
                            None => "".to_string(),
                        }
                    )
                }
                Self::SpaceNotEnough => format!("space is not enough for insertion"),
            }
        )
    }
}
