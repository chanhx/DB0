use {
    access::btree::error::Error as BTreeError, snafu::prelude::*,
    storage::buffer::Error as StorageError,
};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("Failed with accessing, source: {}", source))]
    Access {
        #[snafu(backtrace)]
        source: BTreeError,
    },

    #[snafu(display("Failed with storage error, source: {}", source))]
    Storage {
        #[snafu(backtrace)]
        source: StorageError,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
