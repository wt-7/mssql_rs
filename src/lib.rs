mod error;
mod manager;
mod sql_server;

pub use error::{Error, Result};
pub use sql_server::{SqlServer, SqlServerBuilder};
pub use tiberius;

pub trait TryFromRow {
    fn try_from(row: tiberius::Row) -> Result<Self, crate::Error>
    where
        Self: Sized;
}
