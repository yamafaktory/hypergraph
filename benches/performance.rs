#![deny(unsafe_code, nonstandard_style)]

use std::fmt::{Display, Formatter, Result};

use criterion::{Criterion, criterion_group, criterion_main};
use hypergraph::{HyperedgeIndex, Hypergraph, VertexIndex};
use itertools::Itertools;

static HYPEREDGES: usize = 10_000;
static VERTICES: usize = 10_000;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Vertex(pub usize);

impl Vertex {
    pub fn new(rnd: usize) -> Self {
        Vertex(rnd)
    }
}

impl Display for Vertex {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct Hyperedge(pub usize);

impl Hyperedge {
    pub fn new(rnd: usize) -> Self {
        Hyperedge(rnd)
    }
}

impl Display for Hyperedge {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}

impl From<Hyperedge> for usize {
    fn from(Hyperedge(cost): Hyperedge) -> Self {
        cost
    }
}

fn criterion_benchmark(criterion: &mut Criterion) {
    let mut graph = Hypergraph::<Vertex, Hyperedge>::new();

    for i in 0..VERTICES {
        graph.add_vertex(Vertex::new(i)).unwrap();
    }

    for i in 0..HYPEREDGES {
        let vertices = (i..i + 1).map(VertexIndex).collect_vec();

        graph.add_hyperedge(vertices, Hyperedge::new(i)).unwrap();
    }

    criterion.bench_function("get-hyperedge-vertices", |bencher| {
        bencher.iter(|| graph.get_hyperedge_vertices(HyperedgeIndex((HYPEREDGES / 2) - 1)))
    });

    criterion.bench_function("get-hyperedge-connecting", |bencher| {
        bencher.iter(|| {
            graph.get_hyperedges_connecting(
                VertexIndex((VERTICES / 2) - 1),
                VertexIndex(VERTICES / 2),
            )
        })
    });

    criterion.bench_function("get-all-hyperedges-intersections", |bencher| {
        bencher.iter(|| {
            graph.get_hyperedges_intersections((0..HYPEREDGES).map(HyperedgeIndex).collect_vec())
        })
    });

    criterion.bench_function("dijkstra", |bencher| {
        bencher.iter(|| graph.get_dijkstra_connections(VertexIndex(0), VertexIndex(VERTICES)))
    });

    criterion.bench_function("dijkstra-reversed", |bencher| {
        bencher.iter(|| graph.get_dijkstra_connections(VertexIndex(VERTICES), VertexIndex(0)))
    });

    criterion.bench_function("remove-vertex", |bencher| {
        bencher.iter(|| graph.remove_vertex(VertexIndex(VERTICES)))
    });

    criterion.bench_function("remove-hyperedge", |bencher| {
        bencher.iter(|| graph.remove_hyperedge(HyperedgeIndex(HYPEREDGES)))
    });
}

criterion_group!(benches, criterion_benchmark);

criterion_main!(benches);
