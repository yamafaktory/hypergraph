use crate::{errors::HypergraphError, HyperedgeIndex, Hypergraph, SharedTrait, VertexIndex};

use itertools::Itertools;

/// Enumeration of the different types of connection.
/// Only used as a guard argument for the `get_connections` method.
pub(crate) enum Connection<Index = VertexIndex> {
    In(Index),
    Out(Index),
    InAndOut(Index, Index),
}

impl<V, HE> Hypergraph<V, HE>
where
    V: SharedTrait,
    HE: SharedTrait,
{
    /// Private helper function used internally.
    /// Takes a connection as an enum and returns a vector of tuples of the
    /// form (hyperedge index, connected vertex index) where connected vertex
    /// index is an optional value - None for InAndOut connections.
    #[allow(clippy::type_complexity)]
    pub(crate) fn get_connections(
        &self,
        connections: Connection,
    ) -> Result<Vec<(HyperedgeIndex, Option<VertexIndex>)>, HypergraphError<V, HE>> {
        let internal_index = self.get_internal_vertex(match connections {
            Connection::In(vertex_index) | Connection::Out(vertex_index) => vertex_index,
            Connection::InAndOut(vertex_index, _) => vertex_index,
        })?;

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
                        match connections {
                            Connection::In(from) => {
                                // Inject the index of the hyperedge and the
                                // vertex index if the current window is a
                                // match.
                                if *window_from == from {
                                    return index_acc
                                        .into_iter()
                                        .chain(vec![(hyperedge_index, Some(*window_to))])
                                        .collect_vec();
                                }
                            }
                            Connection::Out(to) => {
                                // Inject the index of the hyperedge and the
                                // vertex index if the current window is a
                                // match.
                                if *window_to == to {
                                    return index_acc
                                        .into_iter()
                                        .chain(vec![(hyperedge_index, Some(*window_from))])
                                        .collect_vec();
                                }
                            }
                            Connection::InAndOut(from, to) => {
                                // Inject only the index of the hyperedge
                                // if the current window is a match.
                                if *window_from == from && *window_to == to {
                                    return index_acc
                                        .into_iter()
                                        .chain(vec![(hyperedge_index, None)])
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
