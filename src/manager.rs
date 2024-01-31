use crate::error::MssqlError;
use async_trait::async_trait;
use bb8;
use tiberius::SqlBrowser;
use tiberius::{Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub(crate) struct TiberiusConnectionManager {
    config: Config,
    use_sql_browser: bool,
}

impl TiberiusConnectionManager {
    pub fn new(
        config: Config,
        use_sql_browser: bool,
    ) -> tiberius::Result<TiberiusConnectionManager> {
        Ok(TiberiusConnectionManager {
            config,
            use_sql_browser,
        })
    }
}
#[async_trait]
impl bb8::ManageConnection for TiberiusConnectionManager {
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
