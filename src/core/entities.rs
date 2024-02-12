use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::collections::HashSet;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Vertex<V> {
    pub(crate) hyperedges: HashSet<Uuid>,
    pub(crate) weight: V,
}

impl<V> Vertex<V> {
    pub(crate) fn new(weight: V) -> Self {
        Self {
            hyperedges: HashSet::default(),
            weight,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Hyperedge<HE> {
    pub(crate) vertices: Vec<Uuid>,
    pub(crate) weight: HE,
}

impl<HE> Hyperedge<HE> {
    pub(crate) fn new(weight: HE) -> Self {
        Self {
            vertices: vec![],
            weight,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum EntityKind {
    Hyperedge,
    Vertex,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) enum Entity<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    Hyperedge(Hyperedge<HE>),
    Vertex(Vertex<V>),
}

#[derive(Clone, Debug)]
pub(crate) enum EntityRelation {
    Hyperedge(Vec<Uuid>),
    Vertex(HashSet<Uuid>),
}

#[derive(Clone, Debug)]
pub(crate) enum EntityWeight<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    Hyperedge(HE),
    Vertex(V),
}

impl<V, HE> From<&EntityWeight<V, HE>> for EntityKind
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn from(entity_weight: &EntityWeight<V, HE>) -> Self {
        match entity_weight {
            EntityWeight::Hyperedge(_) => EntityKind::Hyperedge,
            EntityWeight::Vertex(_) => EntityKind::Vertex,
        }
    }
}
