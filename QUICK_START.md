# Quick Start Guide

Get Pyralog up and running in 5 minutes!

## Prerequisites

- Rust 1.70 or higher
- 4GB RAM minimum
- Linux, macOS, or Windows

## Installation

```bash
# Clone the repository
git clone https://github.com/pyralog/pyralog.git
cd dlog

# Build the project
cargo build --release

# The binary will be in target/release/dlog
```

## Single Node Setup

### 1. Start the Server

```bash
cargo run --release
```

The server will start on `localhost:9092` by default.

### 2. Use the Client Library

Create a new Rust project:

```bash
cargo new my-dlog-app
cd my-dlog-app
```

Add Pyralog to your `Cargo.toml`:

```toml
[dependencies]
dlog = { path = "../dlog" }
tokio = { version = "1.35", features = ["full"] }
bytes = "1.5"
```

Create a simple producer:

```rust
use dlog::prelude::*;
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<()> {
    // Connect to Pyralog
    let client = PyralogClient::new("localhost:9092");
    client.connect().await?;

    // Create a log
    let log_id = LogId::new("test", "my-first-log");
    client.create_log(log_id.clone(), 3, 1).await?;

    // Produce a message
    let offset = client.produce(
        log_id,
        Some(Bytes::from("key1")),
        Bytes::from("Hello, Pyralog!"),
    ).await?;

    println!("Message written at offset: {}", offset);

    Ok(())
}
```

Run it:

```bash
cargo run
```

## Three Node Cluster Setup

### Terminal 1 - Node 1

```bash
cargo run --release -- \
  --node-id 1 \
  --data-dir ./data1 \
  --listen 0.0.0.0:9092 \
  --internal 0.0.0.0:9093 \
  --cluster-nodes 1,2,3
```

### Terminal 2 - Node 2

```bash
cargo run --release -- \
  --node-id 2 \
  --data-dir ./data2 \
  --listen 0.0.0.0:9094 \
  --internal 0.0.0.0:9095 \
  --cluster-nodes 1,2,3
```

### Terminal 3 - Node 3

```bash
cargo run --release -- \
  --node-id 3 \
  --data-dir ./data3 \
  --listen 0.0.0.0:9096 \
  --internal 0.0.0.0:9097 \
  --cluster-nodes 1,2,3
```

## Docker Setup

Create a `docker-compose.yml`:

```yaml
version: '3.8'

services:
  dlog-1:
    build: .
    ports:
      - "9092:9092"
      - "9093:9093"
    environment:
      - NODE_ID=1
      - CLUSTER_NODES=1,2,3
    volumes:
      - ./data1:/data

  dlog-2:
    build: .
    ports:
      - "9094:9092"
      - "9095:9093"
    environment:
      - NODE_ID=2
      - CLUSTER_NODES=1,2,3
    volumes:
      - ./data2:/data

  dlog-3:
    build: .
    ports:
      - "9096:9092"
      - "9097:9093"
    environment:
      - NODE_ID=3
      - CLUSTER_NODES=1,2,3
    volumes:
      - ./data3:/data
```

Start the cluster:

```bash
docker-compose up
```

## Basic Operations

### Create a Log

```rust
let log_id = LogId::new("namespace", "log-name");
client.create_log(
    log_id,
    partitions: 8,      // Number of partitions
    replication: 3,     // Replication factor
).await?;
```

### Produce Messages

```rust
// Single message
client.produce(
    log_id.clone(),
    Some(Bytes::from("key")),
    Bytes::from("value"),
).await?;

// Batch
let records = vec![
    (Some(Bytes::from("key1")), Bytes::from("value1")),
    (Some(Bytes::from("key2")), Bytes::from("value2")),
];
client.produce_batch(log_id, records).await?;
```

### Consume Messages

```rust
let records = client.consume(
    log_id,
    PartitionId::new(0),
    LogOffset::new(0),
    max_records: 100,
).await?;

for record in records {
    println!("{:?}", record);
}
```

### List Logs

```rust
let logs = client.list_logs().await?;
for log in logs {
    println!("Log: {}", log);
}
```

## Configuration

Create a `config.json`:

```json
{
  "node": {
    "node_id": 1,
    "data_dir": "./data",
    "cluster_nodes": [1, 2, 3]
  },
  "network": {
    "listen_address": "0.0.0.0:9092",
    "internal_address": "0.0.0.0:9093",
    "max_connections": 10000,
    "request_timeout_ms": 30000
  },
  "storage": {
    "segment_config": {
      "max_size": 1073741824,
      "use_mmap": true,
      "sync_on_write": false
    },
    "cache_config": {
      "max_size": 16777216,
      "max_buffer_time_ms": 10,
      "enabled": true
    }
  },
  "replication": {
    "quorum": {
      "replication_factor": 3,
      "write_quorum": 2,
      "read_quorum": 2
    }
  }
}
```

Load the configuration:

```rust
let config = PyralogConfig::from_file("config.json")?;
let server = PyralogServer::new(config).await?;
```

## Troubleshooting

### Server Won't Start

Check that:
- Port 9092 is not already in use
- You have write permissions to the data directory
- Rust version is 1.70 or higher

### Can't Connect from Client

Verify:
- Server is running (`ps aux | grep dlog`)
- Firewall allows connections on port 9092
- Using correct address (localhost vs 0.0.0.0)

### Performance Issues

Try:
- Enabling write cache
- Using memory-mapped I/O
- Increasing batch sizes
- Adding more partitions

## Next Steps

- Read the [Architecture Guide](ARCHITECTURE.md) to understand internals
- Check out [Examples](EXAMPLES.md) for advanced patterns
- Review [Design Document](DESIGN.md) for design decisions
- Join our [Discord](https://discord.gg/pyralog) community

## Getting Help

- GitHub Issues: Report bugs and request features
- Discord: Real-time help from the community
- Documentation: https://docs.pyralog.io

Happy logging! 🚀

