use std::collections::{HashMap, HashSet};

use async_compat::Compat;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use smol::prelude::*;

use vrcx_insights::{debug, time_it};
use vrcx_insights::zaphkiel::db::establish_connection;
use vrcx_insights::zaphkiel::gamelog_join_leave::{GamelogJoinLeave, GamelogJoinLeaveRow};
use vrcx_insights::zaphkiel::utils::get_pb;

#[tokio::main]
async fn main() {
    let conn = time_it!("establishing connection to database" => establish_connection().await);

    let rows = sqlx::query_as::<_, GamelogJoinLeaveRow>("SELECT * FROM gamelog_join_leave")
        .fetch_all(&conn)
        .await
        .unwrap()
        .iter()
        .cloned()
        .map(|row| row.into())
        .collect::<Vec<GamelogJoinLeave>>();

    let locations = time_it!("finding out all locations" => rows
        .iter()
        .filter_map(|row| row.location.clone())
        .collect::<HashSet<_>>());

    let others = time_it!(at once | format!("processing {} locations", locations.len()) => {
    locations
        .par_iter()
        .progress_with(get_pb(locations.len() as u64, "Processing locations"))
        .map(|location| {
            let q =
                "select *
                    from gamelog_join_leave
                    where location like ?";

            let prefix = location.get_prefix();
            let location = format!("%{}%", prefix);

            let rows = smol::block_on(
                Compat::new(
                async {
                    sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
                        .bind(location)
                        .fetch_all(&conn).await
                })
            ).unwrap();

            debug!("found {} rows", rows.len());

            let rows = rows
                .iter()
                .cloned()
                .map(|row| row.into())
                .collect::<Vec<GamelogJoinLeave>>();

            debug!("converted {} rows", rows.len());

            rows.iter()
                .filter(|row| {
                    row.location.is_some()
                })
                .map(|row| row.display_name.clone())
                .collect::<HashSet<_>>()
        })
        .filter(|other| !other.is_empty())
        .collect::<Vec<_>>()
    });
    let others = time_it!("merging all instances of other users" => {
        let mut everyone_else = HashMap::new();

        others.iter().for_each(|other| {
            other.iter().cloned().for_each(|user_id| {
                let old = everyone_else.get(&user_id).unwrap_or(&0_u32);
                everyone_else.insert(user_id, old + 1);
            });
        });

        everyone_else
    });

    let others = time_it!("converting to sorted vector" => {
        let mut others = others
            .iter()
            .map(|(name, occurrence)| (name.clone(), *occurrence))
            .collect::<Vec<_>>();
        others.sort_by(|a, b| b.1.cmp(&a.1));
        others
    });

    println!("top 10: {:#?}", others.iter().take(10).collect::<Vec<_>>());
}
