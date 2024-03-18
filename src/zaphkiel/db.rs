use std::time::Duration;

use sqlx::{Sqlite, SqlitePool};

use crate::zaphkiel::cpu_info::CPU_THREADS;

pub async fn establish_connection() -> SqlitePool {
    sqlx::pool::PoolOptions::<Sqlite>::new()
        .acquire_timeout(Duration::from_secs(60 * 60))
        .max_connections(CPU_THREADS as u32)
        .connect("./db/VRCX.sqlite3")
        .await
        .unwrap()
}
