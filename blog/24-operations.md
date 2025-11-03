# Operating Pyralog in Production: A Practical Guide

**Deployment, monitoring, and keeping the lights on**

*Published: November 3, 2025*

---

## Production Reality

Development: "It works on my laptop!"
Production: "It crashed at 3 AM on Black Friday."

**This guide covers what you actually need in production.**

---

## Deployment Architectures

### Option 1: Bare Metal (Highest Performance)

```
┌──────────────────────────────────────────┐
│         BARE METAL DEPLOYMENT             │
├──────────────────────────────────────────┤
│                                          │
│  Server 1-3: Obelisk Nodes              │
│  ├─ 32GB RAM, 4 cores                   │
│  ├─ NVMe SSD (sparse files)             │
│  └─ 10Gbps network                      │
│                                          │
│  Server 4-103: Pyramid Nodes            │
│  ├─ 256GB RAM, 32 cores                 │
│  ├─ 4×4TB NVMe SSD (RAID0)              │
│  └─ 25Gbps network                      │
│                                          │
│  Load Balancer: HAProxy/Envoy           │
│  ├─ Route to nearest Pyramid            │
│  └─ Health checks                       │
│                                          │
└──────────────────────────────────────────┘
```

**Cost**: ~$500/month per Pyramid node (rented)
**Performance**: ✅ Maximum
**Flexibility**: ⚠️ Manual scaling

---

### Option 2: Kubernetes (Most Flexible)

```yaml
# pyramid-node.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: pyramid-nodes
spec:
  serviceName: pyramid
  replicas: 100
  template:
    spec:
      containers:
      - name: pyramid
        image: pyralog/pyramid:latest
        resources:
          requests:
            memory: "128Gi"
            cpu: "16"
          limits:
            memory: "256Gi"
            cpu: "32"
        volumeMounts:
        - name: data
          mountPath: /data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 4Ti
```

**Cost**: ~$400/month per node (GKE/EKS)
**Performance**: ✅ Good (with proper tuning)
**Flexibility**: ✅ Auto-scaling, rolling updates

---

### Option 3: Cloud Managed (Easiest)

```terraform
# AWS deployment with Terraform
resource "aws_instance" "pyramid" {
  count         = 100
  instance_type = "i3en.8xlarge" # 32 vCPU, 256GB RAM, 2×7.5TB NVMe
  ami           = data.aws_ami.pyralog.id
  
  ebs_optimized = true
  
  tags = {
    Name = "pyramid-${count.index}"
    Role = "pyramid-node"
  }
}

resource "aws_instance" "obelisk" {
  count         = 3
  instance_type = "c6i.2xlarge" # 8 vCPU, 16GB RAM
  ami           = data.aws_ami.pyralog.id
  
  tags = {
    Name = "obelisk-${count.index}"
    Role = "obelisk-node"
  }
}
```

**Cost**: ~$600/month per node (AWS on-demand)
**Performance**: ✅ Good
**Flexibility**: ✅ Managed backups, monitoring

---

## Capacity Planning

### Hardware Sizing

**Obelisk Nodes** (ID generation):
```
Minimum:
  • 4 cores (for concurrency)
  • 16GB RAM (for caching)
  • 100GB NVMe SSD (sparse files)
  • 10Gbps network

Recommended:
  • 8 cores
  • 32GB RAM
  • 500GB NVMe SSD
  • 25Gbps network

Scale: 1 Obelisk per 1M IDs/sec
```

**Pyramid Nodes** (storage/compute):
```
Minimum:
  • 16 cores
  • 128GB RAM
  • 1TB NVMe SSD
  • 10Gbps network

Recommended:
  • 32+ cores
  • 256GB+ RAM
  • 4×4TB NVMe SSD (RAID0 for throughput)
  • 25Gbps+ network

Scale: 1 Pyramid per 5M writes/sec or 100GB data
```

### Cluster Sizing Examples

**Small (10-node cluster)**:
```
Data: 1TB
Writes: 50M/sec
Reads: 100M/sec
Cost: $4,000/month
Use: Startup, dev/staging
```

**Medium (100-node cluster)**:
```
Data: 10TB
Writes: 500M/sec
Reads: 1B/sec
Cost: $40,000/month
Use: Mid-size company
```

**Large (1000-node cluster)**:
```
Data: 100TB
Writes: 5B/sec
Reads: 10B/sec
Cost: $400,000/month
Use: Large enterprise, FAANG scale
```

---

## Monitoring & Observability

### Metrics (Prometheus)

```rust
use prometheus::{register_counter, register_histogram, register_gauge};

// Write metrics
static WRITE_TOTAL: Lazy<Counter> = Lazy::new(|| {
    register_counter!("pyralog_writes_total", "Total writes").unwrap()
});

static WRITE_LATENCY: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!("pyralog_write_latency_seconds", "Write latency").unwrap()
});

// Storage metrics
static STORAGE_BYTES: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!("pyralog_storage_bytes", "Storage used").unwrap()
});

// Query metrics
static QUERY_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!("pyralog_query_duration_seconds", "Query duration").unwrap()
});
```

### Grafana Dashboards

**Key Metrics to Monitor**:
```
Throughput:
  • Writes/sec per node
  • Reads/sec per node
  • Network bytes/sec

Latency:
  • p50, p95, p99, p999 write latency
  • p50, p95, p99, p999 read latency
  • Query execution time

Resources:
  • CPU utilization
  • Memory usage
  • Disk IOPS
  • Network bandwidth

Storage:
  • Total data size
  • Compaction lag
  • Segment count
  • Index size

Cluster Health:
  • Node count (up/down)
  • Partition distribution
  • Raft leader elections
  • Replication lag
```

### Alerts (Alertmanager)

```yaml
# alerts.yaml
groups:
- name: pyralog
  rules:
  - alert: HighWriteLatency
    expr: histogram_quantile(0.99, pyralog_write_latency_seconds) > 0.005
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Write p99 latency > 5ms"
      
  - alert: NodeDown
    expr: up{job="pyramid"} == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Pyramid node {{ $labels.instance }} is down"
      
  - alert: DiskSpacelow
    expr: (node_filesystem_avail_bytes / node_filesystem_size_bytes) < 0.1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Disk space < 10% on {{ $labels.instance }}"
      
  - alert: HighCPU
    expr: rate(node_cpu_seconds_total{mode="idle"}[5m]) < 0.2
    for: 10m
    labels:
      severity: warning
    annotations:
      summary: "CPU usage > 80% on {{ $labels.instance }}"
```

---

## Failure Modes & Recovery

### Scenario 1: Single Node Failure

**Symptoms**: One Pyramid node crashes

**Impact**:
- Affected partitions: ~1% (if 100 nodes)
- Writes: Fail for those partitions
- Reads: Served by replicas (if RF≥2)

**Recovery**:
```
1. Automatic:
   • Raft elects new leader (3-5 seconds)
   • Writes resume automatically
   
2. Manual (if persistent failure):
   • Replace failed node
   • Raft replicates data to new node
   • Time: ~10 minutes per 100GB
```

---

### Scenario 2: Network Partition

**Symptoms**: Cluster splits into two groups

**Impact**:
- Majority partition: Continues normally
- Minority partition: Read-only (no quorum)

**Recovery**:
```
1. Resolve network issue
2. Minority rejoins majority
3. Raft catches up (automatic)
4. Time: <1 minute after network restored
```

---

### Scenario 3: Full Cluster Failure

**Symptoms**: All nodes down (datacenter outage)

**Recovery**:
```
1. Restore power/network
2. Nodes start automatically
3. WAL replay per node (~1 minute per GB)
4. Cluster reforms (3-5 minutes)
5. Resume operations

Total downtime: 5-30 minutes (depends on WAL size)
```

---

### Scenario 4: Data Corruption

**Symptoms**: Checksum mismatch, corrupted segment

**Recovery**:
```
1. Detect via background verification
2. Mark segment as corrupted
3. Recover from replica:
   • If RF≥2: Copy from replica (automatic)
   • If RF=1: Restore from backup
4. Time: ~10 minutes per corrupted segment
```

---

## Performance Tuning

### 1. Write Optimization

```rust
// config.yaml
storage:
  # Larger MemTable = fewer flushes
  memtable_size: 128MB  # Default: 16MB
  
  # Batch writes for throughput
  write_batch_size: 10000  # Default: 1000
  
  # Async fsync for latency
  fsync_policy: interval(10ms)  # Default: every_write
```

**Impact**: 10× higher write throughput

---

### 2. Read Optimization

```rust
// config.yaml
storage:
  # Larger block cache = more hits
  block_cache_size: 32GB  # Default: 8GB
  
  # Enable Bloom filters
  bloom_filter_bits_per_key: 10
  
  # PPHM for L0 segments
  l0_index_type: pphm
```

**Impact**: 5× faster point lookups

---

### 3. Compaction Tuning

```rust
// config.yaml
storage:
  # More threads = faster compaction
  compaction_threads: 8  # Default: 4
  
  # Aggressive compaction for read-heavy
  compaction_style: leveled
  compaction_trigger: 0.5  # Start at 50% capacity
```

**Impact**: 50% less read amplification

---

## Cost Optimization

### 1. Use Hybrid Storage

```
Hot data (30%): Native LSM on SSD ($100/TB/month)
Cold data (70%): External files on S3 ($23/TB/month)

Savings: 64% storage cost
```

### 2. Right-Size Instances

```
Don't overprovision:
  • Start small
  • Monitor actual usage
  • Scale gradually

Example:
  • Provisioned: 256GB RAM, 50% used
  • Actual need: 128GB RAM
  • Savings: $200/month per node
```

### 3. Use Reserved/Spot Instances

```
AWS Reserved Instances (1-year):
  • On-demand: $600/month
  • Reserved: $360/month (40% off)
  
AWS Spot Instances:
  • On-demand: $600/month
  • Spot: $180/month (70% off)
  • Risk: Instance can be terminated
  • Use for: Read replicas, batch processing
```

---

## Disaster Recovery

### Backup Strategy

```yaml
# backup.yaml
schedule:
  # Incremental every hour
  incremental: "0 * * * *"
  
  # Full backup daily
  full: "0 2 * * *"
  
destination:
  type: s3
  bucket: pyralog-backups
  encryption: aes256
  
retention:
  incremental: 7 days
  full: 30 days
```

### Recovery Testing

```bash
# Monthly DR drill
1. Stop cluster
2. Delete all data
3. Restore from backup
4. Verify data integrity
5. Measure recovery time

Target: RTO < 1 hour, RPO < 15 minutes
```

---

## Summary

Operating Pyralog in production requires:

### Deployment
- ✅ Choose architecture: Bare metal, K8s, or cloud
- ✅ Size hardware: 32 cores, 256GB RAM, 4TB NVMe per Pyramid
- ✅ Scale: 1 Pyramid per 5M writes/sec or 100GB data

### Monitoring
- ✅ Metrics: Prometheus + Grafana
- ✅ Alerts: Latency, nodes, disk, CPU
- ✅ Dashboards: Throughput, latency, resources

### Operations
- ✅ Handle failures: Automatic recovery in 3-5 seconds
- ✅ Tune performance: MemTable size, block cache, compaction
- ✅ Optimize cost: Hybrid storage, right-sizing, reserved instances

### The Bottom Line

Production Pyralog is **reliable and observable**:
- Automatic failover: 3-5 seconds
- Self-healing: Raft + replicas
- Full visibility: Metrics, logs, traces
- Cost-effective: 64% savings with hybrid storage

*Ship with confidence.*

---

## Next Steps

- Read [Deployment Guide](../docs/deployment.md)
- See [Monitoring Setup](../docs/monitoring.md)
- Check [Runbooks](../docs/runbooks/) for common issues

---

*Part 24 of the Pyralog Blog Series*

*Previously: [PoW Without Miners](23-pow-useful.md)*
*Next: [Migrating from Kafka](25-kafka-migration.md)*

