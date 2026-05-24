use std::{
    fmt,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use fjall::Keyspace;
use quick_cache::sync::Cache;
use serde::{Serialize, de::DeserializeOwned};

use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

mod hyperedges;
mod open;
mod vertices;

// ──────────────────────────────────────────────────────────────────────────────
// Type aliases
// ──────────────────────────────────────────────────────────────────────────────

/// Cached hyperedge entry: ordered vertex list + weight.
type HyperedgeArc<HE> = Arc<(Vec<VertexIndex>, HE)>;

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

const META_VERTEX_IDX: &[u8] = b"vi";
const META_VERTEX_COUNT: &[u8] = b"vc";
const META_HYPEREDGE_IDX: &[u8] = b"hi";
const META_HYPEREDGE_COUNT: &[u8] = b"hc";

pub(super) const DEFAULT_CACHE_CAPACITY: usize = 10_000;

// ──────────────────────────────────────────────────────────────────────────────
// Key helpers
// ──────────────────────────────────────────────────────────────────────────────

pub(super) fn decode_u64(bytes: &[u8]) -> u64 {
    bytes
        .get(..8)
        .and_then(|b| b.try_into().ok())
        .map_or(0, u64::from_be_bytes)
}

/// Encode a `VertexIndex` as a big-endian 8-byte key.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn vertex_key(idx: VertexIndex) -> [u8; 8] {
    (idx.0 as u64).to_be_bytes()
}

/// Encode a `HyperedgeIndex` as a big-endian 8-byte key.
#[allow(clippy::cast_possible_truncation)]
pub(super) fn hyperedge_key(idx: HyperedgeIndex) -> [u8; 8] {
    (idx.0 as u64).to_be_bytes()
}

/// Encode a `(vertex, hyperedge)` back-reference as a 16-byte key.
///
/// The first 8 bytes are the vertex index; the last 8 bytes are the hyperedge
/// index. Big-endian layout means all back-references for a given vertex are
/// contiguous in the keyspace, enabling an O(degree) prefix scan.
pub(super) fn vertex_ref_key(v: VertexIndex, he: HyperedgeIndex) -> [u8; 16] {
    let mut key = [0u8; 16];
    key[..8].copy_from_slice(&vertex_key(v));
    key[8..].copy_from_slice(&hyperedge_key(he));
    key
}

pub(super) fn key_to_vertex(key: &[u8]) -> Option<VertexIndex> {
    key.get(..8)
        .and_then(|b| b.try_into().ok())
        .map(|b: [u8; 8]| {
            #[allow(clippy::cast_possible_truncation)]
            VertexIndex(u64::from_be_bytes(b) as usize)
        })
}

pub(super) fn key_to_hyperedge(key: &[u8]) -> Option<HyperedgeIndex> {
    key.get(..8)
        .and_then(|b| b.try_into().ok())
        .map(|b: [u8; 8]| {
            #[allow(clippy::cast_possible_truncation)]
            HyperedgeIndex(u64::from_be_bytes(b) as usize)
        })
}

pub(super) fn storage_err<V, HE, E>(e: E) -> HypergraphError<V, HE>
where
    V: Copy + Eq,
    HE: Copy + Eq,
    E: fmt::Display,
{
    HypergraphError::StorageError(e.to_string())
}

// ──────────────────────────────────────────────────────────────────────────────
// Struct definition
// ──────────────────────────────────────────────────────────────────────────────

/// A directed hypergraph persisted on disk via fjall (LSM-tree) with a
/// [`quick_cache`] hot-data layer.
///
/// ## Larger-than-RAM support
///
/// fjall is the primary store; the in-memory cache is bounded. Three separate
/// fjall keyspaces are used:
///
/// | Keyspace | Key | Value |
/// |---|---|---|
/// | `vertices` | `vertex_idx (8 B)` | serialized vertex weight |
/// | `hyperedges` | `hyperedge_idx (8 B)` | serialized `(vertices, weight)` |
/// | `vertex_refs` | `vertex_idx (8 B) ‖ hyperedge_idx (8 B)` | empty |
///
/// Back-references (which hyperedges include a vertex) are stored as individual
/// 16-byte keys in `vertex_refs` rather than as an inline list inside the
/// vertex record. This means:
///
/// - **Vertex weight read**: single O(1) point lookup — never touches back-refs.
/// - **Adding/removing a back-reference**: single O(1) key insert/delete.
/// - **Getting all hyperedges for a vertex**: O(degree) prefix scan that streams
///   from disk one entry at a time, with no in-memory accumulation beyond the
///   returned `Vec`.
///
/// High-degree "hub" vertices therefore impose no special memory cost.
///
/// ## Thread safety
///
/// `PersistentHypergraph` is `Send + Sync`. All write methods take `&self` and use
/// atomic counters internally, so the same instance can be wrapped in an `Arc`
/// and shared across threads without an external `Mutex`.
///
/// Note that individual multi-step operations (e.g. `add_hyperedge`) are **not**
/// serializable with respect to concurrent writers: concurrent calls may
/// interleave. For full operation-level isolation wrap in a `Mutex`.
///
/// ## Open or create
///
/// ```ignore
/// use std::sync::Arc;
/// use hypergraph::PersistentHypergraph;
///
/// let g = Arc::new(PersistentHypergraph::<MyVertex, MyEdge>::open("/var/data/my-graph")?);
///
/// let g2 = Arc::clone(&g);
/// std::thread::spawn(move || { g2.add_vertex(my_vertex)?; Ok(()) });
/// ```
pub struct PersistentHypergraph<V, HE> {
    pub(super) db: fjall::Database,
    /// Stores serialized vertex weights, keyed by `vertex_idx`.
    pub(super) vertices_ks: Keyspace,
    /// Stores serialized `(vertex_list, weight)` tuples, keyed by `hyperedge_idx`.
    pub(super) hyperedges_ks: Keyspace,
    /// Stores back-references as presence-only 16-byte keys
    /// `vertex_idx ‖ hyperedge_idx`. No value payload.
    pub(super) vertex_refs_ks: Keyspace,
    pub(super) meta_ks: Keyspace,
    /// Hot cache: `vertex_idx` → deserialized weight `V`.
    pub(super) vertex_cache: Cache<u64, V>,
    /// Hot cache: `hyperedge_idx` → `Arc<(Vec<VertexIndex>, HE)>`.
    pub(super) hyperedge_cache: Cache<u64, HyperedgeArc<HE>>,
    /// Monotonically increasing counter; never decrements.
    pub(super) vertices_next_idx: AtomicU64,
    /// Actual number of vertices currently in the graph.
    pub(super) vertices_count: AtomicU64,
    /// Monotonically increasing counter; never decrements.
    pub(super) hyperedges_next_idx: AtomicU64,
    /// Actual number of hyperedges currently in the graph.
    pub(super) hyperedges_count: AtomicU64,
    pub(super) _phantom: PhantomData<(V, HE)>,
}

impl<V, HE> fmt::Debug for PersistentHypergraph<V, HE>
where
    V: VertexTrait,
    HE: HyperedgeTrait,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PersistentHypergraph")
            .field("vertices", &self.vertices_count.load(Ordering::Relaxed))
            .field("hyperedges", &self.hyperedges_count.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Internal helpers
// ──────────────────────────────────────────────────────────────────────────────

impl<V, HE> PersistentHypergraph<V, HE>
where
    V: VertexTrait + Serialize + DeserializeOwned,
    HE: HyperedgeTrait + Serialize + DeserializeOwned,
{
    /// Fetches the weight of a vertex from cache, falling back to disk.
    pub(super) fn load_vertex(&self, idx: VertexIndex) -> Result<V, HypergraphError<V, HE>> {
        let raw_key = idx.0 as u64;

        if let Some(cached) = self.vertex_cache.get(&raw_key) {
            return Ok(cached);
        }

        let bytes = self
            .vertices_ks
            .get(vertex_key(idx))
            .map_err(storage_err)?
            .ok_or(HypergraphError::VertexIndexNotFound(idx))?;

        let weight: V = postcard::from_bytes(&bytes).map_err(storage_err)?;
        self.vertex_cache.insert(raw_key, weight);
        Ok(weight)
    }

    /// Serializes and stores a vertex weight, updating the cache.
    pub(super) fn store_vertex(
        &self,
        idx: VertexIndex,
        weight: V,
    ) -> Result<(), HypergraphError<V, HE>> {
        let bytes = postcard::to_allocvec(&weight).map_err(storage_err)?;
        self.vertices_ks
            .insert(vertex_key(idx), bytes.as_slice())
            .map_err(storage_err)?;
        self.vertex_cache.insert(idx.0 as u64, weight);
        Ok(())
    }

    /// Removes a vertex weight from disk and cache.
    pub(super) fn delete_vertex(&self, idx: VertexIndex) -> Result<(), HypergraphError<V, HE>> {
        self.vertices_ks
            .remove(vertex_key(idx))
            .map_err(storage_err)?;
        self.vertex_cache.remove(&(idx.0 as u64));
        Ok(())
    }

    /// Records that `he` includes `v` by inserting a 16-byte presence key.
    pub(super) fn add_vertex_ref(
        &self,
        v: VertexIndex,
        he: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        self.vertex_refs_ks
            .insert(vertex_ref_key(v, he), [])
            .map_err(storage_err)
    }

    /// Removes the back-reference `(v, he)`.
    pub(super) fn remove_vertex_ref(
        &self,
        v: VertexIndex,
        he: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        self.vertex_refs_ks
            .remove(vertex_ref_key(v, he))
            .map_err(storage_err)
    }

    /// Returns all hyperedge indices that include `v` via a prefix scan.
    ///
    /// Streams one 16-byte key at a time from disk. Memory usage is O(degree)
    /// for the returned `Vec`, not proportional to the entire back-ref keyspace.
    pub(super) fn load_vertex_refs(
        &self,
        v: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        self.vertex_refs_ks
            .prefix(vertex_key(v))
            .map(|guard| {
                let (k, _) = guard.into_inner().map_err(storage_err)?;
                let he_bytes: [u8; 8] = k
                    .get(8..16)
                    .and_then(|b| b.try_into().ok())
                    .ok_or_else(|| {
                        HypergraphError::StorageError(
                            "invalid vertex ref key length".into(),
                        )
                    })?;
                #[allow(clippy::cast_possible_truncation)]
                Ok(HyperedgeIndex(u64::from_be_bytes(he_bytes) as usize))
            })
            .collect()
    }

    /// Deletes all back-reference keys for `v` from the `vertex_refs` keyspace.
    pub(super) fn delete_vertex_refs(
        &self,
        v: VertexIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        let keys: Vec<[u8; 16]> = self
            .vertex_refs_ks
            .prefix(vertex_key(v))
            .filter_map(|guard| {
                guard
                    .into_inner()
                    .ok()
                    .and_then(|(k, _)| k.get(..16).and_then(|b| b.try_into().ok()))
            })
            .collect();

        for key in keys {
            self.vertex_refs_ks.remove(key).map_err(storage_err)?;
        }
        Ok(())
    }

    pub(super) fn load_hyperedge(
        &self,
        idx: HyperedgeIndex,
    ) -> Result<HyperedgeArc<HE>, HypergraphError<V, HE>> {
        let raw_key = idx.0 as u64;

        if let Some(cached) = self.hyperedge_cache.get(&raw_key) {
            return Ok(cached);
        }

        let bytes = self
            .hyperedges_ks
            .get(hyperedge_key(idx))
            .map_err(storage_err)?
            .ok_or(HypergraphError::HyperedgeIndexNotFound(idx))?;

        let entry: (Vec<VertexIndex>, HE) =
            postcard::from_bytes(&bytes).map_err(storage_err)?;

        let arc = Arc::new(entry);
        self.hyperedge_cache.insert(raw_key, arc.clone());
        Ok(arc)
    }

    pub(super) fn store_hyperedge(
        &self,
        idx: HyperedgeIndex,
        vertices: &[VertexIndex],
        weight: HE,
    ) -> Result<(), HypergraphError<V, HE>> {
        let bytes = postcard::to_allocvec(&(vertices, weight)).map_err(storage_err)?;
        self.hyperedges_ks
            .insert(hyperedge_key(idx), bytes.as_slice())
            .map_err(storage_err)?;
        self.hyperedge_cache
            .insert(idx.0 as u64, Arc::new((vertices.to_vec(), weight)));
        Ok(())
    }

    pub(super) fn delete_hyperedge(
        &self,
        idx: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        self.hyperedges_ks
            .remove(hyperedge_key(idx))
            .map_err(storage_err)?;
        self.hyperedge_cache.remove(&(idx.0 as u64));
        Ok(())
    }

    pub(super) fn flush_meta(&self) -> Result<(), HypergraphError<V, HE>> {
        let vi = self.vertices_next_idx.load(Ordering::Relaxed);
        let vc = self.vertices_count.load(Ordering::Relaxed);
        let hi = self.hyperedges_next_idx.load(Ordering::Relaxed);
        let hc = self.hyperedges_count.load(Ordering::Relaxed);

        let mut batch = self.db.batch();
        batch.insert(&self.meta_ks, META_VERTEX_IDX, vi.to_be_bytes().as_slice());
        batch.insert(&self.meta_ks, META_VERTEX_COUNT, vc.to_be_bytes().as_slice());
        batch.insert(&self.meta_ks, META_HYPEREDGE_IDX, hi.to_be_bytes().as_slice());
        batch.insert(&self.meta_ks, META_HYPEREDGE_COUNT, hc.to_be_bytes().as_slice());
        batch.commit().map_err(storage_err)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Graph-level utilities
// ──────────────────────────────────────────────────────────────────────────────

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
