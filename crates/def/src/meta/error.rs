use {
    crate::types::Error as TypeError,
    snafu::{prelude::*, Backtrace},
    std::{io, string::FromUtf8Error},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(in crate::meta)))]
pub enum Error {
    FromIo {
        source: io::Error,
    },

    Utf8Encoding {
        source: FromUtf8Error,
    },

    #[snafu(display("internal error"))]
    Internal {
        backtrace: Backtrace,
    },

    MismatchedType {
        backtrace: Backtrace,
    },

    TypeEncoding {
        #[snafu(backtrace)]
        source: TypeError,
    },

    #[snafu(display("the count of values does not match the count of columns"))]
    ValuesCount {
        backtrace: Backtrace,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
