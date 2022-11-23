use {
    super::PageTag,
    snafu::prelude::*,
    std::{backtrace::Backtrace, io},
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    Io {
        backtrace: Backtrace,
        source: io::Error,
    },

    #[snafu(display("page is not in buffer"))]
    PageNotInBuffer {
        backtrace: Backtrace,
        page_tag: PageTag,
    },

    #[snafu(display("buffer pool has no more buffer to offer"))]
    NoMoreBuffer { backtrace: Option<Backtrace> },
}

pub type Result<T> = std::result::Result<T, Error>;
