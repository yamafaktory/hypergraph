use ahash::RandomState;
use indexmap::{
    IndexMap,
    IndexSet,
};

/// Type alias to use `AHash` as a faster hasher for `IndexMap`.
pub(crate) type AIndexMap<K, V> = IndexMap<K, V, RandomState>;

/// Type alias to use `AHash` as a faster hasher for `IndexSet`.
pub(crate) type AIndexSet<T> = IndexSet<T, RandomState>;

/// Type alias for the `AHash` hasher factory.
pub(crate) type ARandomState = RandomState;
