//! Integration tests for the optional `serde` feature.

#![deny(unsafe_code, nonstandard_style)]
#![allow(missing_docs)]

#[cfg(feature = "serde")]
mod serde_tests {
    use std::fmt::{
        Display,
        Formatter,
        Result,
    };

    use hypergraph::{
        HyperedgeIndex,
        Hypergraph,
        VertexIndex,
    };

    // Owned types that can derive serde traits.

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
    struct V(u32);

    impl Display for V {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self.0)
        }
    }

    #[derive(Copy, Clone, Debug, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
    struct HE(u32);

    impl Display for HE {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "{}", self.0)
        }
    }

    impl From<HE> for usize {
        fn from(HE(cost): HE) -> Self {
            cost as usize
        }
    }

    fn build() -> (Hypergraph<V, HE>, VertexIndex, VertexIndex, HyperedgeIndex) {
        let mut g = Hypergraph::<V, HE>::new();
        let a = g.add_vertex(V(0)).unwrap();
        let b = g.add_vertex(V(1)).unwrap();
        let ab = g.add_hyperedge(vec![a, b], HE(10)).unwrap();
        (g, a, b, ab)
    }

    #[test]
    fn round_trip_json() {
        let (g, a, b, ab) = build();

        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Hypergraph<V, HE> = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(g2.count_vertices(), 2);
        assert_eq!(g2.count_hyperedges(), 1);
        assert_eq!(g2.get_vertex_weight(a).unwrap(), &V(0));
        assert_eq!(g2.get_vertex_weight(b).unwrap(), &V(1));
        assert_eq!(g2.get_hyperedge_weight(ab).unwrap(), &HE(10));
        assert_eq!(g2.get_hyperedge_vertices(ab).unwrap(), vec![a, b]);
    }

    #[test]
    fn empty_graph_round_trip() {
        let g = Hypergraph::<V, HE>::new();
        let json = serde_json::to_string(&g).expect("serialize");
        let g2: Hypergraph<V, HE> = serde_json::from_str(&json).expect("deserialize");
        assert!(g2.is_empty());
    }

    #[test]
    fn indexes_serialize() {
        let idx = VertexIndex(42);
        let json = serde_json::to_string(&idx).unwrap();
        let back: VertexIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(back, idx);

        let hidx = HyperedgeIndex(7);
        let json = serde_json::to_string(&hidx).unwrap();
        let back: HyperedgeIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(back, hidx);
    }
}
