#![feature(lazy_cell)]
#![feature(async_closure)]
#![feature(let_chains)]

use std::collections::{HashMap, HashSet};
use std::convert::Into;
use std::hash::Hash;
use std::sync::{Arc, LazyLock, OnceLock};
use std::time::Instant;

use petgraph::dot::Config;
use petgraph::visit::Walker;
use petgraph::Graph;
use ron::ser::{to_writer_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::task::JoinSet;
use tokio::time::sleep;

use vrcx_insights::zaphkiel::cache::{ArcStrMap, ArcStrSet};
use vrcx_insights::zaphkiel::db::establish_connection;
use vrcx_insights::zaphkiel::gamelog_join_leave::{GamelogJoinLeave, GamelogJoinLeaveRow};
use vrcx_insights::zaphkiel::world_instance::WorldInstance;

static KAT_ID: LazyLock<Id> =
    LazyLock::new(|| Id("usr_c2a23c47-1622-4b7a-90a4-b824fcaacc69".into()));
static KAT_DISPLAY_NAME: OnceLock<Name> = OnceLock::new();
static KAT_EXISTS: LazyLock<bool> = LazyLock::new(|| std::fs::metadata(".kat").is_ok());

#[allow(clippy::too_many_lines)]
#[tokio::main(flavor = "multi_thread", worker_threads = 15)]
async fn main() {
    // let mut set = JoinSet::new();
    //
    // for i in 0..10_usize {
    //     set.spawn(async move {
    //         sleep(tokio::time::Duration::from_secs(i as u64)).await;
    //         i
    //     });
    // }
    //
    // let mut seen = [false; 10];
    // while let Some(res) = set.join_next().await {
    //     let idx = res.unwrap();
    //     println!("{idx}");
    //     seen[idx] = true;
    // }
    //
    // for i in seen {
    //     assert!(i);
    // }
    //
    // println!("done");

    let start = Instant::now();

    dbg!(start.elapsed());
    let owner_id: Id = std::fs::read_to_string("owner_id.txt")
        .unwrap()
        .trim()
        .into();
    dbg!(start.elapsed());

    let conn = establish_connection().await;
    let conn = Arc::new(conn);

    let cache = ArcStrMap::new_empty_arc_str();

    dbg!(start.elapsed());
    let _owner_name = get_display_name_for(owner_id.clone(), conn.clone(), cache.clone());
    dbg!(start.elapsed());

    KAT_DISPLAY_NAME
        .set(get_display_name_for(KAT_ID.to_string().into(), conn.clone(), cache.clone()).await)
        .unwrap();

    dbg!(start.elapsed());
    let latest_name = get_display_name_for(owner_id.clone(), conn.clone(), cache.clone()).await;
    dbg!(start.elapsed());

    dbg!(start.elapsed());
    let locations = get_locations_for(owner_id.clone(), conn.clone()).await;
    dbg!(start.elapsed());

    dbg!(start.elapsed());
    let others: ArcStrMap<Id, u32> = get_others_for(owner_id, conn.clone(), locations).await;
    dbg!(start.elapsed());

    dbg!(start.elapsed());
    let mut others_names = ArcStrMap::new();
    dbg!(start.elapsed());

    dbg!(start.elapsed());
    // let mut set = JoinSet::new();
    let mut handles = JoinSet::new();

    // for i in 0..10_usize {
    for (user_id, count) in others.clone().into_iter() {
        let conn = conn.clone();
        let cache = cache.clone();
        // set.spawn(async move {
        handles.spawn(async move {
            if user_id.is_kat() {
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!early exit");
                return None;
            }
            // sleep(Duration::from_secs(i as u64)).await;
            sleep(tokio::time::Duration::from_secs(10)).await;
            let display_name = get_display_name_for(user_id, conn, cache).await;
            println!("{}", display_name.to_string());

            if display_name.is_kat() {
                println!("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!early exit");
                return None;
            }

            Some((display_name, count))
        });
    }
    dbg!(start.elapsed());

    // while let Some(res) = set.join_next().await {
    while let Some(res) = handles.join_next().await {
        // let idx = res.unwrap();
        let res = res.unwrap();
        println!("{res:?}");
        match res {
            None => {}
            Some((name, count)) => {
                others_names.insert(name, count);
            }
        }
    }
    dbg!(start.elapsed());

    dbg!(&others_names);

    let others_names_len = dbg! {others_names.len()};

    let mut graph: ArcStrMap<Name, ArcStrMap<Name, u32>> = ArcStrMap::new();
    graph.insert(latest_name, others_names);

    dbg!(start.elapsed());
    let mut handles = JoinSet::new();
    for (user_id, _) in others.clone().into_iter() {
        let conn = conn.clone();
        let cache = cache.clone();
        let _ = handles.spawn(async move {
            let latest_name =
                get_display_name_for(user_id.clone(), conn.clone(), cache.clone()).await;
            let locations = get_locations_for(user_id.clone(), conn.clone()).await;
            let others = get_others_for(user_id.clone(), conn.clone(), locations).await;

            let mut others_name = ArcStrMap::new();
            for (user_id, count) in others.iter() {
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
    dbg!(start.elapsed());

    dbg!(start.elapsed());
    handles.join_next().await.into_iter().for_each(|handle| {
        let _ = handle
            .unwrap()
            .and_then(|(node, edges)| graph.insert(node, edges));
    });
    dbg!(start.elapsed());

    dbg!(start.elapsed());
    let graph = graph
        .iter()
        .filter_map(|(node, edges)| {
            if node.is_kat() && *KAT_EXISTS {
                return None;
            }
            let edges = edges
                .iter()
                .filter(|(edge, _)| !(edge.is_kat() && *KAT_EXISTS))
                .map(|(edge, count)| (edge.to_owned(), count.to_owned()))
                .collect::<ArcStrMap<Name, u32>>();
            if edges.is_empty() {
                return None;
            }
            Some((node, edges))
        })
        .map(|(node, edges)| (node.to_owned(), edges))
        .collect::<ArcStrMap<Name, ArcStrMap<Name, u32>>>();
    dbg!(start.elapsed());

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
                    let percentage = f64::from(*count) / f64::from(total) * 100_f64;
                    let percentile = f64::from(*count) / f64::from(max) * 100_f64;
                    (
                        k.clone(),
                        (
                            *count,
                            (percentage * 100_f64).round() / 100_f64,
                            (percentile * 100_f64).round() / 100_f64,
                        ),
                    )
                })
                .collect::<ArcStrMap<Name, _>>();
            if new_others.is_empty() {
                None
            } else {
                Some((name.clone(), new_others))
            }
        })
        .collect::<ArcStrMap<Name, ArcStrMap<Name, _>>>();

    let mut graph2_sorted = graph2
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();

    graph2_sorted.sort_by(|a, b| {
        let (_, a) = a;
        let (_, b) = b;
        let a_len = a.iter().map(|(_, (count, _, _))| count).sum::<u32>();
        let b_len = b.iter().map(|(_, (count, _, _))| count).sum::<u32>();
        b_len.cmp(&a_len)
    });

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
        let mut adjacency_matrix: ArcStrMap<Name, ArcStrSet<Name>> = ArcStrMap::new();
        for (name, others) in &graph2_sorted {
            let mut current_list: ArcStrSet<Name> = {
                match adjacency_matrix.get(name) {
                    None => {
                        let ret = ArcStrSet::new();
                        adjacency_matrix.insert(name.clone(), ret.clone());
                        ret
                    }
                    Some(set) => set.clone(),
                }
            };
            for other in others.keys() {
                current_list.insert_set(other.clone());
            }

            for other in current_list.keys() {
                let mut other_list = match adjacency_matrix.get(other) {
                    None => {
                        let ret = ArcStrSet::new();
                        adjacency_matrix.insert(other.clone(), ret.clone());
                        ret
                    }
                    Some(ret) => ret.clone(),
                };
                other_list.insert_set(name.clone());
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

        for (edge, weight) in edges.iter() {
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

    println!("\x07Total run time => {:?}", start.elapsed());
}

#[derive(Debug, Eq, PartialEq, Hash)]
pub struct Id(Arc<str>);

impl Clone for Id {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<Arc<str>> for Id {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

impl From<&Arc<str>> for Id {
    fn from(value: &Arc<str>) -> Self {
        Self(value.clone())
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl Into<Arc<str>> for Id {
    fn into(self) -> Arc<str> {
        self.0
    }
}

impl Id {
    pub fn is_kat(&self) -> bool {
        *self == *KAT_ID
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Deserialize, Serialize)]
pub struct Name(Arc<str>);

impl Clone for Name {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl ToString for Name {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl From<Arc<str>> for Name {
    fn from(value: Arc<str>) -> Self {
        Self(value)
    }
}

impl Into<Arc<str>> for Name {
    fn into(self) -> Arc<str> {
        self.0
    }
}

impl From<&Name> for Arc<str> {
    fn from(value: &Name) -> Self {
        value.into()
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Name {
    pub fn is_kat(&self) -> bool {
        self == KAT_DISPLAY_NAME.get().unwrap()
    }
}

#[must_use]
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

pub async fn get_display_name_for(
    user_id: Id,
    pool: Arc<SqlitePool>,
    mut cache: ArcStrMap<Id, Arc<str>>,
) -> Name {
    if let Some(display_name) = cache.get(&user_id) {
        return display_name.clone().into();
    }

    let q = "select *
        from gamelog_join_leave
        where user_id like ?
        order by created_at desc
        limit 1";

    println!("{}", user_id.to_string());

    let name: String = sqlx::query_as::<_, GamelogJoinLeaveRow>(q)
        .bind(user_id.to_string())
        .fetch_one(pool.as_ref())
        .await
        .unwrap()
        .display_name;

    let name: Arc<str> = name.into();

    cache.insert(user_id, name.clone());

    name.into()
}

#[must_use]
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
pub async fn get_others_for(
    user_id: Id,
    conn: Arc<SqlitePool>,
    locations: HashSet<WorldInstance>,
) -> ArcStrMap<Id, u32> {
    let mut others = vec![];
    let len = locations.len();
    let mut handles = JoinSet::new();
    for (idx, location) in locations.into_iter().enumerate() {
        let conn = conn.clone();
        let user_id = user_id.clone();
        let _ = handles.spawn(async move {
            let q = "select *
                    from gamelog_join_leave
                    where location like ?
                    and location != ''
                    and user_id != ?
                    and user_id is not ''";

            let prefix = location.get_prefix();
            let location = format!("{}%", prefix);

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
                Some(other)
            }
        });
    }
    handles.join_next().await.into_iter().for_each(|handle| {
        let _ = handle.unwrap().map(|set| {
            others.push(set);
            Some(())
        });
    });

    let mut everyone_else = ArcStrMap::new();

    for other in others {
        for user_id in other {
            let old = everyone_else.get(&user_id).unwrap_or(&0_u32);
            everyone_else.insert(user_id, old + 1);
        }
    }

    everyone_else.into()
}
