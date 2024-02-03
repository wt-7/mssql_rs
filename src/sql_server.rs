use crate::{
    error::Error,
    manager::{ConnectionManager, ConnectionManagerBuilder},
    TryFromRow,
};
use bb8;
use futures_util::{Stream, TryStreamExt};
use serde::de::DeserializeOwned;
use tiberius::{Query, QueryItem};

pub struct SqlServer {
    pool: bb8::Pool<ConnectionManager>,
}

impl SqlServer {
    /// Create a new SqlServer using the default configuration.
    /// The default configuration uses a single connection, SQL Browser, and a 5 second connection timeout.
    /// For more control over the configuration, use [`SqlServerBuilder`] instead.
    pub async fn new(config: tiberius::Config) -> Result<Self, Error> {
        SqlServerBuilder::new().build(config).await
    }

    /// Returns true if a connection is successfully returned from the pool
    pub async fn connection_ok(&self) -> bool {
        self.pool.get().await.is_ok()
    }
    /// Run a JSON query (e.g. SELECT ... FOR JSON PATH;) and return the result as a serde deserializable object.
    ///
    /// # Example
    ///
    /// ```
    /// let sql_server = SqlServer::new(cfg).await?;
    ///
    /// #[derive(serde::Deserialize)]
    /// struct Person {
    ///    id: i32,
    ///    name: String,
    /// }
    ///
    /// let query = "SELECT id, name FROM people FOR JSON PATH;";
    ///
    /// let rows = sql_server.json_query::<Vec<Person>>(query, &[]).await?;
    /// ```
    pub async fn json_query<T>(&self, query: &'static str, params: &[String]) -> Result<T, Error>
    where
        T: DeserializeOwned,
    {
        let mut select = Query::new(query);

        for param in params {
            select.bind(param);
        }

        let mut conn = self.pool.get().await?;

        let mut stream = select.query(&mut conn).await?;

        let size = stream.size_hint().1.unwrap_or(0);

        let mut json_buffer = String::with_capacity(size);

        while let Some(item) = stream.try_next().await? {
            if let QueryItem::Row(row) = item {
                if let Some(partial) = row.get(0) {
                    json_buffer.push_str(partial);
                }
            }
        }

        if json_buffer.is_empty() {
            // Return an error if the result set is empty, as this won't be valid JSON.
            // This error should be semantically different from a failure to parse.
            return Err(Error::EmptyResult);
        }

        serde_json::from_str::<T>(&json_buffer).map_err(|e| e.into())
    }

    /// Run a SQL query and return the result as Vec<T>.
    /// T must implement the TryFromRow trait, which specifies how to convert a [`tiberius::Row`] into T.
    ///
    /// # Example
    ///
    /// ```
    /// let sql_server = SqlServer::new(cfg).await?;
    ///
    /// struct Person {
    ///     id: i32,
    ///     name: String,
    /// }
    ///
    /// impl TryFromRow for Person {
    ///     fn try_from(row: tiberius::Row) -> mssql_rs::Result<Self> {
    ///         Ok(Person {
    ///             id: row.get(0).unwrap(),
    ///             name: row.get(1).map(|s: &str| s.to_owned()).unwrap(),
    ///         })
    ///     }
    /// }
    ///
    /// let query = "SELECT id, name FROM people;";
    ///
    /// let rows = sql_server.row_query::<Person>(query, &[]).await;
    /// ```

    pub async fn row_query<T>(
        &self,
        query: &'static str,
        params: &[String],
    ) -> Result<Vec<T>, Error>
    where
        T: TryFromRow,
    {
        let mut select = Query::new(query);

        for param in params {
            select.bind(param);
        }

        let mut conn = self.pool.get().await?;

        let mut stream = select.query(&mut conn).await?;

        let size = stream.size_hint().1.unwrap_or(0);

        let mut buf = Vec::with_capacity(size);

        while let Some(item) = stream.try_next().await? {
            if let QueryItem::Row(row) = item {
                let value = T::try_from(row)?;
                buf.push(value);
            }
        }

        Ok(buf)
    }
}

pub struct SqlServerBuilder {
    pool_max_size: u32,
    pool_connection_timeout: std::time::Duration,
    use_sql_browser: bool,
}

impl SqlServerBuilder {
    pub async fn build(&self, config: tiberius::Config) -> Result<SqlServer, Error> {
        let manager = ConnectionManagerBuilder::new()
            .use_sql_browser(self.use_sql_browser)
            .build(config)?;

        let pool = bb8::Pool::builder()
            .max_size(self.pool_max_size)
            .connection_timeout(self.pool_connection_timeout)
            .build(manager)
            .await?;

        Ok(SqlServer { pool })
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn pool_max_size(&mut self, pool_max_size: u32) -> &mut Self {
        self.pool_max_size = pool_max_size;
        self
    }

    pub fn use_sql_browser(&mut self, yes: bool) -> &mut Self {
        self.use_sql_browser = yes;
        self
    }

    pub fn pool_connection_timeout(
        &mut self,
        pool_connection_timeout: std::time::Duration,
    ) -> &mut Self {
        self.pool_connection_timeout = pool_connection_timeout;
        self
    }
}

impl Default for SqlServerBuilder {
    fn default() -> Self {
        SqlServerBuilder {
            pool_max_size: 1,
            use_sql_browser: true,
            pool_connection_timeout: std::time::Duration::from_secs(5),
        }
    }
}
