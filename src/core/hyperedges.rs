use crate::{
    HyperedgeIndex, HyperedgeVertices, Hypergraph, SharedTrait, VertexIndex, WeightedHyperedgeIndex,
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

    /// Update the weight of a hyperedge based on its weighted index.
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
                    Some(t) => {
                        // Last, use swap and remove. It will remove the old weight
                        // and insert the new one at the index position of the former.
                        weights.swap_remove(t)
                    }
                    None => false,
                }
            }
            None => false,
        }
    }

    /// Gets the hyperedge's vertices.
    pub fn get_hyperedge_vertices(&self, index: HyperedgeIndex) -> Option<&HyperedgeVertices> {
        self.hyperedges
            .get_index(index)
            .map(|(vertices, _)| vertices)
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
}
