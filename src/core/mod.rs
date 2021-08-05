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

use debug::ExtendedDebug;
// use dot::render_to_graphviz_dot;

use indexmap::{IndexMap, IndexSet};
use std::{
    collections::HashMap,
    fmt::{Debug, Display, Formatter, Result},
    hash::Hash,
    ops::Index,
};

/// Vertex stable index representation as usize.
/// Uses the newtype index pattern.
/// https://matklad.github.io/2018/06/04/newtype-index-pattern.html
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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HyperedgeKey<HE> {
    vertices: Vec<usize>,
    weight: HE,
}

impl<HE> HyperedgeKey<HE> {
    pub fn new(vertices: Vec<usize>, weight: HE) -> HyperedgeKey<HE> {
        Self { vertices, weight }
    }
}

pub struct BiHashMap<Index>
where
    Index: SharedTrait,
{
    left: HashMap<usize, Index>,
    right: HashMap<Index, usize>,
}

impl<Index> BiHashMap<Index>
where
    Index: SharedTrait,
{
    pub fn new() -> BiHashMap<Index> {
        Self {
            left: HashMap::<usize, Index>::with_capacity(0),
            right: HashMap::<Index, usize>::with_capacity(0),
        }
    }
}

impl<Index> Default for BiHashMap<Index>
where
    Index: SharedTrait,
{
    fn default() -> Self {
        BiHashMap::new()
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

    // Mimic bi-directional maps for hyperedges and vertices.
    hyperedges_mapping: BiHashMap<HyperedgeIndex>,
    vertices_mapping: BiHashMap<VertexIndex>,

    // Keep stable index generation counters.
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

/// Shared Trait for hyperedges and vertices.
/// This is a set of traits that must be implemented to use the library.
pub trait SharedTrait: Copy + Debug + Display + Eq + Hash {}

impl<T> SharedTrait for T where T: Copy + Debug + Display + Eq + Hash {}

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

// impl<V, HE> Index<V> for Hypergraph<V, HE>
// where
//     V: SharedTrait,
//     HE: SharedTrait,
// {
//     type Output = usize;

//     fn index(&self, vertex: V) -> &Self::Output {
//         let (index, _, _) = self.vertices.get_full(&vertex).unwrap();
//         dbg!(index);
//         &0
//     }
// }
