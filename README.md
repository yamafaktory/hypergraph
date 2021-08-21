![graph](hypergraph.svg)

---

![GitHub Workflow Status](https://img.shields.io/github/workflow/status/yamafaktory/hypergraph/ci?style=for-the-badge) ![Crates.io](https://img.shields.io/crates/v/hypergraph?style=for-the-badge) ![docs.rs](https://img.shields.io/docsrs/hypergraph?style=for-the-badge)

Hypergraph is a data structure library to generate **directed** [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph).

A hypergraph is a generalization of a graph in which a hyperedge can join any number of vertices.

## Features

This library enables you to:

- represent **non-simple** hypergraphs with two or more hyperedges - with different weights - containing the exact same set of vertices
- represent **self-loops** - i.e., hyperedges containing vertices directed to themselves one or more times
- represent **unaries** - i.e., hyperedges containing a unique vertex

Additional features:

- Safe Rust implementation
- Proper error handling
- Stable indexes assigned for each hyperedge and each vertex

## Installation

Add this to your `Cargo.toml` (replace _current_version_ with the [latest version of the library](https://crates.io/crates/hypergraph)):

```toml
[dependencies]
hypergraph = "curent_version"
```

## Usage

Please read the [documentation](https://docs.rs/hypergraph) to get started.
