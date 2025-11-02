use pyralog::{PyralogServer, PyralogConfig};
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_level(true)
        .init();

    // Load configuration
    let config = PyralogConfig::default();

    tracing::info!("Starting Pyralog server with node_id={}", config.node.node_id);

    // Create and start server
    let server = Arc::new(PyralogServer::new(config).await?);
    server.start().await?;

    Ok(())
}

