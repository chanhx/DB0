use snafu::prelude::*;

type AnotherError = Box<dyn std::error::Error>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("Failed with storage error, source: {}", source))]
    Storage {
        // #[snafu(backtrace)]
        source: AnotherError,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
