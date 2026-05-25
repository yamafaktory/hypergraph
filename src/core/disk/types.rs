use std::{
    fmt,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{
            AtomicU64,
            Ordering,
        },
    },
};

use fjall::Keyspace;
use quick_cache::sync::Cache;

use crate::{
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
};

/// Cached hyperedge entry: ordered vertex list + weight.
pub(super) type HyperedgeArc<HE> = Arc<(Vec<VertexIndex>, HE)>;

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

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::core::test_support::disk::build_persistent;

    #[test]
    fn debug_contains_counts() {
        let dir = TempDir::new().unwrap();
        let (g, _, _) = build_persistent(dir.path());
        let s = format!("{g:?}");
        assert!(s.contains("PersistentHypergraph"));
        assert!(s.contains("vertices: 4"));
        assert!(s.contains("hyperedges: 3"));
    }
}
