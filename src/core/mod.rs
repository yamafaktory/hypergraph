pub(crate) mod bi_hash_map;
#[doc(hidden)]
pub mod errors;
#[doc(hidden)]
pub mod hyperedges;
mod shared;
mod utils;
#[doc(hidden)]
pub mod vertices;

use bi_hash_map::BiHashMap;

use indexmap::{IndexMap, IndexSet};
use std::{
    fmt::{Debug, Display, Formatter, Result},
    hash::Hash,
};

/// Shared Trait for the vertices.
/// Must be implemented to use the library.
pub trait VertexTrait: Copy + Debug + Display + Eq + Hash {}

impl<T> VertexTrait for T where T: Copy + Debug + Display + Eq + Hash {}

/// Shared Trait for the hyperedges.
/// Must be implemented to use the library.
pub trait HyperedgeTrait: VertexTrait + Into<usize> {}

impl<T> HyperedgeTrait for T where T: VertexTrait + Into<usize> {}

/// Vertex stable index representation as usize.
/// Uses the newtype index pattern.
/// <https://matklad.github.io/2018/06/04/newtype-index-pattern.html>
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VertexIndex(pub usize);

impl Display for VertexIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}

impl From<usize> for VertexIndex {
    fn from(index: usize) -> Self {
        VertexIndex(index)
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

impl From<usize> for HyperedgeIndex {
    fn from(index: usize) -> Self {
        HyperedgeIndex(index)
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

impl<V, HE> Debug for Hypergraph<V, HE>
where
    V: Eq + Hash + Debug,
    HE: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Hypergraph")
            .field("vertices", &self.vertices)
            .field("hyperedges", &self.hyperedges)
            .finish()
    }
}

impl<V, HE> Default for Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    fn default() -> Self {
        Hypergraph::new()
    }
}

/// Hypergraph implementations.
impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Clears the hypergraph.
    pub fn clear(&mut self) {
        // Clear the hyperedges and vertices sets while keeping their capacities.
        self.hyperedges.clear();
        self.vertices.clear();

        // Reset the mappings.
        self.hyperedges_mapping = BiHashMap::default();
        self.vertices_mapping = BiHashMap::default();

        // Reset the counters.
        self.hyperedges_count = 0;
        self.vertices_count = 0;
    }

    /// Creates a new hypergraph with no allocation.
    pub fn new() -> Self {
        Hypergraph::with_capacity(0, 0)
    }

    /// Creates a new hypergraph with the specified capacity.
    pub fn with_capacity(vertices: usize, hyperedges: usize) -> Self {
        Hypergraph {
            hyperedges_count: 0,
            hyperedges_mapping: BiHashMap::default(),
            hyperedges: IndexSet::with_capacity(hyperedges),
            vertices_count: 0,
            vertices_mapping: BiHashMap::default(),
            vertices: IndexMap::with_capacity(vertices),
        }
    }
}
