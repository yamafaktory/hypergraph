mod common;

use std::time::Duration;

use common::{prepare, Vertex};
use hypergraph::errors::HypergraphError;
use tokio::time::sleep;

#[tokio::test(flavor = "multi_thread")]
async fn integration_main() -> Result<(), HypergraphError> {
    let (graph, clear) = prepare().await?;

    let id = graph.add_vertex(Vertex {}).await?;

    let vertex = graph.get_vertex(id).await?;

    dbg!(vertex);
    sleep(Duration::from_millis(5000)).await;
    // graph.add_hyperedge(Hyperedge {}, vec![id]).await?;

    let _ = clear().await;

    Ok(())
}
