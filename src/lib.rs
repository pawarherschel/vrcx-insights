#![feature(lazy_cell)]

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;
use std::sync::{Arc, RwLock};

use sqlx::SqlitePool;
use tokio::task::JoinSet;

use crate::zaphkiel::gamelog_join_leave::{GamelogJoinLeave, GamelogJoinLeaveRow};
use crate::zaphkiel::is_kat::{Id, IsKat, Name, KAT_EXISTS};
use crate::zaphkiel::world_instance::WorldInstance;

pub mod zaphkiel {
    pub mod cpu_info;
    pub mod db;
    pub mod gamelog_join_leave;
    pub mod group_access_type;
    pub mod is_kat;
    pub mod join_leave_event;
    pub mod macros;
    pub mod metadata;
    // pub mod vertex;
    pub mod world_instance;
    pub mod world_regions;
}

#[must_use]
#[inline]
pub async fn get_uuid_of(display_name: Name, pool: &SqlitePool) -> Id {
    let q = "select *
        from gamelog_join_leave
        where display_name like ?
        and user_id is not ''";

    let row = sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
        .bind(display_name.to_string())
        .fetch_one(pool)
        .await
        .unwrap();

    assert!(
        !row.user_id.is_empty(),
        "No user_id found for {}",
        display_name.to_string()
    );

    row.user_id.into()
}

#[inline]
pub async fn get_display_name_for<S>(
    user_id: Id,
    pool: Arc<SqlitePool>,
    cache: Arc<RwLock<HashMap<Id, Arc<str>, S>>>,
) -> Name
where
    S: BuildHasher + Send + Sync,
{
    if let Some(display_name) = cache.read().unwrap().get(&user_id) {
        return display_name.clone().into();
    }

    let q = "select *
        from gamelog_join_leave
        where user_id like ?
        order by created_at desc
        limit 1";

    let name: String = sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
        .bind(user_id.to_string())
        .fetch_one(pool.as_ref())
        .await
        .unwrap()
        .display_name;

    let name: Arc<str> = name.into();

    cache.write().unwrap().insert(user_id, name.clone());

    name.into()
}

#[must_use]
#[inline]
pub async fn get_locations_for(user_id: Id, conn: Arc<SqlitePool>) -> HashSet<WorldInstance> {
    let q = "select *
        from gamelog_join_leave
        where user_id like ?";

    let rows = sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
        .bind(user_id.to_string())
        .fetch_all(conn.as_ref())
        .await
        .unwrap();

    rows.into_iter()
        .map(std::convert::Into::into)
        .filter_map(|row: GamelogJoinLeave| row.location)
        .collect()
}

#[must_use]
#[inline]
pub async fn get_others_for<S>(
    user_id: Id,
    conn: Arc<SqlitePool>,
    locations: HashSet<WorldInstance, S>,
) -> HashMap<Id, u32>
where
    S: BuildHasher + Send + Sync,
{
    let mut others = vec![];
    // let len = locations.len();
    let mut handles = JoinSet::new();
    for (idx, location) in locations.into_iter().enumerate() {
        let conn = conn.clone();
        let user_id = user_id.clone();
        handles.spawn(async move {
            let q = "select *
                    from gamelog_join_leave
                    where location like ?
                    and location != ''
                    and user_id != ?
                    and user_id is not ''";

            let prefix = location.get_prefix();
            let location = format!("{prefix}%");

            let rows = sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
                .bind(location)
                .bind(user_id.to_string())
                .fetch_all(conn.as_ref())
                .await
                .unwrap();

            let rows = rows.into_iter().map(std::convert::Into::into);

            let other = rows
                .map(|row: GamelogJoinLeave| row.user_id.unwrap().into())
                .filter(|it: &Id| !(*KAT_EXISTS && it.is_kat()))
                .collect::<HashSet<_>>();

            if other.is_empty() {
                None
            } else {
                Some((idx, other))
            }
        });
    }

    handles.len();

    while let Some(handle) = handles.join_next().await {
        if let Ok(Some((_idx, set))) = handle {
            others.push(set);
        }
    }

    let mut everyone_else = HashMap::new();

    for other in others {
        for user_id in other {
            let old = everyone_else.get(&user_id).unwrap_or(&0_u32);
            everyone_else.insert(user_id, old + 1);
        }
    }

    everyone_else
}
