use std::fmt::{self, Debug};

use uuid::Uuid;

use crate::entities::{EntityKind, EntityRelation, EntityWeight};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ReadOp(pub(crate) Uuid, pub(crate) EntityKind);

impl ReadOp {
    pub(crate) fn get_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for ReadOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.1, self.0)
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

impl<V, HE> fmt::Display for WriteOp<V, HE>
where
    V: Clone + Debug + Send + Sync,
    HE: Clone + Debug + Send + Sync,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WriteOp::Create(uuid, _) => write!(f, "Create {}", uuid),
            WriteOp::Delete(uuid, _) => write!(f, "Delete {}", uuid),
            WriteOp::UpdateRelation(uuid, _) => write!(f, "Update relation {}", uuid),
            WriteOp::UpdateWeight(uuid, _) => write!(f, "Update weight {}", uuid),
        }
    }
}
