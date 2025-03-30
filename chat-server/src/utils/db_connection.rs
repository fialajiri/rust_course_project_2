use anyhow::Result;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;

// Define the DbConn type for Rocket database connection
#[derive(rocket_db_pools::Database)]
#[database("postgres")]
pub struct DbConn(rocket_db_pools::diesel::PgPool);

// Define an alias for our database pool type
pub type DbPool = Pool<AsyncPgConnection>;

/// Creates a database connection pool
///
/// This is used for non-Rocket parts of the application
pub async fn create_pool() -> Result<DbPool> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);
    let pool = Pool::builder(config).max_size(5).build()?;

    Ok(pool)
}
