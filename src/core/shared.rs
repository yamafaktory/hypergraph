use crate::{Hypergraph, SharedTrait, StableVertexIndex};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Private helper function used internally.
    pub(crate) fn get_connections(
        &self,
        from: StableVertexIndex,
        to: Option<StableVertexIndex>,
    ) -> Vec<StableVertexIndex> {
        match self.vertices_mapping_right.get(&from) {
            Some(from) => {
                self.vertices.get_index(*from).iter().fold(
                    Vec::new(),
                    |acc: Vec<StableVertexIndex>, (_, hyperedges)| {
                        hyperedges.iter().enumerate().fold(
                            acc,
                            |hyperedge_acc, (index, hyperedge)| {
                                hyperedge.iter().tuple_windows::<(_, _)>().fold(
                                    hyperedge_acc,
                                    |index_acc, (window_from, window_to)| {
                                        match to {
                                            Some(to) => {
                                                // Inject the index of the hyperedge if the current window is a match.
                                                if let Some(unstable_to) =
                                                    self.vertices_mapping_right.get(&to)
                                                {
                                                    if window_from == from
                                                        && window_to == unstable_to
                                                    {
                                                        return index_acc
                                                            .into_iter()
                                                            .chain(vec![*self
                                                                .vertices_mapping_left
                                                                .get(&index)
                                                                .unwrap()])
                                                            .collect::<Vec<StableVertexIndex>>();
                                                    }
                                                }
                                            }
                                            None => {
                                                // Inject the next vertex if the current window is a match.
                                                if window_from == from {
                                                    return index_acc
                                                        .into_iter()
                                                        .chain(vec![*self
                                                            .vertices_mapping_left
                                                            .get(window_to)
                                                            .unwrap()])
                                                        .collect::<Vec<StableVertexIndex>>();
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
