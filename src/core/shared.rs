use std::cmp::Ordering;

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

/// Min-heap entry used by Dijkstra implementations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Visitor {
    pub(crate) distance: usize,
    pub(crate) index: VertexIndex,
}

impl Visitor {
    pub(crate) fn new(distance: usize, index: VertexIndex) -> Self {
        Self { distance, index }
    }
}

impl Ord for Visitor {
    fn cmp(&self, other: &Visitor) -> Ordering {
        other
            .distance
            .cmp(&self.distance)
            .then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialOrd for Visitor {
    fn partial_cmp(&self, other: &Visitor) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

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

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use crate::{
        VertexIndex,
        core::shared::Visitor,
    };

    #[test]
    fn new_sets_fields() {
        let v = Visitor::new(5, VertexIndex(3));
        assert_eq!(v.distance, 5);
        assert_eq!(v.index, VertexIndex(3));
    }

    #[test]
    fn min_heap_pops_smallest_distance_first() {
        let mut heap = BinaryHeap::new();
        heap.push(Visitor::new(10, VertexIndex(0)));
        heap.push(Visitor::new(1, VertexIndex(1)));
        heap.push(Visitor::new(5, VertexIndex(2)));
        assert_eq!(heap.pop().unwrap().distance, 1);
        assert_eq!(heap.pop().unwrap().distance, 5);
        assert_eq!(heap.pop().unwrap().distance, 10);
    }

    #[test]
    fn equal_distance_breaks_tie_by_larger_index_first() {
        let mut heap = BinaryHeap::new();
        heap.push(Visitor::new(3, VertexIndex(2)));
        heap.push(Visitor::new(3, VertexIndex(5)));
        // tie-break is self.index.cmp(&other.index) — not reversed — so
        // the larger index wins in the underlying max-heap and pops first.
        assert_eq!(heap.pop().unwrap().index, VertexIndex(5));
    }
}
