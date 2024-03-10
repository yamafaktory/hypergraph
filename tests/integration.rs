mod common;

use common::{get_tracing_subscriber, prepare, Vertex};
use hypergraph::errors::HypergraphError;

#[tokio::test(flavor = "multi_thread")]
async fn sequential_tests() -> Result<(), HypergraphError> {
    get_tracing_subscriber();

    integration_add_get_delete_vertex().await?;
    integration_add_get_delete_vertex_no_cache().await?;

    Ok(())
}

async fn integration_add_get_delete_vertex() -> Result<(), HypergraphError> {
    let (graph, clear) = prepare(false).await?;

    let uuid = graph.create_vertex(Vertex {}).await?;

    let vertex = graph.get_vertex(uuid).await?;

    assert_eq!(vertex.unwrap(), Vertex {});

    graph.delete_vertex(uuid).await?;

    let vertex = graph.get_vertex(uuid).await?;

    assert_eq!(vertex, None);

    let _ = clear().await;

    Ok(())
}

async fn integration_add_get_delete_vertex_no_cache() -> Result<(), HypergraphError> {
    let (graph, clear) = prepare(true).await?;

    let uuid = graph.create_vertex(Vertex {}).await?;

    let vertex = graph.get_vertex(uuid).await?;

    assert_eq!(vertex.unwrap(), Vertex {});

    graph.delete_vertex(uuid).await?;

    let vertex = graph.get_vertex(uuid).await?;

    assert_eq!(vertex, None);

    let _ = clear().await;

    Ok(())
}
