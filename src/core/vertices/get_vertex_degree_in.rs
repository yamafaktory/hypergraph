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
    /// Returns the in-degree of the vertex at `to`.
    ///
    /// The in-degree is the number of directed connections that arrive at `to`
    /// across all hyperedges (counting each `predecessor → to` pair once per
    /// hyperedge). See <https://en.wikipedia.org/wiki/Directed_graph#Indegree_and_outdegree>.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `to` does not exist.
    pub fn get_vertex_degree_in(&self, to: VertexIndex) -> Result<usize, HypergraphError<V, HE>> {
        let results = self.get_connections(&Connection::Out(to))?;

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
    fn counts_incoming_edges() {
        let (g, [_v0, v1, _v2, _v3], _) = build();
        assert_eq!(g.get_vertex_degree_in(v1).unwrap(), 1);
    }

    #[test]
    fn source_vertex_has_zero_in_degree() {
        let (g, [v0, _v1, _v2, _v3], _) = build();
        assert_eq!(g.get_vertex_degree_in(v0).unwrap(), 0);
    }

    #[test]
    fn not_found_returns_error() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(g.get_vertex_degree_in(VertexIndex(99)).is_err());
    }
}
