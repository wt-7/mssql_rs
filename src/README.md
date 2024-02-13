# mssql_rs

**WIP**

High level MSSQL crate built on top of Tiberius


## Features:
- Async (Tokio) 
- Preconfigured bb8 connection pool
- Serde deserialization for JSON queries


## Getting started
Add the following to your cargo.toml:
```toml
mssql_rs = { git = "https://github.com/wt-7/mssql_rs" }
```
## Example

```rust
use mssql_rs::SqlServer;

const CON_STR: &str = "your-connection-string"

#[tokio::main]
async fn main() -> anyhow::Result<()>{
    let config = tiberius::Config::from_ado_string(CON_STR)?;
    // Create a SqlServer with the default configuration.
    // Use mssql_rs::SqlServerBuilder for more options
    let sql_server = SqlServer::new(cfg).await?;

     #[derive(serde::Deserialize)]
     struct Person {
        id: i32,
        name: String,
     }
    
    // JSON formatted query
    let query = "SELECT id, name FROM people FOR JSON PATH;";
    
    let rows = sql_server.json_query::<Vec<Person>>(query,&[]).await?;

    Ok(())
}
```
