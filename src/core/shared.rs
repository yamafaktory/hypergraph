use crate::{HyperedgeIndex, Hypergraph, SharedTrait, VertexIndex};

use itertools::Itertools;

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Private helper function used internally.
    pub(crate) fn get_connections(
        &self,
        from: VertexIndex,
        to: Option<VertexIndex>,
    ) -> Vec<HyperedgeIndex> {
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
}
