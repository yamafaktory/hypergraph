use std::sync::atomic::Ordering;

use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::{
    PersistentHypergraph,
    helpers::{
        key_to_hyperedge,
        key_to_vertex,
        storage_err,
    },
};
use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> PersistentHypergraph<V, HE>
where
    V: VertexTrait + Serialize + DeserializeOwned,
    HE: HyperedgeTrait + Serialize + DeserializeOwned,
{
    /// Returns the number of vertices currently in the hypergraph.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn count_vertices(&self) -> usize {
        self.vertices_count.load(Ordering::Relaxed) as usize
    }

    /// Returns the number of hyperedges currently in the hypergraph.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn count_hyperedges(&self) -> usize {
        self.hyperedges_count.load(Ordering::Relaxed) as usize
    }

    /// Returns `true` if the hypergraph contains no vertices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.vertices_count.load(Ordering::Relaxed) == 0
    }

    /// Flushes all pending writes to durable storage (fsync).
    ///
    /// Normal writes are already appended to the WAL and durable on crash;
    /// this call additionally syncs the journal to the physical medium.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure.
    pub fn persist(&self) -> Result<(), HypergraphError<V, HE>> {
        self.db
            .persist(fjall::PersistMode::SyncAll)
            .map_err(storage_err)
    }

    /// Clears all vertices and hyperedges from the graph.
    ///
    /// Also clears the `vertex_refs` keyspace and the in-memory caches.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure.
    pub fn clear(&self) -> Result<(), HypergraphError<V, HE>> {
        for ks in [&self.vertices_ks, &self.hyperedges_ks, &self.vertex_refs_ks] {
            let keys: Vec<Vec<u8>> = ks
                .iter()
                .filter_map(|guard| guard.into_inner().ok().map(|(k, _)| k.to_vec()))
                .collect();
            for k in keys {
                ks.remove(k).map_err(storage_err)?;
            }
        }

        self.vertex_cache.clear();
        self.hyperedge_cache.clear();
        self.vertices_count.store(0, Ordering::Relaxed);
        self.hyperedges_count.store(0, Ordering::Relaxed);
        self.vertices_next_idx.store(0, Ordering::Relaxed);
        self.hyperedges_next_idx.store(0, Ordering::Relaxed);

        self.flush_meta()
    }

    /// Returns all vertex indices currently stored in the graph.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure.
    pub fn vertex_indices(&self) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        self.vertices_ks
            .iter()
            .map(|guard| {
                let (k, _) = guard.into_inner().map_err(storage_err)?;
                key_to_vertex(&k).ok_or_else(|| {
                    HypergraphError::StorageError("invalid vertex key in storage".into())
                })
            })
            .collect()
    }

    /// Returns all hyperedge indices currently stored in the graph.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure.
    pub fn hyperedge_indices(&self) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        self.hyperedges_ks
            .iter()
            .map(|guard| {
                let (k, _) = guard.into_inner().map_err(storage_err)?;
                key_to_hyperedge(&k).ok_or_else(|| {
                    HypergraphError::StorageError("invalid hyperedge key in storage".into())
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::core::test_support::disk::{
        EP,
        WP,
        build_persistent,
    };

    #[test]
    fn count_vertices_and_hyperedges() {
        let dir = tempdir().unwrap();
        let (g, _, _) = build_persistent(dir.path());
        assert_eq!(g.count_vertices(), 4);
        assert_eq!(g.count_hyperedges(), 3);
    }

    #[test]
    fn is_empty_false_after_add() {
        let dir = tempdir().unwrap();
        let (g, _, _) = build_persistent(dir.path());
        assert!(!g.is_empty());
    }

    #[test]
    fn is_empty_true_for_new_graph() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(g.is_empty());
    }

    #[test]
    fn vertex_indices_returns_all() {
        let dir = tempdir().unwrap();
        let (g, [v0, v1, v2, v3], _) = build_persistent(dir.path());
        let mut got = g.vertex_indices().unwrap();
        got.sort();
        assert_eq!(got, vec![v0, v1, v2, v3]);
    }

    #[test]
    fn hyperedge_indices_returns_all() {
        let dir = tempdir().unwrap();
        let (g, _, [e0, e1, e2]) = build_persistent(dir.path());
        let mut got = g.hyperedge_indices().unwrap();
        got.sort();
        assert_eq!(got, vec![e0, e1, e2]);
    }

    #[test]
    fn clear_resets_everything() {
        let dir = tempdir().unwrap();
        let (g, _, _) = build_persistent(dir.path());
        g.clear().unwrap();
        assert_eq!(g.count_vertices(), 0);
        assert_eq!(g.count_hyperedges(), 0);
        assert!(g.is_empty());
    }

    #[test]
    fn persist_does_not_error() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(g.persist().is_ok());
    }
}
