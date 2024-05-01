mod common;
use common::{get_tracing_subscriber, prepare, Vertex};
use hypergraph::errors::HypergraphError;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn sequential_tests() -> Result<(), HypergraphError> {
    get_tracing_subscriber();

    // integration_add_get_delete_vertex(false).await?;
    integration_add_get_delete_vertex(true).await?;

    Ok(())
}

async fn integration_add_get_delete_vertex(
    bypass_memory_cache: bool,
) -> Result<(), HypergraphError> {
    let (graph, clear, wait) = prepare(bypass_memory_cache).await?;

    let uuid = graph.create_vertex(Vertex {}).await?;

    let mut files = wait(2).await?;

    assert!(files.any(|file| file.ends_with("vertices.db")));
    assert!(files.any(|file| {
        let file_stem = file.file_stem().unwrap();
        Uuid::try_parse(&file_stem.to_string_lossy()).is_ok()
    }));

    let vertex = graph.get_vertex(uuid).await?;

    assert_eq!(vertex.unwrap(), Vertex {});

    // graph.delete_vertex(uuid).await?;
    //
    // sleep(Duration::from_millis(1000)).await;
    //
    // let vertex = graph.get_vertex(uuid).await?;
    //
    // assert_eq!(vertex, None);

    let _ = clear(()).await;

    Ok(())
}
