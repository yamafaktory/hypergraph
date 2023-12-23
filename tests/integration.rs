use hypergraph::Hypergraph;

#[tokio::test]
async fn integration_main() -> Result<(), ()> {
    tracing_subscriber::fmt::fmt()
        .pretty()
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_ids(true)
        .init();

    #[derive(Clone, Copy, Debug)]
    struct Vertex {}

    #[derive(Clone, Copy, Debug)]
    struct Hyperedge {}

    let graph = Hypergraph::<Vertex, Hyperedge>::init().await?;

    let t = graph.add_vertex(Vertex {}).await;

    Ok(())
}
