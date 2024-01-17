mod common;

use common::{get_tracing_subscriber, prepare, Vertex};
use hypergraph::errors::HypergraphError;

#[tokio::test(flavor = "multi_thread")]
async fn sequential_tests() -> Result<(), HypergraphError> {
    get_tracing_subscriber();

    integration_add_get_vertex().await?;
    integration_add_get_vertex_no_cache().await?;

    Ok(())
}

async fn integration_add_get_vertex() -> Result<(), HypergraphError> {
    let (graph, clear) = prepare(false).await?;

    let id = graph.add_vertex(Vertex {}).await?;

    let vertex = graph.get_vertex(id).await?;

    let _ = clear().await;

    assert_eq!(vertex.unwrap(), Vertex {});

    Ok(())
}

async fn integration_add_get_vertex_no_cache() -> Result<(), HypergraphError> {
    let (graph, clear) = prepare(true).await?;

    let id = graph.add_vertex(Vertex {}).await?;

    let vertex = graph.get_vertex(id).await?;

    let _ = clear().await;

    assert_eq!(vertex.unwrap(), Vertex {});

    Ok(())
}
