# Pyralog Performance Guide

Comprehensive guide to understanding and optimizing Pyralog performance.

## Table of Contents

1. [Performance Characteristics](#performance-characteristics)
2. [Benchmarking](#benchmarking)
3. [Optimization Techniques](#optimization-techniques)
4. [Configuration Tuning](#configuration-tuning)
5. [Monitoring Performance](#monitoring-performance)
6. [Common Bottlenecks](#common-bottlenecks)

## Performance Characteristics

### Expected Performance

On modern hardware (AWS c5.2xlarge equivalent):

| Operation | Throughput | Latency (p50) | Latency (p99) |
|-----------|------------|---------------|---------------|
| Write (single) | 100K ops/sec | 0.3ms | 0.8ms |
| Write (batch 100) | 1M+ ops/sec | 0.8ms | 1.2ms |
| Write (batch 1000) | 5M+ ops/sec | 2ms | 5ms |
| Read (single) | 200K ops/sec | 0.1ms | 0.3ms |
| Read (sequential) | 10M+ ops/sec | 0.5ms | 1ms |

### Factors Affecting Performance

**Hardware**:
- CPU: Single-thread performance critical
- Memory: More is better (for caching)
- Disk: NVMe >> SAS SSD >> SATA SSD >> HDD
- Network: Bandwidth and latency matter

**Configuration**:
- Write cache size and timeout
- Segment size
- Memory mapping
- Quorum settings
- Batch size

**Workload**:
- Record size
- Key distribution
- Read/write ratio
- Sequential vs random access

## Benchmarking

### Built-in Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench write_latency

# With profiling
cargo bench --profile-time 10
```

### Custom Load Test

```rust
use dlog::prelude::*;
use std::time::Instant;

async fn benchmark_writes(
    client: &PyralogClient,
    log_id: LogId,
    num_records: usize,
    batch_size: usize,
) {
    let start = Instant::now();
    
    for batch_num in 0..(num_records / batch_size) {
        let records: Vec<_> = (0..batch_size)
            .map(|i| {
                let key = format!("key-{}-{}", batch_num, i);
                let value = format!("value-{}", i);
                (
                    Some(Bytes::from(key)),
                    Bytes::from(value),
                )
            })
            .collect();
        
        client.produce_batch(log_id.clone(), records).await.unwrap();
    }
    
    let elapsed = start.elapsed();
    let ops_per_sec = num_records as f64 / elapsed.as_secs_f64();
    
    println!("Throughput: {:.0} ops/sec", ops_per_sec);
    println!("Avg latency: {:.2}ms", elapsed.as_millis() as f64 / num_records as f64);
}
```

### Measuring Latency Distribution

```rust
use hdrhistogram::Histogram;

let mut histogram = Histogram::<u64>::new(3).unwrap();

for _ in 0..10000 {
    let start = Instant::now();
    client.produce(log_id.clone(), key, value).await?;
    let latency = start.elapsed().as_micros() as u64;
    histogram.record(latency).unwrap();
}

println!("p50: {}μs", histogram.value_at_quantile(0.50));
println!("p99: {}μs", histogram.value_at_quantile(0.99));
println!("p999: {}μs", histogram.value_at_quantile(0.999));
println!("max: {}μs", histogram.max());
```

### Profiling

#### CPU Profiling

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin dlog

# Open flamegraph.svg in browser
```

#### Memory Profiling

```bash
# Install heaptrack
sudo apt-get install heaptrack

# Profile Pyralog
heaptrack target/release/dlog

# Analyze results
heaptrack_gui heaptrack.dlog.<pid>.gz
```

#### Perf (Linux)

```bash
# Record
sudo perf record -g target/release/dlog

# Report
sudo perf report
```

## Optimization Techniques

### 1. Write Caching

**Concept**: Buffer writes in memory before flushing to disk

**Impact**: 
- ↑ Throughput: 10-100x
- ↓ Latency: 5-10x
- ↓ Durability: Window of data loss

**Configuration**:
```json
{
  "storage": {
    "cache_config": {
      "max_size": 67108864,      // 64MB
      "max_buffer_time_ms": 10,  // 10ms
      "enabled": true
    }
  }
}
```

**Tuning**:
- Larger cache = Higher throughput, higher latency variance
- Smaller timeout = Lower latency, more frequent I/O
- Balance based on workload

### 2. Batching

**Concept**: Group multiple operations together

**Impact**:
- ↑ Throughput: 10-50x
- ↑ Latency: Slightly higher for individual records
- ↓ CPU overhead: Amortized across batch

**Client-side batching**:
```rust
let mut buffer = Vec::new();
let mut last_flush = Instant::now();

loop {
    // Collect records
    buffer.push(record);
    
    // Flush conditions
    if buffer.len() >= 1000 || last_flush.elapsed() > Duration::from_millis(100) {
        client.produce_batch(log_id.clone(), buffer.drain(..).collect()).await?;
        last_flush = Instant::now();
    }
}
```

**Optimal batch sizes**:
- Small records (< 1KB): 500-1000 per batch
- Medium records (1-10KB): 100-500 per batch
- Large records (> 10KB): 10-100 per batch

### 3. Memory Mapping

**Concept**: Map files directly into process memory

**Impact**:
- ↑ Read performance: 2-3x
- ↓ CPU usage: OS handles I/O
- ↑ Memory usage: OS page cache

**Configuration**:
```json
{
  "storage": {
    "segment_config": {
      "use_mmap": true
    }
  }
}
```

**When to use**:
- ✅ Read-heavy workloads
- ✅ Random access patterns
- ✅ Large segments
- ❌ Write-heavy workloads
- ❌ Memory-constrained systems

### 4. Compression

**Concept**: Compress data before storage

**Impact**:
- ↓ Disk usage: 2-5x less space
- ↓ Network bandwidth: Less data to transfer
- ↑ CPU usage: Compression overhead
- ↔ Latency: Depends on CPU vs I/O bound

**When to use**:
- Disk I/O is bottleneck
- Network bandwidth is limited
- CPU resources available

### 5. Partitioning

**Concept**: Split log into independent partitions

**Impact**:
- ↑ Parallelism: Independent writers/readers
- ↑ Throughput: Linear scaling
- ↔ Ordering: Within partition only

**Optimal partition count**:
```
Partitions = CPU cores * 2-4
```

**Key considerations**:
- More partitions = Better parallelism
- Too many partitions = Coordination overhead
- Balance based on workload

## Configuration Tuning

### Latency-Optimized Configuration

Goal: Minimize p99 latency

```json
{
  "storage": {
    "segment_config": {
      "max_size": 536870912,     // 512MB (smaller segments)
      "use_mmap": true,
      "sync_on_write": false     // Use cache for speed
    },
    "cache_config": {
      "max_size": 8388608,       // 8MB (small cache)
      "max_buffer_time_ms": 1,   // 1ms (fast flush)
      "enabled": true
    }
  },
  "replication": {
    "quorum": {
      "replication_factor": 3,
      "write_quorum": 1,         // Async replication
      "read_quorum": 1
    }
  }
}
```

**Expected results**:
- p99 write latency: < 1ms
- Throughput: 100K+ ops/sec

### Throughput-Optimized Configuration

Goal: Maximize operations per second

```json
{
  "storage": {
    "segment_config": {
      "max_size": 4294967296,    // 4GB (large segments)
      "use_mmap": true,
      "sync_on_write": false
    },
    "cache_config": {
      "max_size": 134217728,     // 128MB (large cache)
      "max_buffer_time_ms": 50,  // 50ms (batch more)
      "enabled": true
    }
  },
  "replication": {
    "quorum": {
      "replication_factor": 3,
      "write_quorum": 1,
      "read_quorum": 1
    }
  }
}
```

**Expected results**:
- Throughput: 1M+ ops/sec
- p99 latency: 2-5ms

### Durability-Optimized Configuration

Goal: Maximize data safety

```json
{
  "storage": {
    "segment_config": {
      "max_size": 1073741824,
      "use_mmap": false,
      "sync_on_write": true      // fsync every write
    },
    "cache_config": {
      "enabled": false           // No caching
    }
  },
  "replication": {
    "quorum": {
      "replication_factor": 5,
      "write_quorum": 5,         // All replicas
      "read_quorum": 3
    }
  }
}
```

**Expected results**:
- Data loss window: Near zero
- Throughput: 10K-50K ops/sec
- p99 latency: 10-20ms

## Monitoring Performance

### Key Metrics

```bash
# Latency percentiles
write_latency_p50
write_latency_p99
write_latency_p999

# Throughput
write_throughput_bytes_per_sec
write_throughput_ops_per_sec
read_throughput_bytes_per_sec

# Resources
cpu_usage_percent
memory_usage_bytes
disk_io_ops_per_sec
network_bandwidth_mbps

# Health
replication_lag_ms
isr_count
epoch_number
```

### Performance Dashboard

Example Grafana queries:

```promql
# p99 write latency
histogram_quantile(0.99, rate(dlog_write_duration_seconds_bucket[5m]))

# Throughput
rate(dlog_writes_total[1m])

# Replication lag
dlog_replication_lag_seconds
```

## Common Bottlenecks

### 1. Disk I/O

**Symptoms**:
- High `iowait` CPU percentage
- Low throughput despite low CPU usage
- Increasing latency under load

**Solutions**:
```bash
# Check disk stats
iostat -x 1

# Upgrade to NVMe SSD
# Enable write caching
# Increase cache size
# Use larger segments (fewer files)
```

### 2. Network Bandwidth

**Symptoms**:
- Network utilization near 100%
- Replication lag increasing
- Throttled throughput

**Solutions**:
```bash
# Check network stats
ifconfig | grep bytes

# Upgrade network interface
# Enable compression
# Reduce replication factor
# Add more nodes
```

### 3. CPU Saturation

**Symptoms**:
- CPU usage at 100%
- Latency increases with load
- Cannot scale further

**Solutions**:
```bash
# Check CPU usage
top -H -p $(pидof dlog)

# Add more CPU cores
# Optimize hot paths
# Reduce compression
# Scale horizontally
```

### 4. Memory Pressure

**Symptoms**:
- High swap usage
- OOM killer triggered
- Performance degradation

**Solutions**:
```bash
# Check memory
free -h

# Add more RAM
# Reduce cache size
# Disable mmap
# Reduce partition count
```

### 5. Lock Contention

**Symptoms**:
- CPU usage low despite high load
- Poor multi-core scaling
- Perf shows lock contention

**Solutions**:
```bash
# Profile with perf
sudo perf record -g -p $(pidof dlog)

# Reduce critical sections
# Use finer-grained locks
# Consider lock-free data structures
```

## Best Practices

1. **Profile before optimizing**: Measure, don't guess
2. **Optimize hot paths**: 80/20 rule applies
3. **Test under load**: Synthetic benchmarks != production
4. **Monitor in production**: Catch regressions early
5. **Document changes**: Know what you changed and why

## Performance Checklist

- [ ] Run benchmarks baseline
- [ ] Profile with flamegraph
- [ ] Identify bottlenecks
- [ ] Apply targeted optimizations
- [ ] Re-benchmark
- [ ] Load test
- [ ] Monitor in production

---

For more information, see:
- [Operations Guide](OPERATIONS.md)
- [Architecture Documentation](ARCHITECTURE.md)
- [Configuration Reference](CONFIG.md)

