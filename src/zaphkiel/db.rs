use sqlx::{Connection, SqlitePool};

pub async fn establish_connection() -> SqlitePool {
    SqlitePool::connect("./db/VRCX.sqlite3").await.unwrap()
}
