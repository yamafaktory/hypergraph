mod common;
use common::{get_tracing_subscriber, prepare, Vertex};
use hypergraph::errors::HypergraphError;
use uuid::Uuid;

#[tokio::test(flavor = "multi_thread")]
async fn sequential_tests() -> Result<(), HypergraphError> {
    get_tracing_subscriber();

    // integration_add_get_delete_vertex(false).await?;
    //integration_add_get_delete_vertex(true).await?;
    batch(true).await?;

    Ok(())
}

async fn integration_add_get_delete_vertex(
    bypass_memory_cache: bool,
) -> Result<(), HypergraphError> {
    let (graph, clear, wait) = prepare(bypass_memory_cache).await?;

    // Add a vertex.
    let uuid = graph.create_vertex(Vertex {}).await?;

    // We should get two database files: `vertices` and `Uuid`.
    let mut files = wait(2).await?;
    assert!(files.any(|file| file.ends_with("vertices.db")));
    assert!(files.any(|file| {
        let file_stem = file.file_stem().unwrap();
        Uuid::try_parse(&file_stem.to_string_lossy()).is_ok()
    }));

    let vertex = graph.get_vertex(uuid).await?;
    assert_eq!(vertex.unwrap(), Vertex {});

    // Delete the vertex.
    graph.delete_vertex(uuid).await?;

    // Only the `vertices` database should exist.
    let mut files = wait(1).await?;
    assert!(files.any(|file| file.ends_with("vertices.db")));

    // Try to retrieve the vertex which should now be deleted.
    let vertex = graph.get_vertex(uuid).await?;
    assert_eq!(vertex, None);

    let _ = clear(()).await;

    Ok(())
}

async fn batch(bypass_memory_cache: bool) -> Result<(), HypergraphError> {
    let (graph, clear, wait) = prepare(bypass_memory_cache).await?;

    for n in 1..=1_000 {
        let uuid = graph.create_vertex(Vertex {}).await?;
    }
    //// We should get two database files: `vertices` and `Uuid`.
    //let mut files = wait(2).await?;
    //assert!(files.any(|file| file.ends_with("vertices.db")));
    //assert!(files.any(|file| {
    //    let file_stem = file.file_stem().unwrap();
    //    Uuid::try_parse(&file_stem.to_string_lossy()).is_ok()
    //}));
    //
    //let vertex = graph.get_vertex(uuid).await?;
    //assert_eq!(vertex.unwrap(), Vertex {});
    //
    //// Delete the vertex.
    //graph.delete_vertex(uuid).await?;
    //
    //// Only the `vertices` database should exist.
    //let mut files = wait(1).await?;
    //assert!(files.any(|file| file.ends_with("vertices.db")));
    //
    //// Try to retrieve the vertex which should now be deleted.
    //let vertex = graph.get_vertex(uuid).await?;
    //assert_eq!(vertex, None);

    let _ = clear(()).await;

    Ok(())
}
