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

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_connection() {
        let cfg = tiberius::Config::from_ado_string("server=").unwrap();
        let db = SqlServer::new(cfg).await.unwrap();

        assert_eq!(db.connection_ok().await, true);
    }

    #[tokio::test]
    async fn json_query() {
        let cfg = tiberius::Config::from_ado_string("server=").unwrap();
        let sql_server = SqlServer::new(cfg).await.unwrap();

        #[derive(serde::Deserialize)]
        struct Person {
            id: i32,
            name: String,
        }

        let query = "SELECT id, name FROM people FOR JSON PATH;";

        let rows = sql_server
            .json_query::<Vec<Person>>(query, &[])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn row_query() {
        let cfg = tiberius::Config::from_ado_string("server=").unwrap();
        let sql_server = SqlServer::new(cfg).await.unwrap();

        struct Person {
            id: i32,
            name: String,
        }

        impl TryFromRow for Person {
            fn try_from(row: tiberius::Row) -> crate::Result<Self> {
                let id = row.get(0).unwrap();

                let name = row.try_get(1)?.map(|s: &str| s.to_owned()).unwrap();

                Ok(Person { id, name })
            }
        }

        let query = "SELECT id, name FROM people;";

        let rows = sql_server.row_query::<Person>(query, &[]).await.unwrap();
    }
}
