use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::trait_def::HypergraphQuery;
use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    core::disk::PersistentHypergraph,
    errors::HypergraphError,
};

impl<V, HE> HypergraphQuery<V, HE> for PersistentHypergraph<V, HE>
where
    V: VertexTrait + Serialize + DeserializeOwned,
    HE: HyperedgeTrait + Serialize + DeserializeOwned,
{
    fn count_vertices(&self) -> usize {
        PersistentHypergraph::count_vertices(self)
    }

    fn count_hyperedges(&self) -> usize {
        PersistentHypergraph::count_hyperedges(self)
    }

    fn is_empty(&self) -> bool {
        PersistentHypergraph::is_empty(self)
    }

    fn vertex_indices(&self) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        PersistentHypergraph::vertex_indices(self)
    }

    fn hyperedge_indices(&self) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        PersistentHypergraph::hyperedge_indices(self)
    }

    fn get_vertex_weight(&self, idx: VertexIndex) -> Result<V, HypergraphError<V, HE>> {
        PersistentHypergraph::get_vertex_weight(self, idx)
    }

    fn get_hyperedge_weight(&self, idx: HyperedgeIndex) -> Result<HE, HypergraphError<V, HE>> {
        PersistentHypergraph::get_hyperedge_weight(self, idx)
    }

    fn get_vertex_hyperedges(
        &self,
        idx: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        PersistentHypergraph::get_vertex_hyperedges(self, idx)
    }

    fn get_hyperedge_vertices(
        &self,
        idx: HyperedgeIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        PersistentHypergraph::get_hyperedge_vertices(self, idx)
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        HyperedgeIndex,
        HypergraphQuery,
        VertexIndex,
        core::test_support::disk::{
            EP,
            WP,
            build_persistent,
        },
    };

    #[test]
    fn count_vertices() {
        let dir = tempdir().unwrap();
        let (g, _, _) = build_persistent(dir.path());
        assert_eq!(HypergraphQuery::count_vertices(&g), 4);
    }

    #[test]
    fn count_hyperedges() {
        let dir = tempdir().unwrap();
        let (g, _, _) = build_persistent(dir.path());
        assert_eq!(HypergraphQuery::count_hyperedges(&g), 3);
    }

    #[test]
    fn is_empty_false() {
        let dir = tempdir().unwrap();
        let (g, _, _) = build_persistent(dir.path());
        assert!(!HypergraphQuery::is_empty(&g));
    }

    #[test]
    fn vertex_indices() {
        let dir = tempdir().unwrap();
        let (g, [v0, v1, v2, v3], _) = build_persistent(dir.path());
        let mut got = HypergraphQuery::vertex_indices(&g).unwrap();
        got.sort();
        assert_eq!(got, vec![v0, v1, v2, v3]);
    }

    #[test]
    fn hyperedge_indices() {
        let dir = tempdir().unwrap();
        let (g, _, [e0, e1, e2]) = build_persistent(dir.path());
        let mut got = HypergraphQuery::hyperedge_indices(&g).unwrap();
        got.sort();
        assert_eq!(got, vec![e0, e1, e2]);
    }

    #[test]
    fn get_vertex_weight() {
        let dir = tempdir().unwrap();
        let (g, [v0, _v1, _v2, _v3], _) = build_persistent(dir.path());
        assert_eq!(HypergraphQuery::get_vertex_weight(&g, v0).unwrap(), WP(0));
    }

    #[test]
    fn get_vertex_weight_not_found() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(HypergraphQuery::get_vertex_weight(&g, VertexIndex(99)).is_err());
    }

    #[test]
    fn get_hyperedge_weight() {
        let dir = tempdir().unwrap();
        let (g, _, [e0, _e1, _e2]) = build_persistent(dir.path());
        assert_eq!(
            HypergraphQuery::get_hyperedge_weight(&g, e0).unwrap(),
            EP(1)
        );
    }

    #[test]
    fn get_hyperedge_weight_not_found() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(HypergraphQuery::get_hyperedge_weight(&g, HyperedgeIndex(99)).is_err());
    }

    #[test]
    fn get_vertex_hyperedges() {
        let dir = tempdir().unwrap();
        let (g, [_v0, v1, _v2, _v3], [e0, e1, e2]) = build_persistent(dir.path());
        let mut got = HypergraphQuery::get_vertex_hyperedges(&g, v1).unwrap();
        got.sort();
        assert_eq!(got, vec![e0, e1, e2]);
    }

    #[test]
    fn get_hyperedge_vertices() {
        let dir = tempdir().unwrap();
        let (g, [v0, v1, _v2, _v3], [e0, _e1, _e2]) = build_persistent(dir.path());
        assert_eq!(
            HypergraphQuery::get_hyperedge_vertices(&g, e0).unwrap(),
            vec![v0, v1]
        );
    }
}
