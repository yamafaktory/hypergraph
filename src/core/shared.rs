use crate::{Hypergraph, SharedTrait, StableHyperedgeWeightedIndex, StableVertexIndex};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Private helper function used internally.
    /// Given a starting vertex index and a optional target vertex index,
    /// returns a vector of tuples of the form (stable hyperedge weighted
    /// index, stable vertex index).
    pub(crate) fn get_connections(
        &self,
        from: StableVertexIndex,
        to: Option<StableVertexIndex>,
    ) -> Vec<(StableHyperedgeWeightedIndex, StableVertexIndex)> {
        match self.vertices_mapping_right.get(&from) {
            Some(from) => {
                self.vertices.get_index(*from).iter().fold(
                    Vec::new(),
                    |acc: Vec<(StableHyperedgeWeightedIndex, StableVertexIndex)>,
                     (_, hyperedges)| {
                        hyperedges
                            .iter()
                            .map(|unstable_index| {
                                // Keep track of the stable index of the
                                // hyperedge via a tuple.
                                (
                                    self.hyperedges_mapping_left.get(unstable_index).unwrap(),
                                    self.get_hyperedge_vertices(
                                        *self.hyperedges_mapping_left.get(unstable_index).unwrap(),
                                    )
                                    .unwrap()
                                    .iter()
                                    .map(|stable_vertex_index| {
                                        *self
                                            .vertices_mapping_right
                                            .get(stable_vertex_index)
                                            .unwrap()
                                    })
                                    .collect_vec(),
                                )
                            })
                            .enumerate()
                            .fold(
                                acc,
                                |hyperedge_acc, (index, (weighted_index, hyperedge))| {
                                    hyperedge.iter().tuple_windows::<(_, _)>().fold(
                                        hyperedge_acc,
                                        |index_acc, (window_from, window_to)| {
                                            match to {
                                                Some(to) => {
                                                    // Inject the current index if the current window is a match.
                                                    if let Some(unstable_to) =
                                                        self.vertices_mapping_right.get(&to)
                                                    {
                                                        if window_from == from
                                                            && window_to == unstable_to
                                                        {
                                                            return index_acc
                                                                .into_iter()
                                                                .chain(vec![(
                                                                    *weighted_index,
                                                                    *self
                                                                        .vertices_mapping_left
                                                                        .get(&index)
                                                                        .unwrap(),
                                                                )])
                                                                .collect::<Vec<(
                                                                    StableHyperedgeWeightedIndex,
                                                                    StableVertexIndex,
                                                                )>>(
                                                                );
                                                        }
                                                    }
                                                }
                                                None => {
                                                    // Inject the next vertex index if the current window is a match.
                                                    if window_from == from {
                                                        return index_acc
                                                            .into_iter()
                                                            .chain(vec![(
                                                                *weighted_index,
                                                                *self
                                                                    .vertices_mapping_left
                                                                    .get(window_to)
                                                                    .unwrap(),
                                                            )])
                                                            .collect::<Vec<(
                                                                StableHyperedgeWeightedIndex,
                                                                StableVertexIndex,
                                                            )>>(
                                                            );
                                                    }
                                                }
                                            }

                                            index_acc
                                        },
                                    )
                                },
                            )
                    },
                )
            }
            None => vec![],
        }
    }
}
