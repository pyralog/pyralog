# Pyralog Examples

This document provides practical examples of using Pyralog.

## Table of Contents

1. [Basic Usage](#basic-usage)
2. [Configuration](#configuration)
3. [Advanced Patterns](#advanced-patterns)
4. [Performance Tuning](#performance-tuning)

## Basic Usage

### Creating a Log

```rust
use pyralog::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let client = PyralogClient::new("localhost:9092");
    client.connect().await?;

    // Create a log with 8 partitions and replication factor of 3
    let log_id = LogId::new("events", "user-actions");
    client.create_log(log_id, 8, 3).await?;

    Ok(())
}
```

### Producing Records

```rust
use bytes::Bytes;

// Single record
let key = Bytes::from("user-123");
let value = Bytes::from(r#"{"action":"login","timestamp":1234567890}"#);

let offset = client.produce(log_id.clone(), Some(key), value).await?;
println!("Written at offset: {}", offset);

// Batch produce
let records = vec![
    (Some(Bytes::from("user-123")), Bytes::from("event1")),
    (Some(Bytes::from("user-456")), Bytes::from("event2")),
    (Some(Bytes::from("user-789")), Bytes::from("event3")),
];

let base_offset = client.produce_batch(log_id, records).await?;
```

### Consuming Records

```rust
// Consume from specific offset
let partition = PartitionId::new(0);
let start_offset = LogOffset::new(0);

let records = client.consume(
    log_id.clone(),
    partition,
    start_offset,
    100, // max records
).await?;

for record in records {
    println!(
        "Offset: {}, Key: {:?}, Value: {:?}",
        record.offset,
        record.key,
        record.value
    );
}
```

### Consumer Group

```rust
use pyralog::prelude::*;

struct ConsumerGroup {
    client: PyralogClient,
    log_id: LogId,
    group_id: String,
    partition: PartitionId,
    current_offset: LogOffset,
}

impl ConsumerGroup {
    async fn poll(&mut self) -> Result<Vec<Record>> {
        let records = self.client.consume(
            self.log_id.clone(),
            self.partition,
            self.current_offset,
            100,
        ).await?;

        if let Some(last) = records.last() {
            self.current_offset = last.offset.next();
        }

        Ok(records)
    }
}
```

## Configuration

### Server Configuration

```rust
use pyralog::{PyralogServer, PyralogConfig};
use std::path::PathBuf;

let mut config = PyralogConfig::default();

// Node configuration
config.node.node_id = 1;
config.node.data_dir = PathBuf::from("/var/lib/pyralog");
config.node.cluster_nodes = vec![1, 2, 3];

// Network configuration
config.network.listen_address = "0.0.0.0:9092".to_string();
config.network.internal_address = "0.0.0.0:9093".to_string();
config.network.max_connections = 10000;

// Storage configuration
config.storage.segment_config.max_size = 2 * 1024 * 1024 * 1024; // 2GB
config.storage.cache_config.max_size = 64 * 1024 * 1024; // 64MB
config.storage.cache_config.enabled = true;

// Replication configuration
config.replication.quorum.replication_factor = 3;
config.replication.quorum.write_quorum = 2;
config.replication.quorum.read_quorum = 2;

let server = PyralogServer::new(config).await?;
```

### From Configuration File

```toml
# pyralog.toml

[node]
node_id = 1
data_dir = "/var/lib/pyralog"
cluster_nodes = [1, 2, 3]

[network]
listen_address = "0.0.0.0:9092"
internal_address = "0.0.0.0:9093"
max_connections = 10000
request_timeout_ms = 30000

[storage.segment_config]
max_size = 2147483648  # 2GB
use_mmap = true
sync_on_write = false

[storage.cache_config]
max_size = 67108864  # 64MB
max_buffer_time_ms = 10
enabled = true

[replication.quorum]
replication_factor = 3
write_quorum = 2
read_quorum = 2
```

```rust
let config = PyralogConfig::from_file("pyralog.toml")?;
let server = PyralogServer::new(config).await?;
```

## Advanced Patterns

### Custom Partitioning

```rust
use pyralog::protocol::{Partitioner, PartitionStrategy};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// Key-based partitioning
let partitioner = Partitioner::new(PartitionStrategy::KeyHash, 8);

let key = Bytes::from("user-123");
let value = Bytes::from("data");
let partition = partitioner.partition(Some(&key), &value);

println!("Record will go to partition: {}", partition);
```

### Headers

```rust
use pyralog_core::RecordHeader;

let mut record = Record::new(
    Some(Bytes::from("key")),
    Bytes::from("value"),
);

// Add headers
let headers = vec![
    RecordHeader::new("source".to_string(), Bytes::from("web-api")),
    RecordHeader::new("version".to_string(), Bytes::from("1.0")),
    RecordHeader::new("timestamp".to_string(), Bytes::from("2025-01-01")),
];

record = record.with_headers(headers);
```

### Batching for Performance

```rust
use std::time::Duration;
use tokio::time::interval;

async fn batched_producer(
    client: &PyralogClient,
    log_id: LogId,
) -> Result<()> {
    let mut buffer = Vec::new();
    let mut ticker = interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            // Collect records
            record = receive_record() => {
                buffer.push(record);
                
                if buffer.len() >= 1000 {
                    client.produce_batch(log_id.clone(), buffer.drain(..).collect()).await?;
                }
            }
            
            // Periodic flush
            _ = ticker.tick() => {
                if !buffer.is_empty() {
                    client.produce_batch(log_id.clone(), buffer.drain(..).collect()).await?;
                }
            }
        }
    }
}
```

### Parallel Consumers

```rust
use tokio::task::JoinSet;

async fn parallel_consume(
    client: PyralogClient,
    log_id: LogId,
    partition_count: u32,
) -> Result<()> {
    let mut tasks = JoinSet::new();

    for partition_id in 0..partition_count {
        let client = client.clone();
        let log_id = log_id.clone();
        
        tasks.spawn(async move {
            let partition = PartitionId::new(partition_id);
            let mut offset = LogOffset::ZERO;

            loop {
                let records = client.consume(
                    log_id.clone(),
                    partition,
                    offset,
                    100,
                ).await?;

                if records.is_empty() {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }

                // Process records
                for record in &records {
                    process_record(record).await?;
                }

                offset = records.last().unwrap().offset.next();
            }
        });
    }

    // Wait for all consumers
    while let Some(result) = tasks.join_next().await {
        result??;
    }

    Ok(())
}
```

## Performance Tuning

### High Throughput Configuration

```rust
let mut config = PyralogConfig::default();

// Large segments
config.storage.segment_config.max_size = 4 * 1024 * 1024 * 1024; // 4GB

// Large write cache
config.storage.cache_config.max_size = 128 * 1024 * 1024; // 128MB
config.storage.cache_config.max_buffer_time = Duration::from_millis(50);

// Relaxed quorums
config.replication.quorum.write_quorum = 1; // Async replication
```

### Low Latency Configuration

```rust
let mut config = PyralogConfig::default();

// Small write cache
config.storage.cache_config.max_size = 8 * 1024 * 1024; // 8MB
config.storage.cache_config.max_buffer_time = Duration::from_millis(1);

// Sync writes
config.storage.segment_config.sync_on_write = true;

// Strong quorums
config.replication.quorum.write_quorum = 3; // All replicas
```

### Memory-Mapped I/O

```rust
// Enable mmap for better read performance
config.storage.segment_config.use_mmap = true;

// Works best with:
// - Large sequential reads
// - Random access patterns
// - Read-heavy workloads
```

### Monitoring Performance

```rust
use std::time::Instant;

async fn measure_latency(client: &PyralogClient, log_id: LogId) {
    let start = Instant::now();
    
    client.produce(
        log_id,
        None,
        Bytes::from("test"),
    ).await.unwrap();
    
    let latency = start.elapsed();
    println!("Write latency: {:?}", latency);
}
```

### Load Testing

```rust
use tokio::time::{interval, Duration};

async fn load_test(
    client: PyralogClient,
    log_id: LogId,
    records_per_sec: usize,
) -> Result<()> {
    let interval_duration = Duration::from_secs(1);
    let mut ticker = interval(interval_duration);
    let batch_size = 100;
    let batches_per_sec = records_per_sec / batch_size;

    loop {
        ticker.tick().await;
        
        for _ in 0..batches_per_sec {
            let records: Vec<_> = (0..batch_size)
                .map(|i| {
                    let key = Bytes::from(format!("key-{}", i));
                    let value = Bytes::from(format!("value-{}", i));
                    (Some(key), value)
                })
                .collect();

            client.produce_batch(log_id.clone(), records).await?;
        }
        
        println!("Produced {} records", records_per_sec);
    }
}
```

## Conclusion

These examples demonstrate the flexibility and power of Pyralog. For more information, see the [Architecture Documentation](ARCHITECTURE.md) and [API Documentation](https://docs.rs/pyralog).

