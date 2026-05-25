use crate::{
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
    /// Returns the out-degree of the vertex at `from`.
    ///
    /// The out-degree is the number of directed connections that leave `from`
    /// across all hyperedges (counting each `from → successor` pair once per
    /// hyperedge). See <https://en.wikipedia.org/wiki/Directed_graph#Indegree_and_outdegree>.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `from` does not exist.
    pub fn get_vertex_degree_out(
        &self,
        from: VertexIndex,
    ) -> Result<usize, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::In(from))?;

        Ok(results.len())
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
    fn counts_outgoing_edges() {
        let (g, [_v0, v1, _v2, _v3], _) = build();
        assert_eq!(g.get_vertex_degree_out(v1).unwrap(), 2);
    }

    #[test]
    fn sink_vertex_has_zero_out_degree() {
        let (g, [_v0, _v1, v2, _v3], _) = build();
        assert_eq!(g.get_vertex_degree_out(v2).unwrap(), 0);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_vertex_degree_out(VertexIndex(99)).is_err());
    }
}
