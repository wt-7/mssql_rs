mod error;
mod manager;
mod pool;

pub use error::{Error, Result};
pub use pool::{SqlServerPool, SqlServerPoolBuilder};
pub use tiberius;

/// A trait for types that can be created from a [`tiberius::Row`].
///
/// This trait is required to use [`SqlServer::row_query`]
pub trait TryFromRow {
    fn try_from(row: tiberius::Row) -> Result<Self, crate::Error>
    where
        Self: Sized;
}
