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
        let db = SqlServerBuilder::new()
            .pool_max_size(1)
            .pool_connection_timeout(std::time::Duration::from_secs(5))
            .use_sql_browser(true)
            .build(cfg)
            .await
            .unwrap();

        struct TestRow {
            id: i32,
            name: String,
        }

        impl TryFromRow for TestRow {
            fn try_from(row: tiberius::Row) -> Result<Self, Error> {
                Ok(TestRow {
                    id: row.try_get(0)?.unwrap(),
                    name: row.try_get(1)?.map(|s: &str| s.to_owned()).unwrap(),
                })
            }
        }

        let rows = db
            .row_query::<TestRow>("SELECT 1 as id, 'test' as name", &[])
            .await
            .unwrap();
    }

    // #[tokio::test]
    // async fn json_query() {
    //     let cfg = tiberius::Config::from_ado_string("server=").unwrap();
    //     let sql_server = SqlServer::new(cfg).await?;

    //     #[derive(serde::Deserialize)]
    //     struct Person {
    //         id: i32,
    //         name: String,
    //     }

    //     let query = "SELECT id, name FROM people FOR JSON PATH;";

    //     let rows = sql_server.json_query::<Vec<Person>>(query, &[]).await?;
    // }

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
                let id = row.get(0);

                let name = row.try_get(1)?.map(|s: &str| s.to_owned()).unwrap();

                Ok(Person { id, name })
            }
        }

        let query = "SELECT id, name FROM people FOR JSON PATH;";

        let rows = sql_server.row_query::<Person>(query, &[]).await?;
    }
}
