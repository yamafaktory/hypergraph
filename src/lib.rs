#![forbid(rust_2018_idioms)]
#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]
#![deny(unsafe_code, nonstandard_style)]

//! TODO
use indexmap::{IndexMap, IndexSet};
use std::{fmt, hash::Hash};

/// Hyperedge representation as a growable array of vertices indexes.
pub type HyperedgeVec = Vec<usize>;

/// An Hypergraph composed of generic vertices and hyperedges.
pub struct Hypergraph<V, HE> {
    /// Vertices are stored as an IndexMap whose keys are the weights
    /// and values are an IndexSet containing the hyperedges which are
    /// including the current vertex.
    vertices: IndexMap<V, IndexSet<HyperedgeVec>>,
    /// Hyperedges are stored as an IndexMap whose keys are a vector of
    /// vertices indexes and values are the weights.
    hyperedges: IndexMap<HyperedgeVec, HE>,
}

impl<V: Eq + Hash + fmt::Debug, HE: fmt::Debug> fmt::Debug for Hypergraph<V, HE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.vertices.fmt(f)
    }
}

/// Shared Trait for vertices.
pub trait VertexTrait: Copy + fmt::Debug + Hash + Eq {}

impl<V> VertexTrait for V where V: Copy + fmt::Debug + Hash + Eq {}

impl<V, HE> Default for Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: Hash,
{
    fn default() -> Self {
        Hypergraph::new()
    }
}

/// Hypergraph implementations.
impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: Hash,
{
    /// Create a new hypergraph with no allocation.
    pub fn new() -> Self {
        Hypergraph::with_capacity(0, 0)
    }

    /// Create a new hypergraph with the specified capacity.
    pub fn with_capacity(vertices: usize, hyperedges: usize) -> Self {
        Hypergraph {
            vertices: IndexMap::with_capacity(vertices),
            hyperedges: IndexMap::with_capacity(hyperedges),
        }
    }

    /// Add a vertex as a custom weight in the hypergraph.
    /// Return the index of the vertex.
    pub fn add_vertex(&mut self, weight: V) -> usize {
        self.vertices
            .entry(weight)
            .or_insert(IndexSet::with_capacity(0));

        // Assume that unwrapping the index can't be none due to previous insertion.
        self.vertices.get_index_of(&weight).unwrap()
    }

    /// Get the weight of a vertex from its index.
    pub fn get_vertex_weight(&mut self, index: usize) -> Option<&V> {
        match self.vertices.get_index(index) {
            Some((weight, _)) => Some(weight),
            None => None,
        }
    }

    /// Return the number of vertices in the hypergraph.
    pub fn count_vertices(&mut self) -> usize {
        self.vertices.len()
    }

    /// Add a hyperedge as an array of vertices indexes and a custom weight in the hypergraph.
    /// Return the index of the hyperedge.
    pub fn add_hyperedge(&mut self, vertices: &[usize], weight: HE) -> usize {
        // Insert the new hyperedge.
        self.hyperedges.insert(vertices.to_owned(), weight);

        // Update the vertices so that we keep directly track of the hyperedge.
        for vertex in vertices.iter() {
            let mut set = self.vertices[*vertex].clone();

            set.insert(vertices.to_vec());

            self.vertices
                .insert(self.vertices.get_index(*vertex).unwrap().0.to_owned(), set);
        }

        // Assume that unwrapping the index can't be none due to previous insertion.
        self.hyperedges.get_index_of(vertices).unwrap()
    }

    /// Return the number of hyperedges in the hypergraph.
    pub fn count_hyperedges(&mut self) -> usize {
        self.hyperedges.len()
    }

    /// Get the weight of a hyperedge from its index.
    pub fn get_hyperedge_weight(&mut self, index: usize) -> Option<&HE> {
        match self.hyperedges.get_index(index) {
            Some((_, weight)) => Some(weight),
            None => None,
        }
    }
}
