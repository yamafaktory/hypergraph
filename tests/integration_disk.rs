//! Integration tests for the `persistence` feature and `PersistentHypergraph`.

#[cfg(feature = "persistence")]
mod disk_tests {
    use std::{
        fmt::{
            Display,
            Formatter,
            Result,
        },
        sync::Arc,
    };

    use hypergraph::{
        HyperedgeIndex,
        PersistentHypergraph,
        VertexIndex,
    };

    // ──────────────────────────────────────────────────────────────────────
    // Minimal vertex / hyperedge types
    // ──────────────────────────────────────────────────────────────────────

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
    struct V(u32);

    impl Display for V {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "V({})", self.0)
        }
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
    struct HE(u32);

    impl Display for HE {
        fn fmt(&self, f: &mut Formatter<'_>) -> Result {
            write!(f, "HE({})", self.0)
        }
    }

    impl From<HE> for usize {
        fn from(HE(cost): HE) -> Self {
            cost as usize
        }
    }

    fn open_temp() -> (PersistentHypergraph<V, HE>, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("temp dir");
        let g = PersistentHypergraph::open(dir.path()).expect("open disk graph");
        (g, dir)
    }

    // ──────────────────────────────────────────────────────────────────────
    // Tests
    // ──────────────────────────────────────────────────────────────────────

    #[test]
    fn empty_graph() {
        let (g, _dir) = open_temp();
        assert!(g.is_empty());
        assert_eq!(g.count_vertices(), 0);
        assert_eq!(g.count_hyperedges(), 0);
    }

    #[test]
    fn add_and_get_vertices() {
        let (g, _dir) = open_temp();

        let v0 = g.add_vertex(V(10)).expect("add v0");
        let v1 = g.add_vertex(V(20)).expect("add v1");
        let v2 = g.add_vertex(V(30)).expect("add v2");

        assert_eq!(v0, VertexIndex(0));
        assert_eq!(v1, VertexIndex(1));
        assert_eq!(v2, VertexIndex(2));

        assert_eq!(g.count_vertices(), 3);
        assert_eq!(g.get_vertex_weight(v0).expect("v0"), V(10));
        assert_eq!(g.get_vertex_weight(v1).expect("v1"), V(20));
        assert_eq!(g.get_vertex_weight(v2).expect("v2"), V(30));
    }

    #[test]
    fn update_vertex_weight() {
        let (g, _dir) = open_temp();
        let v0 = g.add_vertex(V(1)).expect("add");

        g.update_vertex_weight(v0, V(99)).expect("update");
        assert_eq!(g.get_vertex_weight(v0).expect("get"), V(99));
    }

    #[test]
    fn update_vertex_weight_unchanged_returns_error() {
        let (g, _dir) = open_temp();
        let v0 = g.add_vertex(V(42)).expect("add");
        assert!(g.update_vertex_weight(v0, V(42)).is_err());
    }

    #[test]
    fn add_and_get_hyperedges() {
        let (g, _dir) = open_temp();

        let v0 = g.add_vertex(V(0)).expect("v0");
        let v1 = g.add_vertex(V(1)).expect("v1");
        let v2 = g.add_vertex(V(2)).expect("v2");

        let he0 = g.add_hyperedge(&[v0, v1, v2], HE(5)).expect("he0");
        let he1 = g.add_hyperedge(&[v1, v2], HE(3)).expect("he1");

        assert_eq!(he0, HyperedgeIndex(0));
        assert_eq!(he1, HyperedgeIndex(1));

        assert_eq!(g.count_hyperedges(), 2);
        assert_eq!(g.get_hyperedge_weight(he0).expect("he0 weight"), HE(5));
        assert_eq!(
            g.get_hyperedge_vertices(he0).expect("he0 verts"),
            vec![v0, v1, v2]
        );
        assert_eq!(
            g.get_hyperedge_vertices(he1).expect("he1 verts"),
            vec![v1, v2]
        );
    }

    #[test]
    fn vertex_hyperedge_refs_are_maintained() {
        let (g, _dir) = open_temp();

        let v0 = g.add_vertex(V(0)).expect("v0");
        let v1 = g.add_vertex(V(1)).expect("v1");

        let he0 = g.add_hyperedge(&[v0, v1], HE(1)).expect("he0");
        let he1 = g.add_hyperedge(&[v0], HE(2)).expect("he1");

        let v0_hes = g.get_vertex_hyperedges(v0).expect("v0 hes");
        assert!(v0_hes.contains(&he0));
        assert!(v0_hes.contains(&he1));

        let v1_hes = g.get_vertex_hyperedges(v1).expect("v1 hes");
        assert!(v1_hes.contains(&he0));
        assert!(!v1_hes.contains(&he1));
    }

    #[test]
    fn remove_hyperedge_cleans_vertex_refs() {
        let (g, _dir) = open_temp();

        let v0 = g.add_vertex(V(0)).expect("v0");
        let v1 = g.add_vertex(V(1)).expect("v1");
        let he0 = g.add_hyperedge(&[v0, v1], HE(1)).expect("he0");

        g.remove_hyperedge(he0).expect("remove");

        assert_eq!(g.count_hyperedges(), 0);
        assert!(g.get_vertex_hyperedges(v0).expect("v0 hes").is_empty());
        assert!(g.get_vertex_hyperedges(v1).expect("v1 hes").is_empty());
    }

    #[test]
    fn remove_vertex_also_removes_solo_hyperedges() {
        let (g, _dir) = open_temp();

        let v0 = g.add_vertex(V(0)).expect("v0");
        let v1 = g.add_vertex(V(1)).expect("v1");
        let _he_solo = g.add_hyperedge(&[v0], HE(1)).expect("solo");
        let _he_shared = g.add_hyperedge(&[v0, v1], HE(2)).expect("shared");

        g.remove_vertex(v0).expect("remove v0");

        assert_eq!(g.count_vertices(), 1);
        // Solo HE must be gone; shared HE now has only v1.
        assert_eq!(g.count_hyperedges(), 1);
        let remaining = g.hyperedge_indices().expect("indices");
        let verts = g.get_hyperedge_vertices(remaining[0]).expect("verts");
        assert_eq!(verts, vec![v1]);
    }

    #[test]
    fn update_hyperedge_vertices() {
        let (g, _dir) = open_temp();

        let v0 = g.add_vertex(V(0)).expect("v0");
        let v1 = g.add_vertex(V(1)).expect("v1");
        let v2 = g.add_vertex(V(2)).expect("v2");
        let he0 = g.add_hyperedge(&[v0, v1], HE(1)).expect("he0");

        g.update_hyperedge_vertices(he0, &[v0, v2]).expect("update");
        assert_eq!(g.get_hyperedge_vertices(he0).expect("verts"), vec![v0, v2]);

        // v1 should no longer reference he0.
        assert!(g.get_vertex_hyperedges(v1).expect("v1 hes").is_empty());
        // v2 should now reference he0.
        assert!(g.get_vertex_hyperedges(v2).expect("v2 hes").contains(&he0));
    }

    #[test]
    fn update_hyperedge_weight() {
        let (g, _dir) = open_temp();
        let v0 = g.add_vertex(V(0)).expect("v0");
        let he0 = g.add_hyperedge(&[v0], HE(1)).expect("he0");

        g.update_hyperedge_weight(he0, HE(99)).expect("update");
        assert_eq!(g.get_hyperedge_weight(he0).expect("weight"), HE(99));
    }

    #[test]
    fn clear_empties_graph() {
        let (g, _dir) = open_temp();

        let v0 = g.add_vertex(V(0)).expect("v0");
        g.add_hyperedge(&[v0], HE(1)).expect("he");

        g.clear().expect("clear");

        assert_eq!(g.count_vertices(), 0);
        assert_eq!(g.count_hyperedges(), 0);
        assert!(g.vertex_indices().expect("vis").is_empty());
    }

    #[test]
    fn persistence_survives_reopen() {
        let dir = tempfile::tempdir().expect("temp dir");

        // Write data.
        {
            let g: PersistentHypergraph<V, HE> =
                PersistentHypergraph::open(dir.path()).expect("open");
            let v0 = g.add_vertex(V(7)).expect("v0");
            let v1 = g.add_vertex(V(8)).expect("v1");
            g.add_hyperedge(&[v0, v1], HE(42)).expect("he");
            g.persist().expect("persist");
        }

        // Re-open and verify data is present.
        {
            let g: PersistentHypergraph<V, HE> =
                PersistentHypergraph::open(dir.path()).expect("reopen");
            assert_eq!(g.count_vertices(), 2);
            assert_eq!(g.count_hyperedges(), 1);
            assert_eq!(g.get_vertex_weight(VertexIndex(0)).expect("v0"), V(7));
            assert_eq!(g.get_vertex_weight(VertexIndex(1)).expect("v1"), V(8));
            assert_eq!(
                g.get_hyperedge_weight(HyperedgeIndex(0)).expect("he0"),
                HE(42)
            );
        }
    }

    #[test]
    fn not_found_errors() {
        let (g, _dir) = open_temp();
        assert!(g.get_vertex_weight(VertexIndex(999)).is_err());
        assert!(g.get_hyperedge_weight(HyperedgeIndex(999)).is_err());
    }

    #[test]
    fn debug_impl() {
        let (g, _dir) = open_temp();
        let repr = format!("{g:?}");
        assert!(repr.contains("PersistentHypergraph"));
    }

    #[test]
    fn concurrent_vertex_adds_produce_unique_indices() {
        const THREADS: u32 = 8;
        const PER_THREAD: u32 = 25;

        let dir = tempfile::tempdir().expect("temp dir");
        let g = Arc::new(PersistentHypergraph::<V, HE>::open(dir.path()).expect("open"));

        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                let g = Arc::clone(&g);
                std::thread::spawn(move || {
                    (0..PER_THREAD)
                        .map(|i| g.add_vertex(V(t * PER_THREAD + i)).expect("add_vertex"))
                        .collect::<Vec<_>>()
                })
            })
            .collect();

        let mut all_indices: Vec<VertexIndex> = handles
            .into_iter()
            .flat_map(|h| h.join().expect("thread panicked"))
            .collect();

        let total = (THREADS * PER_THREAD) as usize;
        assert_eq!(all_indices.len(), total);
        all_indices.sort_unstable();
        all_indices.dedup();
        assert_eq!(
            all_indices.len(),
            total,
            "duplicate vertex indices detected"
        );
        assert_eq!(g.count_vertices(), total);
    }

    #[test]
    fn concurrent_hyperedge_adds_produce_unique_indices() {
        const THREADS: u32 = 8;
        const PER_THREAD: u32 = 25;

        let dir = tempfile::tempdir().expect("temp dir");
        let g = Arc::new(PersistentHypergraph::<V, HE>::open(dir.path()).expect("open"));

        let v0 = g.add_vertex(V(0)).expect("v0");
        let v1 = g.add_vertex(V(1)).expect("v1");

        let handles: Vec<_> = (0..THREADS)
            .map(|t| {
                let g = Arc::clone(&g);
                std::thread::spawn(move || {
                    (0..PER_THREAD)
                        .map(|i| {
                            g.add_hyperedge(&[v0, v1], HE(t * PER_THREAD + i))
                                .expect("add_hyperedge")
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();

        let mut all_indices: Vec<HyperedgeIndex> = handles
            .into_iter()
            .flat_map(|h| h.join().expect("thread panicked"))
            .collect();

        let total = (THREADS * PER_THREAD) as usize;
        assert_eq!(all_indices.len(), total);
        all_indices.sort_unstable();
        all_indices.dedup();
        assert_eq!(
            all_indices.len(),
            total,
            "duplicate hyperedge indices detected"
        );
        assert_eq!(g.count_hyperedges(), total);
    }
}
