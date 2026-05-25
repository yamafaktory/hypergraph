use super::trait_def::HypergraphQuery;
use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    Hypergraph,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> HypergraphQuery<V, HE> for Hypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    fn count_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn count_hyperedges(&self) -> usize {
        self.hyperedges.len()
    }

    fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    fn vertex_indices(&self) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        Ok(self.vertices.keys().copied().collect())
    }

    fn hyperedge_indices(&self) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        Ok(self.hyperedges.keys().copied().collect())
    }

    fn get_vertex_weight(&self, idx: VertexIndex) -> Result<V, HypergraphError<V, HE>> {
        self.vertices
            .get(&idx)
            .map(|(w, _)| *w)
            .ok_or(HypergraphError::VertexIndexNotFound(idx))
    }

    fn get_hyperedge_weight(&self, idx: HyperedgeIndex) -> Result<HE, HypergraphError<V, HE>> {
        self.hyperedges
            .get(&idx)
            .map(|(_, w)| *w)
            .ok_or(HypergraphError::HyperedgeIndexNotFound(idx))
    }

    fn get_vertex_hyperedges(
        &self,
        idx: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        self.vertices
            .get(&idx)
            .map(|(_, he_set)| he_set.iter().copied().collect())
            .ok_or(HypergraphError::VertexIndexNotFound(idx))
    }

    fn get_hyperedge_vertices(
        &self,
        idx: HyperedgeIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        self.hyperedges
            .get(&idx)
            .map(|(v, _)| v.clone())
            .ok_or(HypergraphError::HyperedgeIndexNotFound(idx))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        HyperedgeIndex,
        Hypergraph,
        HypergraphQuery,
        VertexIndex,
        core::test_support::{
            E,
            W,
            build,
        },
    };

    #[test]
    fn count_vertices() {
        let (g, _, _) = build();
        assert_eq!(HypergraphQuery::count_vertices(&g), 4);
    }

    #[test]
    fn count_hyperedges() {
        let (g, _, _) = build();
        assert_eq!(HypergraphQuery::count_hyperedges(&g), 3);
    }

    #[test]
    fn is_empty_false() {
        let (g, _, _) = build();
        assert!(!HypergraphQuery::is_empty(&g));
    }

    #[test]
    fn is_empty_true() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(HypergraphQuery::is_empty(&g));
    }

    #[test]
    fn vertex_indices() {
        let (g, [v0, v1, v2, v3], _) = build();
        let mut got = HypergraphQuery::vertex_indices(&g).unwrap();
        got.sort();
        assert_eq!(got, vec![v0, v1, v2, v3]);
    }

    #[test]
    fn hyperedge_indices() {
        let (g, _, [e0, e1, e2]) = build();
        let mut got = HypergraphQuery::hyperedge_indices(&g).unwrap();
        got.sort();
        assert_eq!(got, vec![e0, e1, e2]);
    }

    #[test]
    fn get_vertex_weight() {
        let (g, [v0, _v1, _v2, _v3], _) = build();
        assert_eq!(HypergraphQuery::get_vertex_weight(&g, v0).unwrap(), W(0));
    }

    #[test]
    fn get_vertex_weight_not_found() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(HypergraphQuery::get_vertex_weight(&g, VertexIndex(99)).is_err());
    }

    #[test]
    fn get_hyperedge_weight() {
        let (g, _, [e0, _e1, _e2]) = build();
        assert_eq!(HypergraphQuery::get_hyperedge_weight(&g, e0).unwrap(), E(1));
    }

    #[test]
    fn get_hyperedge_weight_not_found() {
        let g: Hypergraph<W, E> = Hypergraph::new();
        assert!(HypergraphQuery::get_hyperedge_weight(&g, HyperedgeIndex(99)).is_err());
    }

    #[test]
    fn get_vertex_hyperedges() {
        let (g, [_v0, v1, _v2, _v3], [e0, e1, e2]) = build();
        let mut got = HypergraphQuery::get_vertex_hyperedges(&g, v1).unwrap();
        got.sort();
        assert_eq!(got, vec![e0, e1, e2]);
    }

    #[test]
    fn get_hyperedge_vertices() {
        let (g, [v0, v1, _v2, _v3], [e0, _e1, _e2]) = build();
        assert_eq!(
            HypergraphQuery::get_hyperedge_vertices(&g, e0).unwrap(),
            vec![v0, v1]
        );
    }
}
