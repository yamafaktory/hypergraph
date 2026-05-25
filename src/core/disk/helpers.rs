use std::{
    fmt,
    sync::{
        Arc,
        atomic::Ordering,
    },
};

use serde::{
    Serialize,
    de::DeserializeOwned,
};

use super::types::{
    HyperedgeArc,
    PersistentHypergraph,
};
use crate::{
    HyperedgeIndex,
    HyperedgeTrait,
    VertexIndex,
    VertexTrait,
    errors::HypergraphError,
};

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

pub(super) const META_VERTEX_IDX: &[u8] = b"vi";
pub(super) const META_VERTEX_COUNT: &[u8] = b"vc";
pub(super) const META_HYPEREDGE_IDX: &[u8] = b"hi";
pub(super) const META_HYPEREDGE_COUNT: &[u8] = b"hc";

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
                let he_bytes: [u8; 8] =
                    k.get(8..16)
                        .and_then(|b| b.try_into().ok())
                        .ok_or_else(|| {
                            HypergraphError::StorageError("invalid vertex ref key length".into())
                        })?;
                #[allow(clippy::cast_possible_truncation)]
                Ok(HyperedgeIndex(u64::from_be_bytes(he_bytes) as usize))
            })
            .collect()
    }

    /// Deletes all back-reference keys for `v` from the `vertex_refs` keyspace.
    pub(super) fn delete_vertex_refs(&self, v: VertexIndex) -> Result<(), HypergraphError<V, HE>> {
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

        let entry: (Vec<VertexIndex>, HE) = postcard::from_bytes(&bytes).map_err(storage_err)?;

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
        batch.insert(
            &self.meta_ks,
            META_VERTEX_COUNT,
            vc.to_be_bytes().as_slice(),
        );
        batch.insert(
            &self.meta_ks,
            META_HYPEREDGE_IDX,
            hi.to_be_bytes().as_slice(),
        );
        batch.insert(
            &self.meta_ks,
            META_HYPEREDGE_COUNT,
            hc.to_be_bytes().as_slice(),
        );
        batch.commit().map_err(storage_err)
    }
}
