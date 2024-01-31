use crate::error::MssqlError;
use async_trait::async_trait;
use bb8;
use tiberius::SqlBrowser;
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub(crate) struct ConnectionManager {
    config: Config,
    use_sql_browser: bool,
}

#[async_trait]
impl bb8::ManageConnection for ConnectionManager {
    type Connection = Client<Compat<TcpStream>>;
    type Error = MssqlError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let tcp = if self.use_sql_browser {
            TcpStream::connect_named(&self.config).await?
        } else {
            TcpStream::connect(&self.config.get_addr()).await?
        };

        tcp.set_nodelay(true)?;

        Client::connect(self.config.clone(), tcp.compat_write())
            .await
            .map_err(|e| e.into())
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.simple_query("SELECT 1").await?;
        Ok(())
    }

    fn has_broken(&self, _: &mut Self::Connection) -> bool {
        false
    }
}

pub(crate) struct ConnectionManagerBuilder {
    use_sql_browser: bool,
}

impl ConnectionManagerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn use_sql_browser(&mut self, yes: bool) -> &mut Self {
        self.use_sql_browser = yes;
        self
    }

    pub fn build(&self, config: Config) -> Result<ConnectionManager, MssqlError> {
        Ok(ConnectionManager {
            config,
            use_sql_browser: self.use_sql_browser,
        })
    }
}

impl Default for ConnectionManagerBuilder {
    fn default() -> Self {
        ConnectionManagerBuilder {
            use_sql_browser: true,
        }
    }
}
