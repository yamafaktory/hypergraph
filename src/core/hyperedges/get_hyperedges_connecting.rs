use rayon::prelude::*;

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    core::shared::Connection,
    errors::HypergraphError,
};

impl<V, HE> Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    /// Returns the indices of all hyperedges that contain a direct `from → to`
    /// consecutive connection.
    ///
    /// A hyperedge qualifies when `from` and `to` appear as adjacent entries in
    /// its vertex list (in that order). Supports self-loops when `from == to`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if either `from` or `to`
    /// does not exist.
    pub fn get_hyperedges_connecting(
        &self,
        from: VertexIndex,
        to: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::InAndOut(from, to))?;

        Ok(results
            .into_par_iter()
            .map(|(hyperedged_index, _)| hyperedged_index)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Hypergraph,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn finds_connecting_hyperedge() {
        let (g, [v0, v1, _v2, _v3], [e0, _e1, _e2]) = build();
        assert_eq!(g.get_hyperedges_connecting(v0, v1).unwrap(), vec![e0]);
    }

    #[test]
    fn returns_empty_when_no_direct_connection() {
        let (g, [v0, _v1, v2, _v3], _) = build();
        assert!(g.get_hyperedges_connecting(v0, v2).unwrap().is_empty());
    }
}
