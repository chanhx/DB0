use {
    crate::slotted_page,
    snafu::{prelude::*, Backtrace},
    storage::buffer,
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    Buffer {
        #[snafu(backtrace)]
        source: buffer::Error,
    },

    SlottedPage {
        #[snafu(backtrace)]
        source: slotted_page::Error,
    },

    #[snafu(display("Invalid page type {}", page_type))]
    InvalidPageType {
        backtrace: Backtrace,
        page_type: u8,
    },

    #[snafu(display("Duplicate key"))]
    DuplicateKey {
        backtrace: Backtrace,
    },

    #[snafu(display("Key `{}` not found", key))]
    KeyNotFound {
        backtrace: Backtrace,
        key: String,
    },

    Decoding {
        // #[snafu(backtrace)]
        source: Box<dyn std::error::Error>,
    },

    Encoding {
        source: Box<dyn std::error::Error>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
