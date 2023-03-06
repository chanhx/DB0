mod error;
mod init;

#[cfg(test)]
mod tests;

pub use {error::Error, init::create_meta_tables};
