#![feature(associated_type_defaults)]

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use petgraph::dot::Config;
use petgraph::graph::NodeIndex;
use petgraph::visit::Bfs;
use petgraph::Graph;
use ron::ser::{to_writer_pretty, PrettyConfig};
use tokio::task::JoinSet;
use tokio::time::sleep;

use vrcx_insights::zaphkiel::db::establish_connection;
use vrcx_insights::zaphkiel::is_kat::{Id, IsKat, Name, KAT_DISPLAY_NAME, KAT_EXISTS, KAT_ID};
use vrcx_insights::zaphkiel::metadata::Metadata;
use vrcx_insights::{get_display_name_for, get_locations_for, get_others_for};

#[allow(clippy::too_many_lines)]
#[tokio::main(flavor = "multi_thread", worker_threads = 15)]
async fn main() {
    let start = Instant::now();

    let owner_id: Id = std::fs::read_to_string("owner_id.txt")
        .unwrap()
        .trim()
        .into();

    let conn = establish_connection().await;
    let conn = Arc::new(conn);

    let cache = Arc::new(RwLock::new(HashMap::new()));

    let _owner_name = get_display_name_for(owner_id.clone(), conn.clone(), cache.clone()).await;

    KAT_DISPLAY_NAME
        .set(get_display_name_for(KAT_ID.to_string().into(), conn.clone(), cache.clone()).await)
        .unwrap();

    let latest_name = get_display_name_for(owner_id.clone(), conn.clone(), cache.clone()).await;

    let locations = get_locations_for(owner_id.clone(), conn.clone()).await;

    let others: Arc<HashMap<Id, u32>> = get_others_for(owner_id, conn.clone(), locations)
        .await
        .into();

    let mut others_names: HashMap<Name, u32> = HashMap::new();

    let mut handles = JoinSet::new();

    for (user_id, count) in others.iter() {
        let conn = conn.clone();
        let cache = cache.clone();
        let (user_id, count) = (user_id.clone(), count.to_owned());
        handles.spawn(async move {
            if user_id.is_kat() {
                return None;
            }
            sleep(tokio::time::Duration::from_secs(10)).await;
            let display_name =
                get_display_name_for(user_id.clone(), conn.clone(), cache.clone()).await;

            if display_name.is_kat() {
                return None;
            }

            Some((display_name, count))
        });
    }

    while let Some(res) = handles.join_next().await {
        let res = res.unwrap();
        match res {
            None => {}
            Some((name, count)) => {
                others_names.insert(name, count);
            }
        }
    }

    let mut graph: HashMap<Name, HashMap<Name, u32>> = HashMap::new();
    graph.insert(latest_name, others_names);

    let mut handles = JoinSet::new();
    for (user_id, _) in others.iter() {
        let conn = conn.clone();
        let cache = cache.clone();
        let user_id = user_id.clone();
        handles.spawn(async move {
            let latest_name =
                get_display_name_for(user_id.clone(), conn.clone(), cache.clone()).await;
            let locations = get_locations_for(user_id.clone(), conn.clone()).await;
            let others = get_others_for(user_id.clone(), conn.clone(), locations).await;

            let mut others_name = HashMap::new();
            for (user_id, count) in &others {
                let name = get_display_name_for(user_id.clone(), conn.clone(), cache.clone()).await;
                others_name.insert(name, count.to_owned());
            }

            if latest_name.is_kat() && *KAT_EXISTS {
                None
            } else {
                Some((latest_name, others_name))
            }
        });
    }
    handles.join_next().await.into_iter().for_each(|handle| {
        let _ = handle
            .unwrap()
            .and_then(|(node, edges)| graph.insert(node, edges));
    });

    let graph = graph
        .iter()
        .map(|(name, edges)| {
            (
                name,
                edges
                    .iter()
                    .map(|(name, count)| (name.to_owned(), count.to_owned()))
                    .collect(),
            )
        })
        .map(|(name, edges)| (name.to_owned(), edges))
        .collect::<HashMap<Name, HashMap<Name, u32>>>();

    let graph = graph
        .iter()
        .filter_map(|(node, edges)| {
            if node.is_kat() && *KAT_EXISTS {
                return None;
            }
            let edges = edges
                .iter()
                .filter(|(edge, _)| !(edge.is_kat() && *KAT_EXISTS))
                .map(|(edge, count)| (edge.clone().0, count.to_owned()))
                .collect::<HashMap<Arc<str>, u32>>();
            if edges.is_empty() {
                return None;
            }
            Some((node, edges))
        })
        .map(|(node, edges)| (node.clone().0, edges))
        .collect::<HashMap<Arc<str>, HashMap<Arc<str>, u32>>>();

    if std::fs::metadata("graph.ron").is_ok() {
        std::fs::remove_file("graph.ron").unwrap();
    }
    to_writer_pretty(
        std::fs::File::create("graph.ron").unwrap(),
        &graph,
        PrettyConfig::default(),
    )
    .unwrap();

    let graph2 = graph
        .iter()
        .filter_map(|a| {
            let (name, others) = a;
            let total = others.values().sum::<u32>();
            let max = *others.values().max().unwrap() + 1;
            let new_others = others
                .iter()
                .map(|(k, count)| {
                    let count = *count;
                    let percentage =
                        ((f64::from(count) * 10_000_f64) / f64::from(total)).round() / 100_f64;
                    let percentile =
                        ((f64::from(count) * 10_000_f64) / f64::from(max)).round() / 100_f64;
                    (
                        k.clone(),
                        Metadata {
                            count,
                            max,
                            total,
                            percentage,
                            percentile,
                        },
                    )
                })
                .collect::<HashMap<_, Metadata>>();
            if new_others.is_empty() {
                None
            } else {
                Some((name.clone(), new_others))
            }
        })
        .collect::<HashMap<_, HashMap<_, _>>>();

    let graph2_sorted = graph2
        .iter()
        .map(|(k, v)| {
            (k.clone(), {
                let mut v = v.clone().into_iter().collect::<Vec<_>>();
                v.sort_by_key(|(_, Metadata { count, .. })| *count);
                v.reverse();
                v
            })
        })
        .collect::<BTreeMap<_, _>>();

    let graph2_sorted_set: HashMap<Arc<str>, HashMap<Arc<str>, _>> = graph2_sorted
        .iter()
        .map(|(name, v)| {
            (name.clone(), {
                v.iter()
                    .map(|(a, b)| (a.to_owned(), b.to_owned()))
                    .collect()
            })
        })
        .collect();

    if std::fs::metadata("graph2_sorted.ron").is_ok() {
        std::fs::remove_file("graph2_sorted.ron").unwrap();
    }
    to_writer_pretty(
        std::fs::File::create("graph2_sorted.ron").unwrap(),
        &graph2_sorted,
        PrettyConfig::default(),
    )
    .unwrap();

    let undirected_graph = {
        let mut adjacency_matrix: HashMap<_, HashSet<_>> = HashMap::new();
        for (name, others) in graph2_sorted_set {
            #[allow(clippy::option_if_let_else)] // adjacency_matrix is getting borrowed twice
            let mut current_list: HashSet<_> = match adjacency_matrix.get(&name) {
                None => {
                    let ret = HashSet::new();
                    adjacency_matrix.insert(name.clone(), ret.clone());
                    ret
                }
                Some(set) => set.clone(),
            };
            for other in others.keys() {
                current_list.insert(other.clone());
            }

            for other in &current_list {
                #[allow(clippy::option_if_let_else)] // we're borrowing adjacency_matrix twice
                let mut other_list = match adjacency_matrix.get(other) {
                    None => {
                        let ret = HashSet::new();
                        adjacency_matrix.insert(other.clone(), ret.clone());
                        ret
                    }
                    Some(ret) => ret.clone(),
                };
                other_list.insert(name.clone());
                adjacency_matrix.insert(other.clone(), other_list);
            }
            adjacency_matrix.insert(name.clone(), current_list.clone());
        }
        adjacency_matrix
    };

    let sorted_undirected_graph = {
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
    };

    if std::fs::metadata("sorted_undirected_graph.ron").is_ok() {
        std::fs::remove_file("sorted_undirected_graph.ron").unwrap();
    }
    to_writer_pretty(
        std::fs::File::create("sorted_undirected_graph.ron").unwrap(),
        &sorted_undirected_graph,
        PrettyConfig::default(),
    )
    .unwrap();

    let mut petgraph = Graph::new();
    let mut dot_idxs = HashMap::new();

    for (node, edges) in graph2_sorted {
        if node.is_kat() && !*KAT_EXISTS {
            continue;
        }

        for (edge, weight) in &edges {
            if edge.is_kat() && !*KAT_EXISTS {
                continue;
            }

            dot_idxs
                .entry(node.clone())
                .or_insert_with(|| petgraph.add_node(node.clone()));
            let node_idx = dot_idxs.get(&node).unwrap().to_owned();

            dot_idxs
                .entry(edge.clone())
                .or_insert_with(|| petgraph.add_node(edge.clone()));
            let edge_idx = dot_idxs.get(edge).unwrap().to_owned();

            petgraph.add_edge(node_idx, edge_idx, weight.to_owned());
        }
    }

    let dot_edge_no_label = petgraph::dot::Dot::with_config(&petgraph, &[Config::EdgeNoLabel]);
    let dot_edge_with_label = petgraph::dot::Dot::new(&petgraph);

    std::fs::write("dot_edge_no_label.dot", format!("{dot_edge_no_label:?}")).unwrap();
    std::fs::write(
        "dot_edge_with_label.dot",
        format!("{dot_edge_with_label:?}"),
    )
    .unwrap();

    println!("starting FuzzyDBSCAN");

    println!("\x07Total run time => {:?}", start.elapsed());
}

struct Nu<'a>(pub &'a Graph<Arc<str>, Metadata>, NodeIndex);

impl<'a> fuzzy_dbscan::MetricSpace for Nu<'a> {
    fn distance(&self, other: &Self) -> f64 {
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
        // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA

        // use petgraph::Graph;
        // use petgraph::visit::Bfs;
        //
        // let mut graph = Graph::<_,()>::new();
        // let a = graph.add_node(0);
        //
        // let mut bfs = Bfs::new(&graph, a);
        // while let Some(nx) = bfs.next(&graph) {
        //     // we can access `graph` mutably here still
        //     graph[nx] += 1;
        // }
        //
        // assert_eq!(graph[a], 1);

        let Nu(graph, idx) = other;

        let graph = *graph;

        let mut bfs = Bfs::new(graph, *idx);
        while let Some(nx) = bfs.next(graph) {
            todo!()
        }

        todo!()
    }
}

// pub trait Node {
//     fn get_edge(&self) -> (Arc<str>, Metadata);
//     fn get_neighbors(&self) -> Vec<(Arc<str>, Metadata)>;
//     fn get_hashmap() -> &'static HashMap<Arc<str>, HashMap<Arc<str>, Metadata>>;
// }
//
// pub trait Bfs {
//     /// return type is (Number of Hops, Distance between them)
//     fn get_distance(&self, other: &Self) -> (u64, f64)
//     where
//         Self: Node + Sized + Eq,
//     {
//         let hashmap = Self::get_hashmap();
//
//         let recurse = |acc: (u64, f64), goal: &Self, curr: &Self| -> (u64, f64) {
//             if curr == goal {
//                 return acc;
//             }
//
//             let neighbors = curr.get_neighbors();
//
//             for (name, _) in &neighbors {
//                 let other_edges = hashmap.get(name).unwrap();
//             }
//             todo!()
//         };
//
//         todo!()
//     }
// }
//
// pub struct Edge<T>(T)
// where
//     T: Node + Bfs;
//
// impl<T> fuzzy_dbscan::MetricSpace for Edge<T>
// where
//     T: Node + Eq,
// {
//     fn distance(&self, other: &Self) -> f64 {
//         let node = &self.0;
//         let other = &other.0;
//         let (_hops, dist) = node.get_distance(other);
//
//         dist
//     }
// }
