mod debug;
mod dot;
/// Hyperedges implementation.
#[doc(hidden)]
pub mod hyperedges;
mod shared;
mod utils;
/// Vertices implementation.
#[doc(hidden)]
pub mod vertices;

use debug::ExtendedDebug;
use dot::render_to_graphviz_dot;

use indexmap::{IndexMap, IndexSet};
use std::{
    collections::HashMap,
    fmt::{Debug, Formatter, Result},
    hash::Hash,
    ops::Index,
};

/// Hyperedge representation as a growable array of vertices indexes.
pub type HyperedgeVertices = Vec<usize>;

/// Hyperedge index - without weight(s) - representation as a usize.
pub type HyperedgeIndex = usize;

/// Hyperedge weighted index representation as an array of two usize.
/// The first element is the index of the hyperedge.
/// The second element is the distinct index representing one of its weight.
/// E.g. [0, 0] and [0, 1] are two hyperedges - connecting the same
/// vertices in the same order - with distinct weights (non-simple hypergraph).
pub type WeightedHyperedgeIndex = [usize; 2];

/// Vertex index representation as a usize.
pub type VertexIndex = usize;

/// A directed hypergraph composed of generic vertices and hyperedges.
pub struct Hypergraph<V, HE> {
    /// Vertices are stored as an IndexMap whose keys are the weights
    /// and values are an IndexSet containing the hyperedges which are
    /// including the current vertex.
    pub vertices: IndexMap<V, IndexSet<HyperedgeVertices>>,
    /// Hyperedges are stored as an IndexMap whose keys are a vector of
    /// vertices indexes and values are an IndexSet of weights.
    /// Having an IndexSet of weights allows having two or more hyperedges
    /// containing the same set of vertices (non-simple hypergraph).
    pub hyperedges: IndexMap<HyperedgeVertices, IndexSet<HE>>,

    // Mimic a bi-directional map for hyperedges and vertices.
    // Keep a counter for both for stable index generation.
    hyperedges_count: usize,
    hyperedges_mapping_left: HashMap<WeightedHyperedgeIndex, StableHyperedgeWeightedIndex>,
    hyperedges_mapping_right: HashMap<StableHyperedgeWeightedIndex, WeightedHyperedgeIndex>,
    vertices_count: usize,
    vertices_mapping_left: HashMap<VertexIndex, StableVertexIndex>,
    vertices_mapping_right: HashMap<StableVertexIndex, VertexIndex>,
}

impl<V: Eq + Hash + Debug, HE: Debug> Debug for Hypergraph<V, HE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.vertices.fmt(f)
    }
}

/// Shared Trait for hyperedges and vertices.
/// This is a set of traits that must be implemented to use the library.
pub trait SharedTrait: Copy + Debug + Eq + Hash {}

impl<T> SharedTrait for T where T: Copy + Debug + Eq + Hash {}

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
            hyperedges: IndexMap::with_capacity(hyperedges),

            //
            hyperedges_count: 0,
            hyperedges_mapping_left: HashMap::with_capacity(0),
            hyperedges_mapping_right: HashMap::with_capacity(0),
            vertices_count: 0,
            vertices_mapping_left: HashMap::with_capacity(0),
            vertices_mapping_right: HashMap::with_capacity(0),
        }
    }

    /// Renders the hypergraph to Graphviz dot format.
    /// Due to Graphviz dot inability to render hypergraphs out of the box,
    /// unaries are rendered as vertex peripheries which can't be labelled.
    pub fn render_to_graphviz_dot(&self) {
        println!("{}", render_to_graphviz_dot(self));
    }
}

// Use the newtype index pattern.
// https://matklad.github.io/2018/06/04/newtype-index-pattern.html
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct StableVertexIndex(pub usize);

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct StableHyperedgeWeightedIndex(pub usize, pub usize);

impl<V, HE> Index<V> for Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    type Output = usize;

    fn index(&self, vertex: V) -> &Self::Output {
        let (index, _, _) = self.vertices.get_full(&vertex).unwrap();
        dbg!(index);
        &0
    }
}
