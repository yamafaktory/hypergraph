use std::fmt;

use crate::{
    HyperedgeIndex,
    Hypergraph,
    VertexIndex,
};

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct W(pub(crate) u8);

impl fmt::Display for W {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "W{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub(crate) struct E(pub(crate) usize);

impl fmt::Display for E {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{}", self.0)
    }
}

impl From<E> for usize {
    fn from(e: E) -> usize {
        e.0
    }
}

/// Standard acyclic test graph:
/// ```text
/// v0 -[e0,cost=1]-> v1 -[e1,cost=2]-> v2
///                   |
///                   [e2,cost=3]
///                   |
///                   v3
/// ```
pub(crate) fn build() -> (Hypergraph<W, E>, [VertexIndex; 4], [HyperedgeIndex; 3]) {
    let mut g: Hypergraph<W, E> = Hypergraph::new();
    let v0 = g.add_vertex(W(0)).unwrap();
    let v1 = g.add_vertex(W(1)).unwrap();
    let v2 = g.add_vertex(W(2)).unwrap();
    let v3 = g.add_vertex(W(3)).unwrap();
    let e0 = g.add_hyperedge(vec![v0, v1], E(1)).unwrap();
    let e1 = g.add_hyperedge(vec![v1, v2], E(2)).unwrap();
    let e2 = g.add_hyperedge(vec![v1, v3], E(3)).unwrap();
    (g, [v0, v1, v2, v3], [e0, e1, e2])
}

#[cfg(feature = "persistence")]
pub(crate) mod disk {
    use std::path::Path;

    use serde::{
        Deserialize,
        Serialize,
    };

    use crate::{
        HyperedgeIndex,
        VertexIndex,
        core::disk::PersistentHypergraph,
    };

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct WP(pub(crate) u8);

    impl std::fmt::Display for WP {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "WP{}", self.0)
        }
    }

    #[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
    pub(crate) struct EP(pub(crate) usize);

    impl std::fmt::Display for EP {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "EP{}", self.0)
        }
    }

    impl From<EP> for usize {
        fn from(e: EP) -> usize {
            e.0
        }
    }

    pub(crate) fn build_persistent(
        dir: &Path,
    ) -> (
        PersistentHypergraph<WP, EP>,
        [VertexIndex; 4],
        [HyperedgeIndex; 3],
    ) {
        let g = PersistentHypergraph::<WP, EP>::open(dir).unwrap();
        let v0 = g.add_vertex(WP(0)).unwrap();
        let v1 = g.add_vertex(WP(1)).unwrap();
        let v2 = g.add_vertex(WP(2)).unwrap();
        let v3 = g.add_vertex(WP(3)).unwrap();
        let e0 = g.add_hyperedge(&[v0, v1], EP(1)).unwrap();
        let e1 = g.add_hyperedge(&[v1, v2], EP(2)).unwrap();
        let e2 = g.add_hyperedge(&[v1, v3], EP(3)).unwrap();
        (g, [v0, v1, v2, v3], [e0, e1, e2])
    }
}
