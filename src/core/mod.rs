pub(crate) mod bi_hash_map;
mod debug;
mod dot;
#[doc(hidden)]
pub mod error;
#[doc(hidden)]
pub mod hyperedges;
mod shared;
mod utils;
/// Vertices implementation.
#[doc(hidden)]
pub mod vertices;

use bi_hash_map::BiHashMap;
use debug::ExtendedDebug;
// use dot::render_to_graphviz_dot;

use indexmap::{IndexMap, IndexSet};
use std::{
    fmt::{Debug, Display, Formatter, Result},
    hash::Hash,
};

/// Shared Trait for hyperedges and vertices.
/// Must be implemented to use the library.
pub trait SharedTrait: Copy + Debug + Display + Eq + Hash {}

impl<T> SharedTrait for T where T: Copy + Debug + Display + Eq + Hash {}

/// Vertex stable index representation as usize.
/// Uses the newtype index pattern.
/// <https://matklad.github.io/2018/06/04/newtype-index-pattern.html>
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VertexIndex(pub usize);

impl Display for VertexIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

/// Hyperedge stable index representation as usize.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HyperedgeIndex(pub usize);

impl Display for HyperedgeIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

/// A HyperedgeKey is a representation of both the vertices and the weight
/// of a hyperedge, used as a key in the hyperedges set.
/// In a non-simple hypergraph, since the same vertices can be shared by
/// different hyperedges, the weight is also included in the key to keep
/// it unique.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HyperedgeKey<HE> {
    vertices: Vec<usize>,
    weight: HE,
}

impl<HE> HyperedgeKey<HE> {
    /// Creates a new HyperedgeKey from the given vertices and weight.
    pub(crate) fn new(vertices: Vec<usize>, weight: HE) -> HyperedgeKey<HE> {
        Self { vertices, weight }
    }
}

/// A directed hypergraph composed of generic vertices and hyperedges.
pub struct Hypergraph<V, HE> {
    /// Vertices are stored as a map whose unique keys are the weights
    /// and the values are a set of the hyperedges indexes which include
    // the current vertex.
    pub vertices: IndexMap<V, IndexSet<usize>>,

    /// Hyperedges are stored as a set whose unique keys are a combination of
    /// vertices indexes and a weight. Two or more hyperedges can contain
    /// the exact same vertices (non-simple hypergraph).
    pub hyperedges: IndexSet<HyperedgeKey<HE>>,

    // Bi-directional maps for hyperedges and vertices.
    hyperedges_mapping: BiHashMap<HyperedgeIndex>,
    vertices_mapping: BiHashMap<VertexIndex>,

    // Stable index generation counters.
    hyperedges_count: usize,
    vertices_count: usize,
}

impl<V: Eq + Hash + Debug, HE: Debug> Debug for Hypergraph<V, HE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Hypergraph")
            .field("vertices", &self.vertices)
            .field("hyperedges", &self.hyperedges)
            .finish()
    }
}

impl<'a, V, HE> Default for Hypergraph<V, HE>
where
    V: SharedTrait + ExtendedDebug<'a>,
    HE: SharedTrait + ExtendedDebug<'a>,
{
    fn default() -> Self {
        Hypergraph::new()
    }
}

/// Hypergraph implementations.
impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Creates a new hypergraph with no allocation.
    pub fn new() -> Self {
        Hypergraph::with_capacity(0, 0)
    }

    /// Creates a new hypergraph with the specified capacity.
    pub fn with_capacity(vertices: usize, hyperedges: usize) -> Self {
        Hypergraph {
            vertices: IndexMap::with_capacity(vertices),
            hyperedges: IndexSet::with_capacity(hyperedges),
            hyperedges_mapping: BiHashMap::default(),
            vertices_mapping: BiHashMap::default(),
            hyperedges_count: 0,
            vertices_count: 0,
        }
    }

    /// Renders the hypergraph to Graphviz dot format.
    /// Due to Graphviz dot inability to render hypergraphs out of the box,
    /// unaries are rendered as vertex peripheries which can't be labelled.
    pub fn render_to_graphviz_dot(&self) {
        // println!("{}", render_to_graphviz_dot(self));
    }
}
