use std::{path::Path, time::Duration};

use hypergraph::{errors::HypergraphError, Hypergraph};
use serde::{Deserialize, Serialize};
use tokio::{fs::remove_dir_all, time::sleep};

#[tokio::test(flavor = "multi_thread")]
async fn integration_main() -> Result<(), HypergraphError> {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

    #[derive(Clone, Copy, Debug, Deserialize, Serialize)]
    struct Vertex {}

    #[derive(Clone, Copy, Debug)]
    struct Hyperedge {}

    let path = Path::new("./test");

    let graph = Hypergraph::<Vertex, Hyperedge>::init(path).await?;

    let id = graph.add_vertex(Vertex {}).await?;

    let vertex = graph.get_vertex(id).await?;

    dbg!(vertex);
    sleep(Duration::from_millis(5000)).await;
    // graph.add_hyperedge(Hyperedge {}, vec![id]).await?;

    let _ = remove_dir_all(path).await;

    Ok(())
}
