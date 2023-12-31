use std::path::Path;

use hypergraph::{errors::HypergraphError, Hypergraph};
use serde::Serialize;

#[tokio::test(flavor = "multi_thread")]
async fn integration_main() -> Result<(), HypergraphError> {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

    #[derive(Clone, Copy, Debug, Serialize)]
    struct Vertex {}

    #[derive(Clone, Copy, Debug)]
    struct Hyperedge {}

    let path = Path::new("./test");

    let graph = Hypergraph::<Vertex, Hyperedge>::init(path).await?;

    let id = graph.add_vertex(Vertex {}).await?;

    graph.add_hyperedge(Hyperedge {}, vec![id]).await?;

    Ok(())
}
