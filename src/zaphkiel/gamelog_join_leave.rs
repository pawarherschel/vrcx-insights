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

use std::sync::Arc;

use sqlx::types::chrono::{DateTime, Utc};

use crate::zaphkiel::join_leave_event::JoinLeaveEvent;
use crate::zaphkiel::world_instance::WorldInstance;

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct GamelogJoinLeaveRow {
    pub id: i64,
    pub created_at: String,
    pub r#type: String,
    pub display_name: String,
    pub location: String,
    pub user_id: String,
    pub time: i32,
}

#[derive(Debug, Clone)]
pub struct GamelogJoinLeave {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub event: JoinLeaveEvent,
    pub display_name: Arc<str>,
    pub location: Option<WorldInstance>,
    pub user_id: Option<Arc<str>>,
    pub time: Option<u64>,
}

impl GamelogJoinLeave {
    #[must_use]
    pub fn new() -> Self {
        let (id, created_at, event, location, user_id, time) = Default::default();
        let display_name = "".into();

        Self {
            id,
            created_at,
            event,
            display_name,
            location,
            user_id,
            time,
        }
    }
}

impl From<GamelogJoinLeaveRow> for GamelogJoinLeave {
    fn from(row: GamelogJoinLeaveRow) -> Self {
        let mut ret = Self::new();
        ret.id = row.id;
        ret.created_at = row.created_at.parse().unwrap();
        ret.event = row.r#type.parse().unwrap();
        ret.display_name = row.display_name.into();
        ret.location = if let Ok(location) = row.location.parse() {
            Some(location)
        } else {
            None
        };
        ret.user_id = match row.user_id {
            x if x.is_empty() => None,
            _ => Some(row.user_id.into()),
        };
        ret.time = match row.time {
            ..=0 => None,
            _ => Some(row.time as u64),
        };

        ret
    }
}
