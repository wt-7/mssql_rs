mod error;
mod manager;
mod sql_server;

pub use error::MssqlError;
pub use sql_server::{SqlServer, SqlServerBuilder};
pub use tiberius;

pub trait Queryable {
    fn query() -> &'static str;
}

pub trait FromRow {
    fn from_row(row: tiberius::Row) -> Result<Self, MssqlError>
    where
        Self: Sized;
}
