use std::collections::{HashMap as DefaultHashMap, HashSet as DefaultHashSet};

use ahash::RandomState;

pub(crate) type HashSet<K> = DefaultHashSet<K, RandomState>;
pub(crate) type HashMap<K, V> = DefaultHashMap<K, V, RandomState>;
