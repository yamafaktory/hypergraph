use crate::{
    core::utils::are_arrays_equal, HyperedgeIndex, HyperedgeVertices, Hypergraph, SharedTrait,
    VertexIndex, WeightedHyperedgeIndex,
};

use indexmap::IndexSet;
use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Adds a hyperedge as an array of vertices indexes and a custom weight in the hypergraph.
    /// Returns the weighted index of the hyperedge.
    pub fn add_hyperedge(&mut self, vertices: &[usize], weight: HE) -> WeightedHyperedgeIndex {
        // Update the vertices so that we keep directly track of the hyperedge.
        for vertex in vertices.iter() {
            let mut index_set = self.vertices[*vertex].clone();

            index_set.insert(vertices.to_vec());

            self.vertices.insert(
                self.vertices.get_index(*vertex).unwrap().0.to_owned(),
                index_set,
            );
        }

        // Insert the new hyperedge with the corresponding weight, get back the indexes.
        match self.hyperedges.get(vertices) {
            Some(weights) => {
                let mut new_weights = weights.clone();
                let (weight_index, _) = new_weights.insert_full(weight);
                let (hyperedge_index, _) = self
                    .hyperedges
                    .insert_full(vertices.to_owned(), new_weights);

                [hyperedge_index, weight_index]
            }
            None => {
                let mut weights = IndexSet::new();
                let (weight_index, _) = weights.insert_full(weight);
                let (hyperedge_index, _) =
                    self.hyperedges.insert_full(vertices.to_owned(), weights);

                [hyperedge_index, weight_index]
            }
        }
    }

    /// Returns the number of hyperedges in the hypergraph.
    pub fn count_hyperedges(&self) -> usize {
        self.hyperedges
            .iter()
            .fold(0, |count, (_, weights)| count + weights.len())
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

    /// Gets the hyperedge's vertices.
    pub fn get_hyperedge_vertices(&self, index: HyperedgeIndex) -> Option<HyperedgeVertices> {
        self.hyperedges
            .get_index(index)
            .map(|(vertices, _)| vertices.to_owned())
    }

    /// Gets the weight of a hyperedge from its weighted index.
    pub fn get_hyperedge_weight(
        &self,
        [hyperedge_index, weight_index]: WeightedHyperedgeIndex,
    ) -> Option<&HE> {
        match self.hyperedges.get_index(hyperedge_index) {
            Some((_, weights)) => weights.get_index(weight_index),
            None => None,
        }
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

    /// Updates the weight of a hyperedge based on its weighted index.
    pub fn update_hyperedge_weight(
        &mut self,
        [hyperedge_index, weight_index]: WeightedHyperedgeIndex,
        weight: HE,
    ) -> bool {
        match self.hyperedges.get_index_mut(hyperedge_index) {
            Some((_, weights)) => {
                // We can't directly replace the value in the set.
                // First, we need to insert the new weight, it will end up
                // being at the last position.
                if !weights.insert(weight) {
                    return false;
                };

                // Then get the value by index of the original weight.
                match weights.clone().get_index(weight_index) {
                    Some(weight) => {
                        // Last, use swap and remove. It will remove the old weight
                        // and insert the new one at the index position of the former.
                        weights.swap_remove(weight)
                    }
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Updates the vertices of a hyperedge.
    pub fn update_hyperedge_vertices(
        &mut self,
        hyperedge_index: usize,
        vertices: &[usize],
    ) -> bool {
        match self.hyperedges.clone().get_index(hyperedge_index) {
            Some((key, value)) => {
                // Keep track of the initial indexes.
                let previous_vertices = self.get_hyperedge_vertices(hyperedge_index).unwrap();
                dbg!(previous_vertices.clone());
                // Find the indexes which have been added.
                let added = vertices.iter().fold(vec![], |mut acc: Vec<usize>, index| {
                    if !previous_vertices.iter().any(|x| x == index) {
                        acc.push(*index)
                    }

                    acc
                });

                // Find the indexes which have been removed.
                let removed = previous_vertices
                    .iter()
                    .filter_map(|x| {
                        if !vertices.iter().any(|y| x == y) {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<usize>>();

                // Finally get the unchanged ones.
                let unchanged = previous_vertices
                    .iter()
                    .filter_map(|x| {
                        if !removed.iter().any(|y| x == y) {
                            Some(*x)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<usize>>();

                for index in unchanged.iter() {
                    self.vertices.get_index(*index).map(|(_, vertex)| {
                        dbg!(index, vertex);
                        let r =
                            vertex
                                .iter()
                                .fold(IndexSet::new(), |mut new_index_set, hyperedge| {
                                    new_index_set.insert(
                                        if are_arrays_equal(hyperedge, &previous_vertices) {
                                            vertices
                                        } else {
                                            hyperedge
                                        },
                                    );

                                    new_index_set
                                });

                        dbg!(r);
                    });
                }

                // dbg!(self.vertices.clone());

                // We need to use insert and swap_remove trick here too,
                // see e.g. the update_vertex_weight method.
                self.hyperedges.insert(vertices.to_vec(), value.to_owned());
                self.hyperedges.swap_remove(key).is_some()
            }
            None => false,
        }
    }
}
