use std::{
    marker::PhantomData,
    path::Path,
    sync::atomic::AtomicU64,
};

use fjall::{
    Database,
    KeyspaceCreateOptions,
};
use quick_cache::sync::Cache;
use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::{
    PersistentHypergraph,
    helpers::{
        DEFAULT_CACHE_CAPACITY,
        META_HYPEREDGE_COUNT,
        META_HYPEREDGE_IDX,
        META_VERTEX_COUNT,
        META_VERTEX_IDX,
        decode_u64,
        storage_err,
    },
};
use crate::{
    HyperedgeTrait,
    VertexTrait,
    errors::HypergraphError,
};

impl<V, HE> PersistentHypergraph<V, HE>
where
    V: VertexTrait + Serialize + DeserializeOwned,
    HE: HyperedgeTrait + Serialize + DeserializeOwned,
{
    /// Opens (or creates) a persistent hypergraph at `path`.
    ///
    /// If `path` does not exist, an empty hypergraph is created. If it already
    /// holds hypergraph data, that data is recovered automatically.
    ///
    /// The returned value is `Send + Sync` and can be wrapped in an `Arc` for
    /// sharing across threads.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] if the database cannot be
    /// opened or the metadata cannot be read.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, HypergraphError<V, HE>> {
        Self::open_with_capacity(path, DEFAULT_CACHE_CAPACITY)
    }

    /// Opens a persistent hypergraph with a custom hot-data cache capacity.
    ///
    /// `cache_capacity` controls how many entries each cache layer (vertex
    /// weights and hyperedges) holds before evicting the least-recently-used
    /// entries. The default when using [`open`](Self::open) is 10 000.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] if the database cannot be
    /// opened.
    pub fn open_with_capacity(
        path: impl AsRef<Path>,
        cache_capacity: usize,
    ) -> Result<Self, HypergraphError<V, HE>> {
        let db = Database::builder(path.as_ref())
            .open()
            .map_err(storage_err)?;

        let vertices_ks = db
            .keyspace("vertices", KeyspaceCreateOptions::default)
            .map_err(storage_err)?;
        let hyperedges_ks = db
            .keyspace("hyperedges", KeyspaceCreateOptions::default)
            .map_err(storage_err)?;
        let vertex_refs_ks = db
            .keyspace("vertex_refs", KeyspaceCreateOptions::default)
            .map_err(storage_err)?;
        let meta_ks = db
            .keyspace("meta", KeyspaceCreateOptions::default)
            .map_err(storage_err)?;

        let vertices_next_idx = meta_ks
            .get(META_VERTEX_IDX)
            .map_err(storage_err)?
            .as_deref()
            .map_or(0, decode_u64);

        let vertices_count = meta_ks
            .get(META_VERTEX_COUNT)
            .map_err(storage_err)?
            .as_deref()
            .map_or(0, decode_u64);

        let hyperedges_next_idx = meta_ks
            .get(META_HYPEREDGE_IDX)
            .map_err(storage_err)?
            .as_deref()
            .map_or(0, decode_u64);

        let hyperedges_count = meta_ks
            .get(META_HYPEREDGE_COUNT)
            .map_err(storage_err)?
            .as_deref()
            .map_or(0, decode_u64);

        Ok(Self {
            db,
            vertices_ks,
            hyperedges_ks,
            vertex_refs_ks,
            meta_ks,
            vertex_cache: Cache::new(cache_capacity),
            hyperedge_cache: Cache::new(cache_capacity),
            vertices_next_idx: AtomicU64::new(vertices_next_idx),
            vertices_count: AtomicU64::new(vertices_count),
            hyperedges_next_idx: AtomicU64::new(hyperedges_next_idx),
            hyperedges_count: AtomicU64::new(hyperedges_count),
            _phantom: PhantomData,
        })
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
    fn open_creates_empty_graph() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert_eq!(g.count_vertices(), 0);
        assert_eq!(g.count_hyperedges(), 0);
        assert!(g.is_empty());
    }

    #[test]
    fn open_with_capacity_creates_empty_graph() {
        let dir = tempdir().unwrap();
        let g =
            crate::core::disk::PersistentHypergraph::<WP, EP>::open_with_capacity(dir.path(), 100)
                .unwrap();
        assert_eq!(g.count_vertices(), 0);
    }

    #[test]
    fn reopen_recovers_data() {
        let dir = tempdir().unwrap();
        {
            let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
            g.add_vertex(WP(1)).unwrap();
            g.add_vertex(WP(2)).unwrap();
        }
        let g2 = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert_eq!(g2.count_vertices(), 2);
    }

    // Silence unused import warning for build_persistent when only some tests use it
    fn _use_build_persistent(dir: &std::path::Path) {
        let _ = build_persistent(dir);
    }
}
