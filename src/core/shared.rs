use itertools::Itertools;
use rayon::prelude::*;

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

/// Enumeration of the different types of connection.
/// Only used as a guard argument for the `get_connections` method.
pub(crate) enum Connection<Index = VertexIndex> {
    In(Index),
    Out(Index),
    InAndOut(Index, Index),
}

type Connections = Vec<(HyperedgeIndex, Option<VertexIndex>)>;

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Private helper function used internally.
    /// Takes a connection as an enum and returns a vector of tuples of the
    /// form (hyperedge index, connected vertex index) where connected vertex
    /// index is an optional value - None for `InAndOut` connections.
    #[allow(clippy::type_complexity)]
    pub(crate) fn get_connections(
        &self,
        connections: &Connection,
    ) -> Result<Connections, HypergraphError<V, HE>> {
        let vertex_index = match connections {
            Connection::InAndOut(vertex_index, _)
            | Connection::In(vertex_index)
            | Connection::Out(vertex_index) => *vertex_index,
        };

        let hyperedge_indices = self.get_vertex_hyperedges(vertex_index)?;

        let hyperedges_with_vertices = hyperedge_indices
            .into_par_iter()
            .map(|he_index| {
                self.get_hyperedge_vertices(he_index)
                    .map(|vertices| (he_index, vertices))
            })
            .collect::<Result<Vec<(HyperedgeIndex, Vec<VertexIndex>)>, HypergraphError<V, HE>>>()?;

        let capacity = hyperedges_with_vertices.len();

        let results = hyperedges_with_vertices
            .into_par_iter()
            .fold_with(
                Vec::with_capacity(capacity),
                |acc, (hyperedge_index, vertices)| {
                    vertices.iter().tuple_windows::<(_, _)>().fold(
                        acc,
                        |mut acc, (window_from, window_to)| {
                            match connections {
                                Connection::In(from) => {
                                    if *window_from == *from {
                                        acc.push((hyperedge_index, Some(*window_to)));
                                    }
                                }
                                Connection::Out(to) => {
                                    if *window_to == *to {
                                        acc.push((hyperedge_index, Some(*window_from)));
                                    }
                                }
                                Connection::InAndOut(from, to) => {
                                    if *window_from == *from && *window_to == *to {
                                        acc.push((hyperedge_index, None));
                                    }
                                }
                            }
                            acc
                        },
                    )
                },
            )
            .flatten()
            .collect::<Connections>();

        Ok(results)
    }
}
