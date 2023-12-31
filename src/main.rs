extern crate core;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use async_compat::Compat;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use ron::ser::to_writer_pretty;
use smol::io::AsyncReadExt;
use sqlx::{Row, SqlitePool};

use vrcx_insights::time_it;
use vrcx_insights::zaphkiel::db::establish_connection;
use vrcx_insights::zaphkiel::gamelog_join_leave::{GamelogJoinLeave, GamelogJoinLeaveRow};
use vrcx_insights::zaphkiel::utils::get_pb;
use vrcx_insights::zaphkiel::world_instance::WorldInstance;

fn main() {
    let start = Instant::now();

    let owner_id: String = std::fs::read_to_string("owner_id.txt").unwrap();

    let conn = time_it!(at once | "establishing connection to database" =>
        smol::block_on(async {establish_connection().await})
    );

    let _owner_name = get_display_name_for(
        owner_id.clone(),
        &conn,
        Arc::new(RwLock::new(HashMap::new())),
    );

    let cache = Arc::new(RwLock::new(HashMap::new()));

    let latest_name = time_it!(at once | "getting latest name of owner" =>
        get_display_name_for(owner_id.clone(), &conn, cache.clone())
    );

    let locations = time_it!(at once | "getting the locations the user was in" =>
        get_locations_for(owner_id.clone(), &conn)
    );

    let others = time_it!("finding out the other users the user has seen" =>
        get_others_for(owner_id.clone(), &conn, locations)
    );

    let others_names = time_it!(at once | "getting names for other users" => others
        .par_iter()
        .progress_with(get_pb(others.len() as u64, "Getting names"))
        .map(|(user_id, count)| {
            (
                get_display_name_for(user_id.clone(), &conn, cache.clone()),
                *count,
            )
        })
        .collect::<HashMap<_, _>>());

    let graph = Arc::new(RwLock::new(HashMap::new()));
    graph
        .clone()
        .write()
        .unwrap()
        .insert(latest_name.clone(), others_names.clone());

    time_it!(at once | "generating the graph" => others
    .par_iter()
    .progress_with(get_pb(others.len() as u64, "Generating graph"))
    .for_each(|(user_id, _)| {
        let latest_name = get_display_name_for(user_id.clone(), &conn, cache.clone());
        let locations = get_locations_for(user_id.clone(), &conn);
        let others = get_others_for(user_id.clone(), &conn, locations);
        let others = others
            .par_iter()
            .map(|(user_id, count)| {
                (
                    get_display_name_for(user_id.clone(), &conn, cache.clone()),
                    *count,
                )
            })
            .collect::<HashMap<_, _>>();
        graph.clone().write().unwrap().insert(latest_name, others);
    }));

    time_it!("writing to graph.ron" => {
        if std::fs::metadata("graph.ron").is_ok() {
            std::fs::remove_file("graph.ron").unwrap();
        }
        to_writer_pretty(
            std::fs::File::create("graph.ron").unwrap(),
            &graph,
            Default::default(),
        )
        .unwrap()
    });

    let graph2 = time_it!(at once | "filtering graph" => graph
        .clone()
        .read()
        .unwrap()
        .par_iter()
        .progress_with(get_pb(
            graph.read().unwrap().len() as u64,
            "Filtering graph",
        ))
        .filter_map(|a| {
            let (name, others) = a;
            let total = others.values().sum::<u32>();
            let max = *others.values().max().unwrap() + 1;
            let new_others = others
                .into_par_iter()
                .filter_map(|(k, count)| {
                    let percentage = *count as f64 / total as f64 * 100_f64;
                    let percentile = *count as f64 / max as f64 * 100_f64;
                    if percentile > 50_f64 || percentage > 5_f64 {
                        Some((
                            k.clone(),
                            (*count, (percentage * 100_f64).round() / 100_f64,
                            (percentile * 100_f64).round() / 100_f64)
                        ))
                    } else {
                        None
                    }
                })
                .collect::<HashMap<String, _>>();
            if new_others.is_empty() {
                None
            } else {
                Some((name.clone(), new_others))
            }
        })
        .collect::<HashMap<String, HashMap<String, _>>>()
    );

    let mut graph2_sorted = time_it!(at once | "duplicating graph" => graph2
        .par_iter()
        .progress_with(get_pb(graph2.len() as u64, "duplicating graph"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>());

    time_it!("sorting graph by weighing adjacency list" => graph2_sorted.sort_by(|a, b| {
        let (_, a) = a;
        let (_, b) = b;
        let a_len = a.iter().map(|(_, (count, _, _))| count).sum::<u32>();
        let b_len = b.iter().map(|(_, (count, _, _))| count).sum::<u32>();
        b_len.cmp(&a_len)
    }));

    time_it!("writing to graph2_sorted.ron" => {
        if std::fs::metadata("graph2_sorted.ron").is_ok() {
            std::fs::remove_file("graph2_sorted.ron").unwrap();
        }
        to_writer_pretty(
            std::fs::File::create("graph2_sorted.ron").unwrap(),
            &graph2_sorted,
            Default::default(),
        )
        .unwrap()
    });

    let undirected_graph = time_it!("convert the directed graph into an undirected graph" => {
        let mut adjacency_matrix = HashMap::new();
        for (name, others) in graph2_sorted.iter() {
            let mut current_list = adjacency_matrix
                .entry(name.clone())
                .or_insert_with(HashSet::new)
                .clone();
            for (other, _) in others.iter() {
                current_list.insert(other.clone());
            }

            for other in current_list.iter() {
                let mut other_list = adjacency_matrix
                    .entry(other.clone())
                    .or_insert_with(HashSet::new)
                    .clone();
                other_list.insert(name.clone());
                adjacency_matrix.insert(other.clone(), other_list);
            }
            adjacency_matrix.insert(name.clone(), current_list.clone());
        }
        adjacency_matrix
    });

    let sorted_undirected_graph = time_it!("sorting the undirected graph by number of entries" => {
        let mut list = undirected_graph
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        list.sort_by(|a, b| {
            let (_, a) = a;
            let (_, b) = b;
            let a_len = a.len();
            let b_len = b.len();
            b_len.cmp(&a_len)
        });
        list
    });

    time_it!("writing to sorted_undirected_graph.ron" => {
        if std::fs::metadata("sorted_undirected_graph.ron").is_ok() {
            std::fs::remove_file("sorted_undirected_graph.ron").unwrap();
        }
        to_writer_pretty(
            std::fs::File::create("sorted_undirected_graph.ron").unwrap(),
            &sorted_undirected_graph,
            Default::default(),
        )
        .unwrap()
    });

    println!("\x07Total run time => {:?}", start.elapsed());
}

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

    if row.user_id.is_empty() {
        panic!("No user_id found for {}", display_name);
    }

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

    rows.par_iter()
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
    let others = locations
        .par_iter()
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

            let rows = rows
                .into_par_iter()
                .map(|row| row.into())
                .collect::<Vec<GamelogJoinLeave>>();

            rows.into_par_iter()
                .map(|row| row.user_id.unwrap())
                .collect::<HashSet<_>>()
        })
        .filter(|other| !other.is_empty())
        .collect::<Vec<_>>();

    let mut everyone_else = HashMap::new();

    others.into_iter().for_each(|other| {
        other.into_iter().for_each(|user_id| {
            let old = everyone_else.get(&user_id).unwrap_or(&0_u32);
            everyone_else.insert(user_id, old + 1);
        });
    });

    everyone_else
}
