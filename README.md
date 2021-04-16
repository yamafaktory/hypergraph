# Hypergraph ![ci](https://github.com/yamafaktory/hypergraph/workflows/ci/badge.svg)

Hypergraph is data structure library to create a directed hypergraph in which a hyperedge can join any number of vertices.

This library allows you to:

- represent **non-simple** hypergraphs with two or more hyperedges containing the same set of vertices with different weights
- represent **self-loops** —i.e., hyperedges containing vertices directed to themselves one or more times
- represent **unaries** —i.e., hyperedges containing a set with a unique vertex
- output a representation of a hypergraph using the [Graphviz dot format](https://graphviz.org/doc/info/lang.html)

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
hypergraph = "0.1.5"
```

## Documentation

Please read the [documentation](https://docs.rs/hypergraph).

## Note

This an early stage project. More functionalities will be added in the future.
