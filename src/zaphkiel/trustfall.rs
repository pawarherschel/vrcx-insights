// use sqlx::{SqliteConnection, SqlitePool};
// use std::collections::HashSet;
// use std::fmt::Debug;
// use trustfall::provider::{
//     field_property, resolve_neighbors_with, resolve_property_with, AsVertex, BasicAdapter,
//     ContextIterator, ContextOutcomeIterator, EdgeParameters, Typename, VertexIterator,
// };
// use trustfall::FieldValue;
//
// #[derive(Debug, Clone, Default)]
// pub struct EveryoneElse {
//     pub everyone_else: HashSet<TrustfallVertex>,
// }
//
// impl From<EveryoneElse> for FieldValue {
//     fn from(value: EveryoneElse) -> Self {
//         todo!("impl From<TrustfallVertex> for FieldValue")
//     }
// }
//
// // = note: required for `EveryoneElse` to implement `Into<Box<dyn Iterator<Item = TrustfallVertex>>>`
// impl From<EveryoneElse> for Box<dyn Iterator<Item = TrustfallVertex>> {
//     fn from(value: EveryoneElse) -> Self {
//         todo!("impl From<EveryoneElse> for Box<dyn Iterator<Item = TrustfallVertex>>")
//     }
// }
//
// #[derive(Debug, Clone, Default)]
// pub struct TrustfallVertex {
//     pub name: String,
//     pub everyone_else: EveryoneElse,
// }
//
// impl TrustfallVertex {
//     fn new(name: impl ToString) -> TrustfallVertex {
//         TrustfallVertex {
//             name: name.to_string(),
//             ..Default::default()
//         }
//     }
// }
//
// impl Typename for TrustfallVertex {
//     fn typename(&self) -> &'static str {
//         todo!("impl Typename for TrustfallVertex")
//     }
// }
//
// impl<'vertex> From<TrustfallVertex> for VertexIterator<'vertex, TrustfallVertex> {
//     fn from(value: TrustfallVertex) -> Self {
//         todo!("impl From<TrustfallVertex> for VertexIterator<'vertex, TrustfallVertex>")
//     }
// }
//
// pub struct TrustfallAdapter {
//     pub db: SqlitePool,
// }
//
// impl TrustfallAdapter {
//     pub fn new(pool: SqlitePool) -> TrustfallAdapter {
//         TrustfallAdapter { db: pool }
//     }
// }
//
// impl<'vertex> BasicAdapter<'vertex> for TrustfallAdapter {
//     type Vertex = TrustfallVertex;
//
//     fn resolve_starting_vertices(
//         &self,
//         edge_name: &str,
//         parameters: &EdgeParameters,
//     ) -> VertexIterator<'vertex, Self::Vertex> {
//         TrustfallVertex::new(edge_name).into()
//     }
//
//     fn resolve_property<V: AsVertex<Self::Vertex> + 'vertex>(
//         &self,
//         contexts: ContextIterator<'vertex, V>,
//         type_name: &str,
//         property_name: &str,
//     ) -> ContextOutcomeIterator<'vertex, V, FieldValue> {
//         resolve_property_with(contexts, field_property!(everyone_else))
//     }
//
//     fn resolve_neighbors<V: AsVertex<Self::Vertex> + 'vertex>(
//         &self,
//         contexts: ContextIterator<'vertex, V>,
//         type_name: &str,
//         edge_name: &str,
//         parameters: &EdgeParameters,
//     ) -> ContextOutcomeIterator<'vertex, V, VertexIterator<'vertex, Self::Vertex>> {
//         let edge_resolver = |vertex: &Self::Vertex| -> VertexIterator<'vertex, Self::Vertex> {
//             vertex.everyone_else.clone().into()
//         };
//
//         resolve_neighbors_with(contexts, edge_resolver)
//     }
//
//     fn resolve_coercion<V: AsVertex<Self::Vertex> + 'vertex>(
//         &self,
//         contexts: ContextIterator<'vertex, V>,
//         type_name: &str,
//         coerce_to_type: &str,
//     ) -> ContextOutcomeIterator<'vertex, V, bool> {
//         todo!()
//     }
// }
