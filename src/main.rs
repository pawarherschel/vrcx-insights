use std::collections::{HashMap, HashSet};

use async_compat::Compat;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use smol::prelude::*;
use sqlx::SqlitePool;

use vrcx_insights::time_it;
use vrcx_insights::zaphkiel::db::establish_connection;
use vrcx_insights::zaphkiel::gamelog_join_leave::{GamelogJoinLeave, GamelogJoinLeaveRow};
use vrcx_insights::zaphkiel::utils::get_pb;
use vrcx_insights::zaphkiel::vertex::Vertex;
use vrcx_insights::zaphkiel::world_instance::WorldInstance;

fn main() {
    let conn = time_it!("establishing connection to database" => smol::block_on(async {establish_connection().await}));

    let test_name = "Kat Sakura".to_string();

    let user_id = get_uuid_of(test_name);

    let locations = get_locations_for(user_id.clone(), &conn);

    let others = get_others_for(user_id.clone(), &conn, locations);

    let mut root = Vertex::new(user_id);
    root.everyone_else = others
        .par_iter()
        .progress_with(get_pb(others.len() as u64, "Generating graph"))
        .map(|(user_id, count)| (Vertex::new(user_id.clone()), *count))
        .map(|(vertex, count)| {
            let locations = get_locations_for(vertex.user_id.clone(), &conn);
            let others = get_others_for(vertex.user_id.clone(), &conn, locations);
            let mut vertex = vertex;
            vertex.everyone_else = others
                .par_iter()
                .map(|(user_id, count)| (Vertex::new(user_id.clone()), *count))
                .collect();
            (vertex, count)
        })
        .collect::<HashMap<_, _>>();

    // let mut vertices = HashSet::new();
    // vertices.insert(root.clone());
    // root.everyone_else.iter().for_each(|(vertex, _)| {
    //     vertices.insert(vertex.clone());
    //     vertex.everyone_else.iter().for_each(|(vertex, _)| {
    //         vertices.insert(vertex.clone());
    //     });
    // });

    // let dumb_vertices = vertices
    //     .clone()
    //     .iter()
    //     .map(|v| Vertex::new(get_display_name_for(v.user_id.clone())))
    //     .collect::<Vec<_>>();
    //
    // println!("vertices = {:#?}", vertices);
    //
    // let mut file = std::fs::File::create("graph.ron").unwrap();
    // to_writer_pretty(&mut file, &dumb_vertices, Default::default()).unwrap();

    // let others = time_it!("sorting others by count" => {
    //     let mut others = others.into_iter().collect::<Vec<_>>();
    //     others.sort_by(|a, b| b.1.cmp(&a.1));
    //     others
    // });
    //
    // let others = others
    //     .iter()
    //     .map(|(user_id, count)| (get_display_name_for(user_id.clone()), count))
    //     .collect::<Vec<_>>();

    // println!("top 10: {:#?}", others.iter().take(10).collect::<Vec<_>>());

    // debug!(others);
}

pub fn get_uuid_of(display_name: String) -> String {
    let q = "select *
        from gamelog_join_leave
        where display_name like ?
        and user_id is not ''";

    let row = time_it!(format!("finding uuid of {}", &display_name) => {
        smol::block_on(
            async {
                sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
                    .bind(display_name.clone())
                    .fetch_one(&establish_connection().await).await
            }
        ).unwrap()
    });

    if row.user_id.is_empty() {
        panic!("No user_id found for {}", display_name);
    }

    row.user_id
}

pub fn get_display_name_for(user_id: String) -> String {
    let q = "select *
        from gamelog_join_leave
        where user_id like ?
        order by created_at desc
        limit 1";

    smol::block_on(async {
        sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
            .bind(user_id.clone())
            .fetch_one(&establish_connection().await)
            .await
    })
    .unwrap()
    .display_name
}

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
        .map(|row| row.into())
        .filter_map(|row: GamelogJoinLeave| row.location)
        .collect()
}

pub fn get_others_for(
    user_id: String,
    conn: &SqlitePool,
    locations: HashSet<WorldInstance>,
) -> HashMap<String, u32> {
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
                        .fetch_all(conn).await
                })
            ).unwrap();


            let rows = rows
                .iter()
                .cloned()
                .map(|row| row.into())
                .collect::<Vec<GamelogJoinLeave>>();


            rows.iter()
                .filter(|row| {
                    row.location.is_some() && row.user_id.is_some() && row.user_id.clone().unwrap() != user_id && row.user_id.clone().unwrap() != "usr_c2a23c47-1622-4b7a-90a4-b824fcaacc69"
                })
                .map(|row| row.user_id.clone().unwrap().clone())
                .collect::<HashSet<_>>()
        })
        .filter(|other| !other.is_empty())
        .collect::<Vec<_>>()
    });

    let mut everyone_else = HashMap::new();

    others.iter().for_each(|other| {
        other.iter().cloned().for_each(|user_id| {
            let old = everyone_else.get(&user_id).unwrap_or(&0_u32);
            everyone_else.insert(user_id, old + 1);
        });
    });

    everyone_else
}
