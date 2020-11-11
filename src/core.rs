use crate::dot::render_to_graphviz_dot;
pub(super) use crate::private::ExtendedDebug;

use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use std::{
    fmt::{Debug, Formatter, Result},
    hash::Hash,
};

/// Hyperedge representation as a growable array of vertices indexes.
pub type HyperedgeVertices = Vec<usize>;

/// Hyperedge index - without weight(s) - representation as a usize.
pub type HyperedgeIndex = usize;

/// Hyperedge weighted index representation as a tuple of usize.
/// The first element is the index of the hyperedge.
/// The second element is the distinct index representing one of its weight.
/// E.g. (0, 0) and (0, 1) are the same hyperedges - connecting the same
/// vertices in the same order - with distinct weights (non-simple hypergraph).
pub type WeightedHyperedgeIndex = (HyperedgeIndex, usize);

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
    /// Having a IndexSet of weights allows having two or more hyperedges
    /// containing the same set of vertices (non-simple hypergraph).
    pub hyperedges: IndexMap<HyperedgeVertices, IndexSet<HE>>,
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
        }
    }

    /// Adds a vertex as a custom weight in the hypergraph.
    /// Returns the index of the vertex.
    pub fn add_vertex(&mut self, weight: V) -> VertexIndex {
        self.vertices
            .entry(weight)
            .or_insert(IndexSet::with_capacity(0));

        // Assume that unwrapping the index can't be none due to previous insertion.
        self.vertices.get_index_of(&weight).unwrap()
    }

    /// Gets the weight of a vertex from its index.
    pub fn get_vertex_weight(&self, index: VertexIndex) -> Option<&V> {
        self.vertices.get_index(index).map(|(weight, _)| weight)
    }

    /// Returns the number of vertices in the hypergraph.
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Adds a hyperedge as an array of vertices indexes and a custom weight in the hypergraph.
    /// Returns the weighted index of the hyperedge.
    pub fn add_hyperedge(&mut self, vertices: &[usize], weight: HE) -> WeightedHyperedgeIndex {
        // Update the vertices so that we keep directly track of the hyperedge.
        for vertex in vertices.iter() {
            let mut set = self.vertices[*vertex].clone();

            set.insert(vertices.to_vec());

            self.vertices
                .insert(self.vertices.get_index(*vertex).unwrap().0.to_owned(), set);
        }

        // Insert the new hyperedge with the corresponding weight, get back the indexes.
        match self.hyperedges.get(vertices) {
            Some(weights) => {
                let mut new_weights = weights.clone();
                let (weight_index, _) = new_weights.insert_full(weight);
                let (hyperedge_index, _) = self
                    .hyperedges
                    .insert_full(vertices.to_owned(), new_weights);

                (hyperedge_index, weight_index)
            }
            None => {
                let mut weights = IndexSet::new();
                let (weight_index, _) = weights.insert_full(weight);
                let (hyperedge_index, _) =
                    self.hyperedges.insert_full(vertices.to_owned(), weights);

                (hyperedge_index, weight_index)
            }
        }
    }

    /// Returns the number of hyperedges in the hypergraph.
    pub fn count_hyperedges(&self) -> usize {
        self.hyperedges
            .iter()
            .fold(0, |count, (_, weights)| count + weights.len())
    }

    /// Gets the weight of a hyperedge from its weighted index.
    pub fn get_hyperedge_weight(
        &self,
        (hyperedge_index, weight_index): WeightedHyperedgeIndex,
    ) -> Option<&HE> {
        match self.hyperedges.get_index(hyperedge_index) {
            Some((_, weights)) => weights.get_index(weight_index),
            None => None,
        }
    }

    /// Gets the hyperedge's vertices.
    pub fn get_hyperedge_vertices(&self, index: HyperedgeIndex) -> Option<&HyperedgeVertices> {
        self.hyperedges
            .get_index(index)
            .map(|(vertices, _)| vertices)
    }

    /// Gets the intersections of a set of hyperedges as a vector of vertices.
    pub fn get_hyperedges_intersections(&self, hyperedges: &[HyperedgeIndex]) -> HyperedgeVertices {
        hyperedges
            .iter()
            .filter_map(|index| {
                self.hyperedges
                    .get_index(*index)
                    .map(|(vertices, _)| vertices.iter().unique().collect_vec())
            })
            .flatten()
            .sorted()
            // Map the result to tuples where the second term is an arbitrary value.
            // The goal is to group them by indexes.
            .map(|index| (*index, 0))
            .into_group_map()
            .iter()
            // Filter the groups having the same size as the hyperedge.
            .filter_map(|(index, occurences)| {
                if occurences.len() == hyperedges.len() {
                    Some(*index)
                } else {
                    None
                }
            })
            .sorted()
            .collect::<Vec<usize>>()
    }

    /// Private helper function used internally.
    fn get_connections(&self, from: VertexIndex, to: Option<VertexIndex>) -> Vec<HyperedgeIndex> {
        self.vertices
            .get_index(from)
            .iter()
            .fold(Vec::new(), |acc, (_, hyperedges)| {
                hyperedges
                    .iter()
                    .enumerate()
                    .fold(acc, |hyperedge_acc, (index, hyperedge)| {
                        hyperedge.iter().tuple_windows::<(_, _)>().fold(
                            hyperedge_acc,
                            |index_acc, (window_from, window_to)| {
                                match to {
                                    Some(to) => {
                                        // Inject the index of the hyperedge if the current window is a match.
                                        if *window_from == from && *window_to == to {
                                            return index_acc
                                                .into_iter()
                                                .chain(vec![index])
                                                .collect::<Vec<usize>>();
                                        }
                                    }
                                    None => {
                                        // Inject the next vertex if the current window is a match.
                                        if *window_from == from {
                                            return index_acc
                                                .into_iter()
                                                .chain(vec![*window_to])
                                                .collect::<Vec<usize>>();
                                        }
                                    }
                                }

                                index_acc
                            },
                        )
                    })
            })
    }

    /// Gets the list of all hyperedges containing a matching connection from
    /// one vertex to another.
    pub fn get_hyperedges_connections(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Vec<HyperedgeIndex> {
        self.get_connections(from, Some(to))
    }

    /// Gets the list of all vertices connected to a given vertex.
    pub fn get_vertex_connections(&self, from: VertexIndex) -> Vec<VertexIndex> {
        self.get_connections(from, None)
    }

    /// Renders the hypergraph to Graphviz dot format.
    /// Due to Graphviz dot inability to render hypergraphs out of the box,
    /// unaries are rendered as vertex peripheries which can't be labelled.
    pub fn render_to_graphviz_dot(&self) {
        println!("{}", render_to_graphviz_dot(&self));
    }
}
