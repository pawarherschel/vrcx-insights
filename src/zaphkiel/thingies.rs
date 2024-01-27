use crate::zaphkiel::gamelog_join_leave::{GamelogJoinLeave, GamelogJoinLeaveRow};
use crate::zaphkiel::world_instance::WorldInstance;
use async_compat::Compat;
use sqlx::SqlitePool;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Instant;

#[must_use]
pub fn get_uuid_of(display_name: String, pool: &SqlitePool) -> String {
    let q = "select *
        from gamelog_join_leave
        where display_name like ?
        and user_id is not ''";

    let row = smol::block_on(async {
        sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
            .bind(display_name.clone())
            .fetch_one(pool)
            .await
    })
    .unwrap();

    assert!(
        !row.user_id.is_empty(),
        "No user_id found for {display_name}"
    );

    row.user_id
}

pub fn get_display_name_for(
    user_id: String,
    pool: &SqlitePool,
    cache: Arc<RwLock<HashMap<String, String>>>,
) -> String {
    if let Some(display_name) = cache.read().unwrap().get(&user_id) {
        return display_name.clone();
    }

    let q = "select *
        from gamelog_join_leave
        where user_id like ?
        order by created_at desc
        limit 1";

    let name = smol::block_on(async {
        sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
            .bind(user_id.clone())
            .fetch_one(pool)
            .await
    })
    .unwrap()
    .display_name;

    cache.write().unwrap().insert(user_id, name.clone());

    name
}

#[must_use]
pub fn get_locations_for(user_id: String, conn: &SqlitePool) -> HashSet<WorldInstance> {
    let q = "select *
        from gamelog_join_leave
        where user_id like ?";

    let rows = smol::block_on(async {
        sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
            .bind(user_id)
            .fetch_all(conn)
            .await
    })
    .unwrap();

    rows.iter()
        .cloned()
        .map(std::convert::Into::into)
        .filter_map(|row: GamelogJoinLeave| row.location)
        .collect()
}

#[must_use]
pub fn get_others_for(
    user_id: String,
    conn: &SqlitePool,
    locations: HashSet<WorldInstance>,
) -> HashMap<String, u32> {
    let _now = Instant::now();

    let others = locations
        .iter()
        // .progress_with(get_pb(locations.len() as u64, "Getting others"))
        .map(|location| {
            let q = "select *
                    from gamelog_join_leave
                    where location like ?
                    and location != ''
                    and user_id != ?
                    and user_id is not ''";

            let prefix = location.get_prefix();
            let location = format!("{}%", prefix);

            let rows = smol::block_on(Compat::new(async {
                sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
                    .bind(location)
                    .bind(user_id.clone())
                    .fetch_all(conn)
                    .await
            }))
            .unwrap();

            let rows = rows.into_iter().map(std::convert::Into::into);

            rows.into_iter()
                .map(|row: GamelogJoinLeave| row.user_id.unwrap())
                .collect::<HashSet<_>>()
        })
        .filter(|other| !other.is_empty())
        .collect::<Vec<_>>();

    let mut everyone_else = HashMap::new();

    for other in others {
        for user_id in other {
            let old = everyone_else.get(&user_id).unwrap_or(&0_u32);
            everyone_else.insert(user_id, old + 1);
        }
    }
    //
    // let mut op1 = everyone_else
    //     .par_iter()
    //     .map(|(k, v)| (k.to_owned(), v.to_owned()))
    //     .collect::<Vec<_>>();
    //
    // op1.sort_unstable_by_key(|(it, _)| it.to_owned());
    //
    // println!("part 1 done in {:?}", now.elapsed());
    //
    // let now = Instant::now();
    //
    // let op2 = get_others_for_new(user_id, conn, locations);
    // let mut op2 = op2
    //     .par_iter()
    //     .map(|(k, v)| (k.to_owned(), v.to_owned()))
    //     .collect::<Vec<_>>();
    //
    // op2.sort_unstable_by_key(|(it, _)| it.to_owned());
    //
    // println!("part 2 done in {:?}", now.elapsed());
    //
    // assert_eq!(op1.len(), op2.len());
    //
    // for ((old_key, old), (new_key, new)) in op1.into_iter().zip(op2.into_iter()) {
    //     assert_eq!(old_key, new_key);
    //     assert_eq!(old, new);
    // }

    everyone_else
}

// #[must_use]
// pub fn get_others_for_new(
//     user_id: String,
//     conn: &SqlitePool,
//     locations: HashSet<WorldInstance>,
// ) -> HashMap<String, u32> {
//     locations
//         .par_iter()
//         .fold(HashMap::new, |mut everyone_else, location| {
//             let prefix = location.get_prefix();
//             let location = format!("{}%", prefix);
//
//             let q = "select *
//                     from gamelog_join_leave
//                     where location like ?
//                     and location != ''
//                     and user_id != ?
//                     and user_id is not ''";
//
//             let rows = smol::block_on(Compat::new(async {
//                 sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
//                     .bind(location)
//                     .bind(user_id.clone())
//                     .fetch_all(conn)
//                     .await
//             }))
//             .unwrap();
//
//             for row in rows.iter().filter(|it| !it.user_id.is_empty()) {
//                 let user_id = row.user_id.clone();
//                 let old = everyone_else.get(&user_id).unwrap_or(&0_u32);
//                 everyone_else.insert(user_id, old + 1);
//             }
//
//             everyone_else
//         })
//         .reduce(HashMap::new, |mut a, b| {
//             for (k, v) in b {
//                 *a.entry(k).or_insert(0) += v;
//             }
//             a
//         })
// }
