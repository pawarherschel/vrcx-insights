extern crate core;

use std::collections::{HashMap, HashSet};
use std::fs;
use std::time::Instant;

use async_compat::Compat;
use dashmap::{DashMap, DashSet};
use indicatif::ParallelProgressIterator;
use petgraph::dot::Config;
use petgraph::Graph;
use rayon::prelude::*;
use ron::ser::{to_writer_pretty, PrettyConfig};
use sqlx::SqlitePool;

use vrcx_insights::time_it;
use vrcx_insights::zaphkiel::db::establish_connection;
use vrcx_insights::zaphkiel::gamelog_join_leave::{GamelogJoinLeave, GamelogJoinLeaveRow};
use vrcx_insights::zaphkiel::utils::get_pb;
use vrcx_insights::zaphkiel::world_instance::WorldInstance;

#[allow(clippy::too_many_lines)]
fn main() {
    let start = Instant::now();

    let owner_id: String = std::fs::read_to_string("owner_id.txt").unwrap();

    let conn = time_it!(at once | "establishing connection to database" =>
        smol::block_on(async {establish_connection().await})
    );

    let _owner_name = get_display_name_for(owner_id.clone(), &conn, &DashMap::new());

    let cache = DashMap::new();

    let latest_name = time_it!(at once | "getting latest name of owner" =>
        get_display_name_for(owner_id.clone(), &conn, &cache)
    );

    let locations = time_it!(at once | "getting the locations the user was in" =>
        get_locations_for(owner_id.clone(), &conn)
    );

    let others = time_it!("finding out the other users the user has seen" =>
        get_others_for(owner_id, &conn, locations)
    );

    let others_names = time_it!(at once | "getting names for other users" => others
        .par_iter()
        .progress_with(get_pb(others.len() as u64, "Getting names"))
        .map(|it| {
                let user_id = it.key();
                let count = it.value();
            (
                get_display_name_for(user_id.clone(), &conn, &cache),
                *count,
            )
        })
        .collect::<DashMap<_, _>>());

    let graph = DashMap::new();
    graph.insert(latest_name, others_names);

    time_it!(at once | "generating the graph" => others
    .par_iter()
    .progress_with(get_pb(others.len() as u64, "Generating graph"))
    .for_each(|it| {
            let user_id = it.key();
        let latest_name = get_display_name_for(user_id.clone(), &conn, &cache);
        let locations = get_locations_for(user_id.clone(), &conn);
        let others = get_others_for(user_id.clone(), &conn, locations);
        let others = others
            .par_iter()
            .map(|it| {
                let user_id = it.key();
                let count = it.value();
                (
                    get_display_name_for(user_id.clone(), &conn, &cache),
                    *count,
                )
            })
            .collect::<DashMap<_, _>>();
        graph.insert(latest_name, others);
    }));

    time_it!("writing to graph.ron" => {
        let graph = graph.par_iter().map(|it| (it.key().to_owned(),
            it.value().iter().map(|it|(it.key().to_owned(), it.value().to_owned())).collect::<HashMap<_,_>>()
        )).collect::<HashMap<_,_>>();

        if std::fs::metadata("graph.ron").is_ok() {
            std::fs::remove_file("graph.ron").unwrap();
        }
        to_writer_pretty(
            std::fs::File::create("graph.ron").unwrap(),
            &graph,
            PrettyConfig::default(),
        )
        .unwrap();
    });
    let graph2 = time_it!(at once | "filtering graph" =>
        graph
            .par_iter()
            .progress_with(get_pb(graph.len() as u64, "Filtering graph"))
            .filter_map(|it| {
                let name = it.key();
                let others = it.value();
                let total = others.iter().map(|it| it.value().to_owned()).sum::<u32>();
                let max = others.iter().map(|it| it.value().to_owned()).max().unwrap() + 1;
                let new_others = others
                    .into_par_iter()
                    .filter_map(|it| {
                        let k = it.key();
                        let count = it.value();
                        let percentage = f64::from(*count) / f64::from(total) * 100_f64;
                        let percentile = f64::from(*count) / f64::from(max) * 100_f64;
                        if percentile > 50_f64 || percentage > 5_f64 {
                            Some((
                                k.clone(),
                                (
                                    *count,
                                    (percentage * 100_f64).round() / 100_f64,
                                    (percentile * 100_f64).round() / 100_f64,
                                ),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect::<DashMap<String, _>>();
                if new_others.is_empty() {
                    None
                } else {
                    Some((name.clone(), new_others))
                }
            })
            .collect::<DashMap<String, DashMap<String, _>>>()
    );

    let mut graph2_sorted = time_it!(at once | "duplicating graph" => graph2
        .par_iter()
        .progress_with(get_pb(graph2.len() as u64, "duplicating graph"))
        .map(|it|{
            let k = it.key();
            let v = it.value().iter().map(|it| (it.key().to_owned(), it.value().to_owned())).collect::<HashMap<_,_>>();
            (k.clone(), v.clone())})
        .collect::<Vec<_>>());

    time_it!("sorting graph by weighing adjacency list" => graph2_sorted.sort_by(|a, b| {
        let (_, a) = a;
        let (_, b) = b;
        let a_len = a.iter().map(|it| it.1.0).sum::<u32>();
        let b_len = b.iter().map(|it| it.1.0).sum::<u32>();
        b_len.cmp(&a_len)
    }));

    time_it!("writing to graph2_sorted.ron" => {
        if std::fs::metadata("graph2_sorted.ron").is_ok() {
            std::fs::remove_file("graph2_sorted.ron").unwrap();
        }
        to_writer_pretty(
            std::fs::File::create("graph2_sorted.ron").unwrap(),
            &graph2_sorted,
            PrettyConfig::default(),
        )
        .unwrap();
    });

    let undirected_graph = time_it!("convert the directed graph into an undirected graph" => {
        let adjacency_matrix = DashMap::new();
        for (name, others) in &graph2_sorted {
            let current_list = adjacency_matrix
                .entry(name.clone())
                .or_insert_with(DashSet::new)
                .clone();
            for other in others.keys() {
                current_list.insert(other.clone());
            }

            for other in current_list.clone() {
                let other_list = adjacency_matrix
                    .entry(other.clone())
                    .or_insert_with(DashSet::new)
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
            .map(|it| (it.key().clone(),
            it.value().iter().map(|it| it.key().to_owned()).collect::<HashSet<_>>()
        ))
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
            PrettyConfig::default(),
        )
        .unwrap();
    });

    let mut petgraph = Graph::new();
    let dot_idxs = DashMap::new();

    time_it! {"converting from DashMap to petgraph" =>
        for (node, edges) in graph2_sorted {
            // if node == "Kat Sakura" {
            //     continue;
            // }

            for (edge, weight) in edges {
                // if edge == "Kat Sakura" {
                //     continue;
                // }

                dot_idxs
                    .entry(node.clone())
                    .or_insert_with(|| petgraph.add_node(node.clone()));
                let node_idx = dot_idxs.get(&node).unwrap().to_owned();

                dot_idxs
                    .entry(edge.clone())
                    .or_insert_with(|| petgraph.add_node(edge.clone()));
                let edge_idx = dot_idxs.get(&edge).unwrap().to_owned();

                petgraph.add_edge(node_idx, edge_idx, weight);
            }
        }
    }

    let dot_edge_no_label = petgraph::dot::Dot::with_config(&petgraph, &[Config::EdgeNoLabel]);
    let dot_edge_with_label = petgraph::dot::Dot::new(&petgraph);
    time_it! { "writing dots" => {
            fs::write("dot_edge_no_label.dot", format!("{dot_edge_no_label:?}")).unwrap();
            fs::write(
                "dot_edge_with_label.dot",
                format!("{dot_edge_with_label:?}"),
            )
                .unwrap();
        }
    }

    println!("\x07Total run time => {:?}", start.elapsed());
}

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
    cache: &DashMap<String, String>,
) -> String {
    if let Some(display_name) = cache.get(&user_id) {
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

    cache.insert(user_id, name.clone());

    name
}

#[must_use]
pub fn get_locations_for(user_id: String, conn: &SqlitePool) -> DashSet<WorldInstance> {
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
        .map(std::convert::Into::into)
        .filter_map(|row: GamelogJoinLeave| row.location)
        .collect()
}

#[must_use]
pub fn get_others_for(
    user_id: String,
    conn: &SqlitePool,
    locations: DashSet<WorldInstance>,
) -> DashMap<String, u32> {
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
                .map(std::convert::Into::into)
                .collect::<Vec<GamelogJoinLeave>>();

            rows.into_par_iter()
                .map(|row| row.user_id.unwrap())
                .collect::<DashSet<_>>()
        })
        .filter(|other| !other.is_empty())
        .collect::<Vec<_>>();

    let everyone_else: DashMap<String, u32> = DashMap::new();

    for other in others {
        for user_id in other {
            let old = everyone_else
                .get(&user_id)
                .map_or(0_u32, |found| found.value().to_owned());
            everyone_else.insert(user_id, old + 1);
        }
    }

    everyone_else
}
