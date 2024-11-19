use itertools::Itertools;

use crate::{
    HyperedgeIndex,
    HyperedgeKey,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

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
            return Err(HypergraphError::HyperedgesInvalidIntersections);
        }

        // Get the internal vertices of the hyperedges and keep the eventual error.
        let vertices = hyperedges
            .into_iter()
            .map(|hyperedge_index| {
                self.get_internal_hyperedge(hyperedge_index)
                    .and_then(|internal_index| {
                        self.hyperedges
                            .get_index(internal_index)
                            .ok_or(HypergraphError::InternalHyperedgeIndexNotFound(
                                internal_index,
                            ))
                            .map(|HyperedgeKey { vertices, .. }| {
                                vertices.iter().unique().copied().collect_vec()
                            })
                    })
            })
            .collect::<Result<Vec<Vec<usize>>, HypergraphError<V, HE>>>();

        vertices.and_then(|vertices| {
            self.get_vertices(
                &vertices
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
        })
    }
}
