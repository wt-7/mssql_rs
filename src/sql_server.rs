use crate::{error::MssqlError, manager::TiberiusConnectionManager, FromRow, Queryable};
use bb8;
use futures_util::{Stream, TryStreamExt};
use serde::de::DeserializeOwned;
use tiberius::{Query, QueryItem};

pub struct SqlServer {
    pool: bb8::Pool<TiberiusConnectionManager>,
}

impl SqlServer {
    pub async fn new(config: tiberius::Config) -> Result<Self, MssqlError> {
        SqlServerBuilder::new().build(config).await
    }

    pub async fn test_connection(&self) -> bool {
        self.pool.get().await.is_ok()
    }

    pub async fn json_query<T>(&self, params: &[String]) -> Result<T, MssqlError>
    where
        T: DeserializeOwned + Queryable,
    {
        let mut select = Query::new(T::query());

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
            return Err(MssqlError::EmptyResult);
        }

        serde_json::from_str::<T>(&json_buffer).map_err(|e| e.into())
    }

    pub async fn row_query<T>(&self, params: &[String]) -> Result<Vec<T>, MssqlError>
    where
        T: Queryable + FromRow,
    {
        let mut select = Query::new(T::query());

        for param in params {
            select.bind(param);
        }

        let mut conn = self.pool.get().await?;

        let mut stream = select.query(&mut conn).await?;

        let size = stream.size_hint().1.unwrap_or(0);

        let mut buf = Vec::with_capacity(size);

        while let Some(item) = stream.try_next().await? {
            if let QueryItem::Row(row) = item {
                if let Ok(value) = T::from_row(row) {
                    buf.push(value);
                }
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
    pub async fn build(&self, config: tiberius::Config) -> Result<SqlServer, MssqlError> {
        let manager = TiberiusConnectionManager::new(config, self.use_sql_browser)?;

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
