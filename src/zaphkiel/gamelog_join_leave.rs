// create table gamelog_join_leave
// (
// id           INTEGER
// primary key,
// created_at   TEXT,
// type         TEXT,
// display_name TEXT,
// location     TEXT,
// user_id      TEXT,
// time         INTEGER,
// unique (created_at, type, display_name)
// );

use sqlx::types::chrono::{DateTime, Utc};

use crate::zaphkiel::join_leave_event::JoinLeaveEvent;
use crate::zaphkiel::world_instance::WorldInstance;

#[derive(Debug, sqlx::FromRow, Default, Clone)]
pub struct GamelogJoinLeaveRow {
    pub id: i64,
    pub created_at: String,
    pub r#type: String,
    pub display_name: String,
    pub location: String,
    pub user_id: String,
    pub time: i32,
}

#[derive(Debug, Clone, Default)]
pub struct GamelogJoinLeave {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub event: JoinLeaveEvent,
    pub display_name: String,
    pub location: Option<WorldInstance>,
    pub user_id: Option<String>,
    pub time: Option<u64>,
}

impl GamelogJoinLeave {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<GamelogJoinLeaveRow> for GamelogJoinLeave {
    fn from(row: GamelogJoinLeaveRow) -> Self {
        let mut ret = Self::new();
        ret.id = row.id;
        ret.created_at = row.created_at.parse().unwrap();
        ret.event = row.r#type.parse().unwrap();
        ret.display_name = row.display_name;
        ret.location = if let Ok(location) = row.location.parse() {
            Some(location)
        } else {
            None
        };
        ret.user_id = match row.user_id {
            x if x.is_empty() => None,
            _ => Some(row.user_id),
        };
        ret.time = match row.time {
            ..=0 => None,
            _ => Some(row.time as u64),
        };

        ret
    }
}
