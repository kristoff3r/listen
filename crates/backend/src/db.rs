use std::time::Duration;

use deadpool::Runtime;
use diesel_async::{
    pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};

pub type PgPool = Pool<AsyncPgConnection>;
pub const POOL_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));
pub const POOL_SIZE: usize = 10;

pub fn setup_database_pool(db_url: &str) -> anyhow::Result<PgPool> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    let pool = Pool::builder(manager)
        .max_size(POOL_SIZE)
        .wait_timeout(POOL_TIMEOUT)
        .create_timeout(POOL_TIMEOUT)
        .recycle_timeout(POOL_TIMEOUT)
        .runtime(Runtime::Tokio1)
        .build()?;

    Ok(pool)
}
