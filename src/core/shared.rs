use crate::{errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait, VertexIndex};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Private helper function used internally.
    #[allow(clippy::type_complexity)]
    pub(crate) fn get_connections(
        &self,
        from: VertexIndex,
        to: Option<VertexIndex>,
    ) -> Result<Vec<(HyperedgeIndex, Option<VertexIndex>)>, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(from)?;

        let (_, hyperedges_index_set) = self
            .vertices
            .get_index(internal_index)
            .ok_or(HypergraphError::InternalVertexIndexNotFound(internal_index))?;

        let hyperedges =
            self.get_hyperedges(hyperedges_index_set.clone().into_iter().collect_vec())?;

        let hyperedges_with_vertices = hyperedges
            .into_iter()
            .map(
                |hyperedge_index| match self.get_hyperedge_vertices(hyperedge_index) {
                    Ok(vertices) => Ok((hyperedge_index, vertices)),
                    Err(error) => Err(error),
                },
            )
            .collect::<Result<Vec<(HyperedgeIndex, Vec<VertexIndex>)>, HypergraphError<V, HE>>>()?;

        let results = hyperedges_with_vertices.into_iter().fold(
            Vec::new(),
            |acc: Vec<(HyperedgeIndex, Option<VertexIndex>)>, (hyperedge_index, vertices)| {
                vertices.iter().tuple_windows::<(_, _)>().fold(
                    acc,
                    |index_acc, (window_from, window_to)| {
                        match to {
                            Some(to) => {
                                // Inject only the index of the hyperedge
                                // if the current window is a match.
                                if *window_from == from && *window_to == to {
                                    return index_acc
                                        .into_iter()
                                        .chain(vec![(hyperedge_index, None)])
                                        .collect_vec();
                                }
                            }
                            None => {
                                // Inject the index of the hyperedge and the
                                // if the current window is a match.
                                if *window_from == from {
                                    return index_acc
                                        .into_iter()
                                        .chain(vec![(hyperedge_index, Some(*window_to))])
                                        .collect_vec();
                                }
                            }
                        }

                        index_acc
                    },
                )
            },
        );

        Ok(results)
    }
}
