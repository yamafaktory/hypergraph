![graph](hypergraph.svg)

---

[<img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/yamafaktory/hypergraph/ci.yml?branch=main&logo=github&style=flat-square">](https://github.com/yamafaktory/hypergraph/actions/workflows/ci.yml) [<img alt="Crates.io" src="https://img.shields.io/crates/v/hypergraph?style=flat-square">](https://crates.io/crates/hypergraph/versions?sort=semver) [<img alt="docs.rs" src="https://img.shields.io/docsrs/hypergraph?style=flat-square">](https://docs.rs/hypergraph)

Hypergraph is a data structure library to generate **directed** [hypergraphs](https://en.wikipedia.org/wiki/Hypergraph).

A hypergraph is a generalization of a graph in which a hyperedge can join any number of vertices.

## 📣 Goal

This library aims at providing the necessary methods for modeling complex, multiway (non-pairwise) relational data found in complex networks.
One of the main advantages of using a hypergraph model over a graph one is to provide a more flexible and natural framework to represent entities and their relationships (e.g. Alice uses some social network, shares some data to Bob, who shares it to Carol, etc).

## 🎁 Features

This library enables you to represent:

- **non-simple** hypergraphs with two or more hyperedges - with different weights - containing the exact same set of vertices
- **self-loops** - i.e., hyperedges containing vertices directed to themselves one or more times
- **unaries** - i.e., hyperedges containing a unique vertex

## ⚗️ Implementation

- 100% safe Rust
- Proper error handling
- Stable indexes assigned for each hyperedge and each vertex
- Parallelism (with Rayon)

## 🛠️ Installation

Add this to your `Cargo.toml` (replace _current_version_ with the [latest version of the library](https://crates.io/crates/hypergraph)):

```toml
[dependencies]
hypergraph = "current_version"
```

## ⚡️ Usage

Please read the [documentation](https://docs.rs/hypergraph) to get started.
