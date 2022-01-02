use crate::{
    errors::HypergraphError, HyperedgeIndex, HyperedgeKey, HyperedgeTrait, Hypergraph, VertexIndex,
    VertexTrait,
};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Gets the intersections of a set of hyperedges as a vector of vertices.
    pub fn get_hyperedges_intersections(
        &self,
        hyperedges: Vec<HyperedgeIndex>,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        // Keep track of the number of hyperedges.
        let number_of_hyperedges = hyperedges.len();

        // Early exit if less than two hyperedges are provided.
        if number_of_hyperedges < 2 {
            return Err(HypergraphError::HyperedgesIntersections);
        }

        // Get the internal vertices of the hyperedges and keep the eventual error.
        let vertices = hyperedges
            .into_iter()
            .map(
                |hyperedge_index| match self.get_internal_hyperedge(hyperedge_index) {
                    Ok(internal_index) => match self.hyperedges.get_index(internal_index).ok_or(
                        HypergraphError::InternalHyperedgeIndexNotFound(internal_index),
                    ) {
                        Ok(HyperedgeKey { vertices, .. }) => {
                            // Keep the unique vertices.
                            Ok(vertices.iter().unique().cloned().collect_vec())
                        }
                        Err(error) => Err(error),
                    },
                    Err(error) => Err(error),
                },
            )
            .collect::<Result<Vec<Vec<usize>>, HypergraphError<V, HE>>>();

        match vertices {
            Ok(vertices) => {
                self.get_vertices(
                    vertices
                        .into_iter()
                        // Flatten and sort the vertices.
                        .flatten()
                        .sorted()
                        // Map the result to tuples where the second term is an arbitrary value.
                        // The goal is to group them by indexes.
                        .map(|index| (index, 0))
                        .into_group_map()
                        .into_iter()
                        // Filter the groups having the same size as the hyperedge.
                        .filter_map(|(index, occurences)| {
                            if occurences.len() == number_of_hyperedges {
                                Some(index)
                            } else {
                                None
                            }
                        })
                        .sorted()
                        .collect_vec(),
                )
            }
            Err(error) => Err(error),
        }
    }
}
