use std::fmt::Debug;

use uuid::Uuid;

use crate::entities::{EntityKind, EntityRelation, EntityWeight};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReadOp(pub(crate) Uuid, pub(crate) EntityKind);

impl ReadOp {
    pub(crate) fn get_uuid(&self) -> Uuid {
        self.0
    }

    pub(crate) fn get_entity_kind(&self) -> EntityKind {
        self.1
    }
}

#[derive(Clone, Debug)]
pub(crate) enum WriteOp<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    Create(Uuid, EntityWeight<V, HE>),
    Delete(Uuid, EntityKind),
    UpdateRelation(Uuid, EntityRelation),
    UpdateWeight(Uuid, EntityWeight<V, HE>),
}
