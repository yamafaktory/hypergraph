#![deny(unsafe_code, nonstandard_style)]
#![allow(missing_docs)]

use std::fmt::{
    Display,
    Formatter,
    Result,
};

use criterion::{
    BatchSize,
    Criterion,
    criterion_group,
    criterion_main,
};
use hypergraph::{
    HyperedgeIndex,
    Hypergraph,
    VertexIndex,
};
use itertools::Itertools;

const VERTICES: usize = 1_000;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct Vertex(usize);

impl Display for Vertex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
struct Hyperedge(usize);

impl Display for Hyperedge {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.0)
    }
}

impl From<Hyperedge> for usize {
    fn from(Hyperedge(cost): Hyperedge) -> Self {
        cost
    }
}

// Builds a linear-chain hypergraph: vertex 0 → 1 → 2 → … → VERTICES-1,
/// each consecutive pair joined by a hyperedge whose cost equals its index + 1.
fn build_graph() -> Hypergraph<Vertex, Hyperedge> {
    let mut graph = Hypergraph::<Vertex, Hyperedge>::new();

    for i in 0..VERTICES {
        graph.add_vertex(Vertex(i)).unwrap();
    }

    for i in 0..VERTICES - 1 {
        graph
            .add_hyperedge(vec![VertexIndex(i), VertexIndex(i + 1)], Hyperedge(i + 1))
            .unwrap();
    }

    graph
}

fn criterion_benchmark(criterion: &mut Criterion) {
    let graph = build_graph();

    criterion.bench_function("get-hyperedge-vertices", |bencher| {
        bencher.iter(|| graph.get_hyperedge_vertices(HyperedgeIndex(VERTICES / 2)))
    });

    criterion.bench_function("get-hyperedges-connecting", |bencher| {
        bencher.iter(|| {
            graph
                .get_hyperedges_connecting(VertexIndex(VERTICES / 2), VertexIndex(VERTICES / 2 + 1))
        })
    });

    criterion.bench_function("get-hyperedges-intersections", |bencher| {
        bencher
            .iter(|| graph.get_hyperedges_intersections((0..10).map(HyperedgeIndex).collect_vec()))
    });

    criterion.bench_function("dijkstra", |bencher| {
        bencher.iter(|| graph.get_dijkstra_connections(VertexIndex(0), VertexIndex(VERTICES - 1)))
    });

    criterion.bench_function("remove-vertex", |bencher| {
        bencher.iter_batched(
            build_graph,
            |mut g| g.remove_vertex(VertexIndex(VERTICES - 1)),
            BatchSize::LargeInput,
        )
    });

    criterion.bench_function("remove-hyperedge", |bencher| {
        bencher.iter_batched(
            build_graph,
            |mut g| g.remove_hyperedge(HyperedgeIndex(VERTICES - 2)),
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
