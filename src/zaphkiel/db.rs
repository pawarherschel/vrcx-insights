use std::time::Duration;

use sqlx::{Sqlite, SqlitePool};

use crate::zaphkiel::cpu_info::CPU_THREADS;

#[allow(clippy::missing_panics_doc)]
pub async fn establish_connection() -> SqlitePool {
    sqlx::pool::PoolOptions::<Sqlite>::new()
        .acquire_timeout(Duration::from_secs(60 * 60))
        .max_connections(u32::try_from(CPU_THREADS).unwrap())
        .connect("sqlite://db/VRCX.sqlite3?mode=ro")
        .await
        .unwrap()
}
