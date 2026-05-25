use indexmap::IndexMap;
use itertools::{
    Itertools,
    fold,
};

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    core::shared::Connection,
    errors::HypergraphError,
};

#[allow(clippy::type_complexity)]
impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns all vertices that have a directed connection into `to`, grouped
    /// with the hyperedges through which they reach it.
    ///
    /// Each element of the result is `(predecessor, hyperedges)` where
    /// `hyperedges` lists every hyperedge that connects `predecessor` to `to`.
    /// This is the incoming-edge counterpart of
    /// [`get_full_adjacent_vertices_from`](Self::get_full_adjacent_vertices_from).
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `to` does not exist.
    pub fn get_full_adjacent_vertices_to(
        &self,
        to: VertexIndex,
    ) -> Result<Vec<(VertexIndex, Vec<HyperedgeIndex>)>, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::Out(to))?;

        Ok(fold(
            results,
            IndexMap::<VertexIndex, Vec<HyperedgeIndex>>::new(),
            |mut acc, (hyperedge_index, vertex_index)| {
                if let Some(index) = vertex_index {
                    let hyperedges = acc.entry(index).or_default();

                    hyperedges.push(hyperedge_index);
                }

                acc
            },
        )
        .into_iter()
        .collect_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        VertexIndex,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn returns_predecessor_with_hyperedges() {
        let (g, [v0, v1, _v2, _v3], [e0, _e1, _e2]) = build();
        let got = g.get_full_adjacent_vertices_to(v1).unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].0, v0);
        assert_eq!(got[0].1, vec![e0]);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_full_adjacent_vertices_to(VertexIndex(99)).is_err());
    }
}
