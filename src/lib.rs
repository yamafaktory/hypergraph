#![deny(unsafe_code, nonstandard_style)]
#![forbid(rust_2018_idioms)]
#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]

//! Hypergraph is an open-source library built in Rust to represent directed hypergraphs.
//! ## Example
//! ```ignore
//!use hypergraph::Hypergraph;
//!
//!// Create a new hypergraph.
//!let mut graph = Hypergraph::<&str, &str>::new();
//!
//!// Some data.
//!let foo = "foo";
//!let bar = "bar";
//!
//!// Add two vertices.
//!assert_eq!(graph.add_vertex(foo), 0);
//!assert_eq!(graph.add_vertex(bar), 1);
//!
//!// Add three hyperedges.
//!let weight_with_unary = "hyperedge with a unary {foo}";
//!assert_eq!(graph.add_hyperedge(&[0], weight_with_unary), Some([0, 0]));
//!let weight_with_self_loop = "hyperedge with a self-loop {foo, bar, bar}";
//!assert_eq!(graph.add_hyperedge(&[0, 1, 1], weight_with_self_loop), Some([1, 0]));
//!let different_weight_same_set = "hyperedge with identical set of vertices but different weight";
//!assert_eq!(graph.add_hyperedge(&[0, 1, 1], different_weight_same_set), Some([1, 1]));
//!
//!// Count the vertices and the hyperedges.
//!assert_eq!(graph.count_vertices(), 2);
//!assert_eq!(graph.count_hyperedges(), 3);
//!
//!// Get the weights of some hyperedges and vertices.
//!assert_eq!(graph.get_vertex_weight(0), Some(&foo));
//!assert_eq!(graph.get_vertex_weight(1), Some(&bar));
//!assert_eq!(graph.get_hyperedge_weight([0, 0]), Some(&weight_with_unary));
//!assert_eq!(graph.get_hyperedge_weight([1, 0]), Some(&weight_with_self_loop));
//!assert_eq!(graph.get_hyperedge_weight([1, 1]), Some(&different_weight_same_set));
//!
//!// Get the vertices of a hyperedge.
//!assert_eq!(graph.get_hyperedge_vertices(1), Some(vec![0, 1, 1]));
//!
//!// Check hyperedges intersections.
//!assert_eq!(
//!    graph.get_hyperedges_intersections(&[0, 1]),
//!    vec![0 as usize]
//!);
//!
//!// Render the graph to Graphviz dot format.
//!graph.render_to_graphviz_dot();
//!//digraph {
//!//    edge [penwidth=0.5, arrowhead=normal, arrowsize=0.5, fontsize=8.0];
//!//    node [color=gray20, fontsize=8.0, fontcolor=white, style=filled, shape=circle];
//!//    rankdir=LR;
//!//
//!//    0 [label="\"foo\"\l", peripheries=2];
//!//    1 [label="\"bar\"\l"];
//!//
//!//    0 -> 1 -> 1 [color="#0c961e", fontcolor="#0c961e", label="\"hyperedge with a self-loop {foo, bar, bar}\"\l"];
//!//    0 -> 1 -> 1 [color="#024668", fontcolor="#024668", label="\"hyperedge with identical set of vertices but different weight\"\l"];
//!//}
//! ```

/// Public API.
pub mod core;

// Reexport of the public API.
#[doc(inline)]
pub use crate::core::*;
