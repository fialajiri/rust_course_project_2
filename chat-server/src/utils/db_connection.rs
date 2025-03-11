use anyhow::Result;
use diesel_async::pooled_connection::deadpool::{Object, Pool};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use dotenvy::dotenv;
use std::env;

pub type DbPool = Pool<AsyncPgConnection>;
pub type DbConnection = Object<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn create_pool() -> Result<DbPool> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;
    let pool_size: u32 = env::var("DB_POOL_SIZE")
        .unwrap_or_else(|_| "16".to_string())
        .parse()
        .unwrap_or(16);

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    Ok(Pool::builder(config).max_size(pool_size as usize).build()?)
}
