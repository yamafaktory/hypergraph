use std::sync::atomic::Ordering;

use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::PersistentHypergraph;
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
    /// Adds a vertex and returns its stable [`VertexIndex`].
    ///
    /// This method takes `&self` and is safe to call from multiple threads
    /// concurrently. Each call atomically reserves a unique index.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::StorageError`] on I/O failure.
    pub fn add_vertex(&self, weight: V) -> Result<VertexIndex, HypergraphError<V, HE>> {
        #[allow(clippy::cast_possible_truncation)]
        let idx = VertexIndex(self.vertices_next_idx.fetch_add(1, Ordering::Relaxed) as usize);
        self.vertices_count.fetch_add(1, Ordering::Relaxed);
        self.store_vertex(idx, weight)?;
        self.flush_meta()?;
        Ok(idx)
    }

    /// Returns the weight of the vertex at `vertex_index`.
    ///
    /// The result is served from the in-memory cache when available, otherwise
    /// a single point lookup is performed against the disk store.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist, or [`HypergraphError::StorageError`] on I/O failure.
    pub fn get_vertex_weight(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<V, HypergraphError<V, HE>> {
        self.load_vertex(vertex_index)
    }

    /// Updates the weight of the vertex at `vertex_index`.
    ///
    /// Only the weight record is touched; back-references are stored separately
    /// and are unaffected.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist, [`HypergraphError::VertexWeightUnchanged`] if `weight` equals
    /// the current weight, or [`HypergraphError::StorageError`] on I/O failure.
    pub fn update_vertex_weight(
        &self,
        vertex_index: VertexIndex,
        weight: V,
    ) -> Result<(), HypergraphError<V, HE>> {
        let current = self.load_vertex(vertex_index)?;
        if current == weight {
            return Err(HypergraphError::VertexWeightUnchanged {
                index: vertex_index,
                weight,
            });
        }
        self.store_vertex(vertex_index, weight)
    }

    /// Returns the indices of all hyperedges that include `vertex_index`.
    ///
    /// Implemented as a prefix scan over the `vertex_refs` keyspace, streaming
    /// one 16-byte key per hyperedge from disk. Memory usage is proportional
    /// to the degree of the vertex, not to the total graph size.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist, or [`HypergraphError::StorageError`] on I/O failure.
    pub fn get_vertex_hyperedges(
        &self,
        vertex_index: VertexIndex,
    ) -> Result<Vec<HyperedgeIndex>, HypergraphError<V, HE>> {
        self.load_vertex(vertex_index)?;
        self.load_vertex_refs(vertex_index)
    }

    /// Removes the vertex at `vertex_index`.
    ///
    /// Hyperedges that become empty after the removal are deleted. Hyperedges
    /// with remaining vertices have the removed vertex stripped from their
    /// vertex list. All back-references for this vertex are deleted via a
    /// single prefix scan over the `vertex_refs` keyspace.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::VertexIndexNotFound`] if `vertex_index` does
    /// not exist, or [`HypergraphError::StorageError`] on I/O failure.
    pub fn remove_vertex(&self, vertex_index: VertexIndex) -> Result<(), HypergraphError<V, HE>> {
        self.load_vertex(vertex_index)?;
        let he_indices = self.load_vertex_refs(vertex_index)?;

        for he_idx in he_indices {
            let vertices = self.get_hyperedge_vertices(he_idx)?;

            let unique_count = {
                let mut tmp = vertices.clone();
                tmp.sort_unstable();
                tmp.dedup();
                tmp.len()
            };

            if unique_count == 1 {
                // vertex_index is the sole unique member; delete the whole hyperedge.
                self.delete_hyperedge(he_idx)?;
                self.hyperedges_count.fetch_sub(1, Ordering::Relaxed);
            } else {
                let new_verts: Vec<VertexIndex> = vertices
                    .into_iter()
                    .filter(|&v| v != vertex_index)
                    .collect();
                let he_weight = self.load_hyperedge(he_idx)?.1;
                self.store_hyperedge(he_idx, &new_verts, he_weight)?;
            }
        }

        // Remove all (vertex_index, *) entries from vertex_refs in one prefix scan.
        self.delete_vertex_refs(vertex_index)?;
        self.delete_vertex(vertex_index)?;
        self.vertices_count.fetch_sub(1, Ordering::Relaxed);
        self.flush_meta()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        VertexIndex,
        core::test_support::disk::{
            EP,
            WP,
            build_persistent,
        },
    };

    #[test]
    fn add_vertex_returns_sequential_indices() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        let i0 = g.add_vertex(WP(0)).unwrap();
        let i1 = g.add_vertex(WP(1)).unwrap();
        assert_eq!(i0, VertexIndex(0));
        assert_eq!(i1, VertexIndex(1));
    }

    #[test]
    fn get_vertex_weight_returns_value() {
        let dir = tempdir().unwrap();
        let (g, [v0, _v1, _v2, _v3], _) = build_persistent(dir.path());
        assert_eq!(g.get_vertex_weight(v0).unwrap(), WP(0));
    }

    #[test]
    fn get_vertex_weight_not_found() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(g.get_vertex_weight(VertexIndex(99)).is_err());
    }

    #[test]
    fn update_vertex_weight() {
        let dir = tempdir().unwrap();
        let (g, [v0, _v1, _v2, _v3], _) = build_persistent(dir.path());
        g.update_vertex_weight(v0, WP(99)).unwrap();
        assert_eq!(g.get_vertex_weight(v0).unwrap(), WP(99));
    }

    #[test]
    fn update_vertex_weight_unchanged_returns_error() {
        let dir = tempdir().unwrap();
        let (g, [v0, _v1, _v2, _v3], _) = build_persistent(dir.path());
        assert!(g.update_vertex_weight(v0, WP(0)).is_err());
    }

    #[test]
    fn get_vertex_hyperedges_returns_indices() {
        let dir = tempdir().unwrap();
        let (g, [_v0, v1, _v2, _v3], [e0, e1, e2]) = build_persistent(dir.path());
        let mut got = g.get_vertex_hyperedges(v1).unwrap();
        got.sort();
        assert_eq!(got, vec![e0, e1, e2]);
    }

    #[test]
    fn remove_vertex_decrements_count() {
        let dir = tempdir().unwrap();
        let (g, [v0, _v1, _v2, _v3], _) = build_persistent(dir.path());
        g.remove_vertex(v0).unwrap();
        assert_eq!(g.count_vertices(), 3);
    }
}
