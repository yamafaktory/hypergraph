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
    /// Adds a hyperedge connecting `vertices` with the given `weight` and
    /// returns its stable [`HyperedgeIndex`].
    ///
    /// Each unique vertex in `vertices` receives an O(1) back-reference insert
    /// into the `vertex_refs` keyspace; no vertex weight data is modified.
    ///
    /// This method takes `&self` and is safe to call from multiple threads.
    /// Each call atomically reserves a unique [`HyperedgeIndex`].
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeCreationNoVertices`] if `vertices`
    /// is empty, [`HypergraphError::VertexIndexNotFound`] if any vertex does
    /// not exist, or [`HypergraphError::StorageError`] on I/O failure.
    pub fn add_hyperedge(
        &self,
        vertices: &[VertexIndex],
        weight: HE,
    ) -> Result<HyperedgeIndex, HypergraphError<V, HE>> {
        if vertices.is_empty() {
            return Err(HypergraphError::HyperedgeCreationNoVertices(weight));
        }

        for &v in vertices {
            self.load_vertex(v)?;
        }

        #[allow(clippy::cast_possible_truncation)]
        let he_idx =
            HyperedgeIndex(self.hyperedges_next_idx.fetch_add(1, Ordering::Relaxed) as usize);
        self.hyperedges_count.fetch_add(1, Ordering::Relaxed);

        self.store_hyperedge(he_idx, vertices, weight)?;

        let mut unique_verts = vertices.to_vec();
        unique_verts.sort_unstable();
        unique_verts.dedup();

        for v in unique_verts {
            self.add_vertex_ref(v, he_idx)?;
        }

        self.flush_meta()?;
        Ok(he_idx)
    }

    /// Returns the weight of the hyperedge at `hyperedge_index`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, or [`HypergraphError::StorageError`] on I/O failure.
    pub fn get_hyperedge_weight(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<HE, HypergraphError<V, HE>> {
        self.load_hyperedge(hyperedge_index).map(|arc| arc.1)
    }

    /// Returns the ordered vertex list of the hyperedge at `hyperedge_index`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, or [`HypergraphError::StorageError`] on I/O failure.
    pub fn get_hyperedge_vertices(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<Vec<VertexIndex>, HypergraphError<V, HE>> {
        self.load_hyperedge(hyperedge_index)
            .map(|arc| arc.0.clone())
    }

    /// Updates the weight of the hyperedge at `hyperedge_index`.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, [`HypergraphError::HyperedgeWeightUnchanged`] if
    /// `weight` equals the current weight, or
    /// [`HypergraphError::StorageError`] on I/O failure.
    pub fn update_hyperedge_weight(
        &self,
        hyperedge_index: HyperedgeIndex,
        weight: HE,
    ) -> Result<(), HypergraphError<V, HE>> {
        let entry = self.load_hyperedge(hyperedge_index)?;
        if entry.1 == weight {
            return Err(HypergraphError::HyperedgeWeightUnchanged {
                index: hyperedge_index,
                weight,
            });
        }
        self.store_hyperedge(hyperedge_index, &entry.0, weight)
    }

    /// Replaces the vertex list of the hyperedge at `hyperedge_index`.
    ///
    /// Back-references are updated with O(1) inserts/deletes per changed
    /// vertex: vertices added to the list receive a new `vertex_refs` entry;
    /// vertices removed from the list have their entry deleted. No vertex
    /// weight data is loaded or modified.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeUpdateNoVertices`] if `vertices` is
    /// empty, [`HypergraphError::VertexIndexNotFound`] if any vertex does not
    /// exist, [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, or [`HypergraphError::HyperedgeVerticesUnchanged`] if
    /// the new list is identical to the current one.
    pub fn update_hyperedge_vertices(
        &self,
        hyperedge_index: HyperedgeIndex,
        vertices: &[VertexIndex],
    ) -> Result<(), HypergraphError<V, HE>> {
        if vertices.is_empty() {
            return Err(HypergraphError::HyperedgeUpdateNoVertices(hyperedge_index));
        }

        for &v in vertices {
            self.load_vertex(v)?;
        }

        let entry = self.load_hyperedge(hyperedge_index)?;
        let prev_vertices = entry.0.clone();

        if vertices == prev_vertices.as_slice() {
            return Err(HypergraphError::HyperedgeVerticesUnchanged(hyperedge_index));
        }

        let mut prev_unique = prev_vertices.clone();
        prev_unique.sort_unstable();
        prev_unique.dedup();

        let mut new_unique = vertices.to_vec();
        new_unique.sort_unstable();
        new_unique.dedup();

        for &v in &new_unique {
            if !prev_unique.contains(&v) {
                self.add_vertex_ref(v, hyperedge_index)?;
            }
        }

        for &v in &prev_unique {
            if !new_unique.contains(&v) {
                self.remove_vertex_ref(v, hyperedge_index)?;
            }
        }

        self.store_hyperedge(hyperedge_index, vertices, entry.1)
    }

    /// Removes the hyperedge at `hyperedge_index`.
    ///
    /// Each vertex that was part of the hyperedge receives an O(1)
    /// back-reference delete in the `vertex_refs` keyspace; no vertex weight
    /// data is modified.
    ///
    /// # Errors
    ///
    /// Returns [`HypergraphError::HyperedgeIndexNotFound`] if `hyperedge_index`
    /// does not exist, or [`HypergraphError::StorageError`] on I/O failure.
    pub fn remove_hyperedge(
        &self,
        hyperedge_index: HyperedgeIndex,
    ) -> Result<(), HypergraphError<V, HE>> {
        let vertices = self.get_hyperedge_vertices(hyperedge_index)?;

        let mut unique_verts = vertices;
        unique_verts.sort_unstable();
        unique_verts.dedup();

        for v in unique_verts {
            self.remove_vertex_ref(v, hyperedge_index)?;
        }

        self.delete_hyperedge(hyperedge_index)?;
        self.hyperedges_count.fetch_sub(1, Ordering::Relaxed);
        self.flush_meta()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use crate::{
        HyperedgeIndex,
        VertexIndex,
        core::test_support::disk::{
            EP,
            WP,
            build_persistent,
        },
    };

    #[test]
    fn add_hyperedge_returns_index() {
        let dir = tempdir().unwrap();
        let (g, [v0, v1, _v2, _v3], _) = build_persistent(dir.path());
        let e = g.add_hyperedge(&[v0, v1], EP(99)).unwrap();
        assert!(e.0 >= 3); // already have 3 edges from build_persistent
    }

    #[test]
    fn add_hyperedge_empty_vertices_returns_error() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(g.add_hyperedge(&[], EP(1)).is_err());
    }

    #[test]
    fn add_hyperedge_missing_vertex_returns_error() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(g.add_hyperedge(&[VertexIndex(99)], EP(1)).is_err());
    }

    #[test]
    fn get_hyperedge_weight() {
        let dir = tempdir().unwrap();
        let (g, _, [e0, _e1, _e2]) = build_persistent(dir.path());
        assert_eq!(g.get_hyperedge_weight(e0).unwrap(), EP(1));
    }

    #[test]
    fn get_hyperedge_weight_not_found() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(g.get_hyperedge_weight(HyperedgeIndex(99)).is_err());
    }

    #[test]
    fn get_hyperedge_vertices() {
        let dir = tempdir().unwrap();
        let (g, [v0, v1, _v2, _v3], [e0, _e1, _e2]) = build_persistent(dir.path());
        assert_eq!(g.get_hyperedge_vertices(e0).unwrap(), vec![v0, v1]);
    }

    #[test]
    fn update_hyperedge_weight() {
        let dir = tempdir().unwrap();
        let (g, _, [e0, _e1, _e2]) = build_persistent(dir.path());
        g.update_hyperedge_weight(e0, EP(99)).unwrap();
        assert_eq!(g.get_hyperedge_weight(e0).unwrap(), EP(99));
    }

    #[test]
    fn update_hyperedge_weight_unchanged_returns_error() {
        let dir = tempdir().unwrap();
        let (g, _, [e0, _e1, _e2]) = build_persistent(dir.path());
        assert!(g.update_hyperedge_weight(e0, EP(1)).is_err());
    }

    #[test]
    fn update_hyperedge_vertices() {
        let dir = tempdir().unwrap();
        let (g, [v0, _v1, v2, _v3], [e0, _e1, _e2]) = build_persistent(dir.path());
        g.update_hyperedge_vertices(e0, &[v0, v2]).unwrap();
        assert_eq!(g.get_hyperedge_vertices(e0).unwrap(), vec![v0, v2]);
    }

    #[test]
    fn remove_hyperedge_decrements_count() {
        let dir = tempdir().unwrap();
        let (g, _, [e0, _e1, _e2]) = build_persistent(dir.path());
        g.remove_hyperedge(e0).unwrap();
        assert_eq!(g.count_hyperedges(), 2);
    }

    #[test]
    fn remove_hyperedge_not_found_returns_error() {
        let dir = tempdir().unwrap();
        let g = crate::core::disk::PersistentHypergraph::<WP, EP>::open(dir.path()).unwrap();
        assert!(g.remove_hyperedge(HyperedgeIndex(99)).is_err());
    }
}
