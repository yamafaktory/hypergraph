#![forbid(rust_2018_idioms)]
#![warn(missing_debug_implementations, missing_docs, unreachable_pub)]
#![deny(unsafe_code, nonstandard_style)]

//! TODO:
//! - Deal with out of bound indexes
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use random_color::{Luminosity, RandomColor};
use std::fmt::Write;
use std::{fmt, hash::Hash};

/// Hyperedge representation as a growable array of vertices indexes.
pub type HyperedgeVertices = Vec<usize>;

/// Hyperedge index - without weight(s) - representation as a usize.
pub type HyperedgeIndex = usize;

/// Hyperedge index representation as a tuple of usize.
pub type WeightedHyperedgeIndex = (HyperedgeIndex, usize);

/// Vertex index representation as a usize.
pub type VertexIndex = usize;

/// A directed Hypergraph composed of generic vertices and hyperedges.
pub struct Hypergraph<V, HE> {
    /// Vertices are stored as an IndexMap whose keys are the weights
    /// and values are an IndexSet containing the hyperedges which are
    /// including the current vertex.
    vertices: IndexMap<V, IndexSet<HyperedgeVertices>>,
    /// Hyperedges are stored as an IndexMap whose keys are a vector of
    /// vertices indexes and values are an IndexSet of weights.
    /// Having a IndexSet of weights allows having two or more hyperedges
    /// containing the same set of vertices (non-simple hypergraph).
    hyperedges: IndexMap<HyperedgeVertices, IndexSet<HE>>,
}

impl<V: Eq + Hash + fmt::Debug, HE: fmt::Debug> fmt::Debug for Hypergraph<V, HE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.vertices.fmt(f)
    }
}

/// Shared Trait for hyperedges and vertices.
pub trait SharedTrait: Copy + fmt::Debug + Hash + Eq {}

impl<T> SharedTrait for T where T: Copy + fmt::Debug + Hash + Eq {}

impl<'a, V, HE> Default for Hypergraph<V, HE>
where
    V: SharedTrait + ExtendedDebug<'a>,
    HE: SharedTrait,
{
    fn default() -> Self {
        Hypergraph::new()
    }
}
/// TODO
pub trait ExtendedDebug<'a> {
    /// TODO
    type Debug: 'a;

    /// TODO
    fn my_debug(self) -> Self::Debug;
}

impl<'a, T> ExtendedDebug<'a> for &'a T {
    type Debug = CustomDebug<'a, T>;

    fn my_debug(self) -> Self::Debug {
        CustomDebug(self)
    }
}

/// TODO
pub struct CustomDebug<'a, T>(&'a T);

impl<'a, T> fmt::Debug for CustomDebug<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for char in format!("{:?}", &self.0).chars() {
            match char {
                '"' | '\\' => f.write_char('\\')?,
                // \l is for left justified linebreak
                '\n' => return f.write_str("\\l"),
                _ => {}
            };

            f.write_char(char)?
        }

        Ok(())
    }
}

/// Hypergraph implementations.
impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
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
    pub fn add_vertex(&mut self, weight: V) -> VertexIndex {
        self.vertices
            .entry(weight)
            .or_insert(IndexSet::with_capacity(0));

        // Assume that unwrapping the index can't be none due to previous insertion.
        self.vertices.get_index_of(&weight).unwrap()
    }

    /// Get the weight of a vertex from its index.
    pub fn get_vertex_weight(&self, index: VertexIndex) -> Option<&V> {
        match self.vertices.get_index(index) {
            Some((weight, _)) => Some(weight),
            None => None,
        }
    }

    /// Return the number of vertices in the hypergraph.
    pub fn count_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Add a hyperedge as an array of vertices indexes and a custom weight in the hypergraph.
    /// Return the index of the hyperedge.
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

    /// Return the number of hyperedges in the hypergraph.
    pub fn count_hyperedges(&self) -> usize {
        self.hyperedges
            .iter()
            .fold(0, |count, (_, weights)| count + weights.len())
    }

    /// Get the weight of a hyperedge from its index.
    pub fn get_hyperedge_weight(
        &self,
        (hyperedge_index, weight_index): WeightedHyperedgeIndex,
    ) -> Option<&HE> {
        match self.hyperedges.get_index(hyperedge_index) {
            Some((_, weights)) => weights.get_index(weight_index),
            None => None,
        }
    }

    /// Get hyperedge's vertices.
    pub fn get_hyperedge_vertices(&self, index: HyperedgeIndex) -> Option<&HyperedgeVertices> {
        match self.hyperedges.get_index(index) {
            Some((vertices, _)) => Some(vertices),
            None => None,
        }
    }

    /// Get the intersections of a set of hyperedges as a vector of vertices.
    pub fn get_hyperedges_intersections(&self, hyperedges: &[HyperedgeIndex]) -> HyperedgeVertices {
        hyperedges
            .iter()
            .filter_map(|index| match self.hyperedges.get_index(*index) {
                // Only get the unique vertices as a hyperedge might contain a self-loop.
                Some((vertices, _)) => Some(vertices.iter().unique().collect_vec()),
                None => None,
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

    /// Render the hypergraph to Graphviz dot format.
    pub fn render(&self) {
        println!("{:?}", self.hyperedges);
        println!("{:?}", self.vertices);

        // Partition the hyperedges in two groups, one for the unaries, the other for the rest.
        // Graphviz dot doesn't provide a way to render a hypergraph. To overcome this issue,
        // we need to track the unaries and treat them separately.
        let partitioned_hyperedges = self.hyperedges.iter().fold(
            (Vec::new(), Vec::new()),
            |mut acc: (
                Vec<(&HyperedgeVertices, &IndexSet<HE>)>,
                Vec<(&HyperedgeVertices, &IndexSet<HE>)>,
            ),
             (vertices, weight)| {
                if vertices.len() == 1 {
                    acc.1.push((vertices, weight)); // Unaries.
                } else {
                    acc.0.push((vertices, weight));
                }

                acc
            },
        );
        dbg!(partitioned_hyperedges);

        let rendered_vertices =
            self.vertices
                .iter()
                .enumerate()
                .fold(String::new(), |acc, (index, (weight, _))| {
                    [
                        format!(r#"  {} [label="{:?}"];"#, index, weight.my_debug()),
                        acc,
                    ]
                    .join("\n")
                });

        // dbg!(t.0);
        // .fold(String::new(), |acc, (vertices, weight)| {
        //     format!(
        //         r#"
        //         {}
        //         {} [ color="{}", label={:?} ];
        //     "#,
        //         acc,
        //         vertices.iter().join(" -> ").as_str(),
        //         RandomColor::new().luminosity(Luminosity::Dark).to_hex(),
        //         weight
        //     )
        // });

        // let test: String =
        //     self.hyperedges
        //         .iter()
        //         .fold(String::new(), |acc, (vertices, weights)| {
        //             let random_color = RandomColor::new().luminosity(Luminosity::Dark).to_hex();

        //             let t = if vertices.len() == 1 {
        //                 dbg!(vertices, weights);
        //                 format!("todo")
        //             } else {
        //                 weights.iter().fold(String::new(), |weights_acc, weight| {
        //                     format!(
        //                         r#"
        //                             {}
        //                             {} [ color="{}", fontcolor="{}", label={:?} ];
        //                         "#,
        //                         weights_acc,
        //                         vertices.iter().join(" -> ").as_str(),
        //                         random_color,
        //                         random_color,
        //                         weight
        //                     )
        //                 })
        //             };

        //             format!(
        //                 r#"
        //                     {}
        //                     {}
        //                 "#,
        //                 acc, t
        //             )
        //         });

        let dot = format!(
            r##"
    digraph {{
        edge [ penwidth =0.5, arrowhead=normal, arrowsize=0.5, fontsize=8.0 ];
        node [ color=gray20, fontsize=8.0, fontcolor=white, style=filled, shape=circle ];
        rankdir = LR;
    
        {}
    }}"##,
            rendered_vertices
        );
        println!("{}", dot);
    }
}
