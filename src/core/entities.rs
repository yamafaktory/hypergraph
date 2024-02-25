use std::fmt::{self, Debug};

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

impl fmt::Display for EntityKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntityKind::Hyperedge => write!(f, "Hyperedge"),
            EntityKind::Vertex => write!(f, "Vertex"),
        }
    }
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

impl From<&EntityRelation> for EntityKind {
    fn from(entity_relation: &EntityRelation) -> Self {
        match entity_relation {
            EntityRelation::Hyperedge(_) => EntityKind::Hyperedge,
            EntityRelation::Vertex(_) => EntityKind::Vertex,
        }
    }
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

impl<V, HE> From<&Entity<V, HE>> for EntityWeight<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn from(entity: &Entity<V, HE>) -> Self {
        match entity {
            Entity::Hyperedge(hyperedge) => EntityWeight::Hyperedge(hyperedge.weight.to_owned()),
            Entity::Vertex(vertex) => EntityWeight::Vertex(vertex.weight.to_owned()),
        }
    }
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
