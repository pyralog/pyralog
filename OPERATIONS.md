# Pyralog Operations Guide

This guide covers deploying, operating, and maintaining Pyralog in production.

## Table of Contents

1. [Deployment](#deployment)
2. [Configuration](#configuration)
3. [Monitoring](#monitoring)
4. [Backup and Recovery](#backup-and-recovery)
5. [Scaling](#scaling)
6. [Troubleshooting](#troubleshooting)
7. [Security](#security)
8. [Performance Tuning](#performance-tuning)

## Deployment

### System Requirements

#### Minimum (Development)
- 2 CPU cores
- 4GB RAM
- 20GB SSD storage
- 1 Gbps network

#### Recommended (Production)
- 8+ CPU cores
- 32GB+ RAM
- 500GB+ NVMe SSD
- 10 Gbps network
- Separate disk for data and logs

#### Hardware Selection

**CPU**:
- Prefer higher clock speeds over more cores
- Modern Intel Xeon or AMD EPYC recommended
- Hyper-threading beneficial

**Memory**:
- More is better (for caching)
- ECC RAM recommended
- 32-64GB typical for production

**Storage**:
- NVMe SSDs strongly recommended
- SAS SSDs acceptable
- SATA SSDs not recommended
- RAID not required (replication handles redundancy)

**Network**:
- 10 Gbps minimum for production
- Low-latency network critical
- Direct connections between nodes preferred

### Operating System

#### Recommended
- Linux (Ubuntu 22.04 LTS, RHEL 8+, Debian 11+)
- macOS (development only)
- Windows (development only)

#### OS Tuning

**Linux Kernel Parameters** (`/etc/sysctl.conf`):
```bash
# Network
net.core.rmem_max = 134217728
net.core.wmem_max = 134217728
net.ipv4.tcp_rmem = 4096 87380 67108864
net.ipv4.tcp_wmem = 4096 65536 67108864
net.core.netdev_max_backlog = 5000

# File descriptors
fs.file-max = 2097152
fs.nr_open = 2097152

# Swappiness
vm.swappiness = 1

# Transparent huge pages
vm.nr_hugepages = 1024
```

Apply changes:
```bash
sudo sysctl -p
```

**File Limits** (`/etc/security/limits.conf`):
```
* soft nofile 1048576
* hard nofile 1048576
* soft nproc 32768
* hard nproc 32768
```

**Disable Transparent Huge Pages**:
```bash
echo 'never' > /sys/kernel/mm/transparent_hugepage/enabled
echo 'never' > /sys/kernel/mm/transparent_hugepage/defrag
```

### Installation

#### From Binary

```bash
# Download latest release
wget https://github.com/pyralog/pyralog/releases/download/v0.1.0/pyralog-linux-amd64.tar.gz

# Extract
tar -xzf dlog-linux-amd64.tar.gz

# Install
sudo mv dlog /usr/local/bin/
sudo chmod +x /usr/local/bin/dlog
```

#### From Source

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/pyralog/pyralog.git
cd dlog
cargo build --release

# Install
sudo cp target/release/dlog /usr/local/bin/
```

#### Docker

```bash
# Pull image
docker pull dlog/dlog:latest

# Run
docker run -d \
  --name dlog-1 \
  -p 9092:9092 \
  -p 9093:9093 \
  -v /data/dlog:/data \
  -e NODE_ID=1 \
  dlog/dlog:latest
```

### Cluster Deployment

#### Three-Node Cluster (Minimum for HA)

**Node 1** (node1.example.com):
```json
{
  "node": {
    "node_id": 1,
    "data_dir": "/var/lib/dlog",
    "cluster_nodes": [1, 2, 3]
  },
  "network": {
    "listen_address": "0.0.0.0:9092",
    "internal_address": "node1.example.com:9093"
  }
}
```

**Node 2** (node2.example.com):
```json
{
  "node": {
    "node_id": 2,
    "data_dir": "/var/lib/dlog",
    "cluster_nodes": [1, 2, 3]
  },
  "network": {
    "listen_address": "0.0.0.0:9092",
    "internal_address": "node2.example.com:9093"
  }
}
```

**Node 3** (node3.example.com):
```json
{
  "node": {
    "node_id": 3,
    "data_dir": "/var/lib/dlog",
    "cluster_nodes": [1, 2, 3]
  },
  "network": {
    "listen_address": "0.0.0.0:9092",
    "internal_address": "node3.example.com:9093"
  }
}
```

#### Systemd Service

Create `/etc/systemd/system/dlog.service`:
```ini
[Unit]
Description=Pyralog Distributed Log Service
After=network.target

[Service]
Type=simple
User=dlog
Group=dlog
ExecStart=/usr/local/bin/dlog --config /etc/dlog/config.json
Restart=on-failure
RestartSec=5
LimitNOFILE=1048576
LimitNPROC=32768

[Install]
WantedBy=multi-user.target
```

Enable and start:
```bash
sudo systemctl daemon-reload
sudo systemctl enable dlog
sudo systemctl start dlog
sudo systemctl status dlog
```

## Configuration

### Configuration File

Full example (`/etc/dlog/config.json`):
```json
{
  "node": {
    "node_id": 1,
    "data_dir": "/var/lib/dlog",
    "cluster_nodes": [1, 2, 3]
  },
  "network": {
    "listen_address": "0.0.0.0:9092",
    "internal_address": "node1.example.com:9093",
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
      "max_size": 67108864,
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

### Environment Variables

```bash
DLOG_NODE_ID=1
DLOG_DATA_DIR=/var/lib/dlog
DLOG_LISTEN_ADDRESS=0.0.0.0:9092
DLOG_CLUSTER_NODES=1,2,3
RUST_LOG=info
```

## Monitoring

### Metrics

Key metrics to monitor:

#### Performance Metrics
- **Write Latency** (p50, p99, p999)
- **Read Latency** (p50, p99, p999)
- **Throughput** (bytes/sec, messages/sec)
- **Batch Size** (average records per batch)

#### Health Metrics
- **Leader Status** (is leader? for which partitions?)
- **ISR Count** (in-sync replicas per partition)
- **Replication Lag** (per follower)
- **Epoch Number** (current epoch per partition)

#### Resource Metrics
- **CPU Usage** (per core)
- **Memory Usage** (RSS, cache)
- **Disk Usage** (used, free, IOPS)
- **Network I/O** (bytes in/out, packets)

#### Error Metrics
- **Failed Writes**
- **Failed Reads**
- **Replication Errors**
- **Consensus Errors**

### Logging

Configure logging levels:
```bash
RUST_LOG=dlog=info,dlog::storage=debug
```

Log locations:
- System logs: `/var/log/dlog/dlog.log`
- Audit logs: `/var/log/dlog/audit.log`
- Error logs: `/var/log/dlog/error.log`

### Health Checks

```bash
# Check if node is running
curl http://localhost:9092/health

# Check cluster status
curl http://localhost:9092/status

# Check specific partition
curl http://localhost:9092/partitions/0/status
```

## Backup and Recovery

### Backup Strategy

#### Full Backup
```bash
# Stop node
sudo systemctl stop dlog

# Backup data directory
sudo tar -czf dlog-backup-$(date +%Y%m%d).tar.gz /var/lib/dlog/

# Copy to backup location
sudo cp dlog-backup-*.tar.gz /backup/

# Start node
sudo systemctl start dlog
```

#### Incremental Backup
```bash
# Backup only new segments
rsync -av --include='*.log' --include='*.index' \
  /var/lib/dlog/ /backup/dlog-incremental/
```

#### Cloud Backup (S3)
```bash
aws s3 sync /var/lib/dlog/ s3://my-bucket/dlog-backups/node-1/
```

### Disaster Recovery

#### Restore from Backup
```bash
# Stop node
sudo systemctl stop dlog

# Restore data
sudo rm -rf /var/lib/dlog/*
sudo tar -xzf dlog-backup-20250101.tar.gz -C /

# Start node
sudo systemctl start dlog
```

#### Partition Recovery
If a partition loses all replicas:
1. Identify backup with latest data
2. Restore to one node
3. Let replication rebuild other replicas
4. Verify data integrity

## Scaling

### Adding Nodes

1. **Prepare new node**
   ```bash
   # Install Pyralog
   # Configure with new node_id
   # Add to cluster_nodes list
   ```

2. **Update existing nodes**
   ```json
   {
     "cluster_nodes": [1, 2, 3, 4]  // Add new node
   }
   ```

3. **Restart existing nodes** (one at a time)
   ```bash
   sudo systemctl restart dlog
   ```

4. **Start new node**
   ```bash
   sudo systemctl start dlog
   ```

5. **Rebalance partitions**
   ```bash
   dlog-admin rebalance --cluster localhost:9092
   ```

### Removing Nodes

1. **Reassign partitions** away from node
2. **Wait for replication** to complete
3. **Stop node**
   ```bash
   sudo systemctl stop dlog
   ```
4. **Update cluster configuration** on remaining nodes
5. **Remove from monitoring**

### Vertical Scaling

To upgrade a node:
1. Add new node with better specs
2. Rebalance partitions to new node
3. Remove old node
4. Repeat for each node

## Troubleshooting

### Common Issues

#### High Write Latency

**Symptoms**: p99 latency > 10ms

**Causes**:
- Disk saturation
- Network congestion
- Write cache disabled

**Solutions**:
```bash
# Enable write cache
"cache_config": { "enabled": true, "max_size": 67108864 }

# Increase segment size
"segment_config": { "max_size": 2147483648 }

# Check disk performance
iostat -x 1
```

#### Replication Lag

**Symptoms**: Followers behind leader

**Causes**:
- Network latency
- Slow follower disk
- Overloaded follower

**Solutions**:
```bash
# Check lag
curl http://localhost:9092/replication/status

# Reduce write load
# Add more replicas
# Upgrade slow node
```

#### Leader Election Failures

**Symptoms**: Frequent leader changes

**Causes**:
- Network instability
- Node overload
- Election timeout too low

**Solutions**:
```json
// Increase election timeout
"election_timeout": {
  "min_ms": 300,
  "max_ms": 600
}
```

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug dlog --config config.json

# Enable trace logging
RUST_LOG=trace dlog --config config.json
```

### Support Information

When reporting issues, include:
- Pyralog version
- Configuration file
- Recent logs
- System metrics
- Network topology

## Security

### Network Security

```bash
# Firewall rules
sudo ufw allow 9092/tcp  # Client port
sudo ufw allow 9093/tcp  # Internal port (restrict to cluster IPs)

# Restrict internal port
sudo ufw allow from 10.0.1.0/24 to any port 9093
```

### Authentication

(To be implemented)
- mTLS for node-to-node communication
- Token-based authentication for clients
- RBAC for access control

### Encryption

(To be implemented)
- TLS for client connections
- Encryption at rest
- Key rotation

## Performance Tuning

### For Low Latency

```json
{
  "storage": {
    "cache_config": {
      "max_size": 8388608,
      "max_buffer_time_ms": 1,
      "enabled": true
    },
    "segment_config": {
      "sync_on_write": true
    }
  }
}
```

### For High Throughput

```json
{
  "storage": {
    "cache_config": {
      "max_size": 134217728,
      "max_buffer_time_ms": 50,
      "enabled": true
    },
    "segment_config": {
      "max_size": 4294967296,
      "sync_on_write": false
    }
  }
}
```

### For Durability

```json
{
  "storage": {
    "segment_config": {
      "sync_on_write": true
    }
  },
  "replication": {
    "quorum": {
      "write_quorum": 3,
      "replication_factor": 3
    }
  }
}
```

---

For more information, see:
- [Architecture Documentation](ARCHITECTURE.md)
- [Performance Guide](PERFORMANCE.md)
- [Troubleshooting Guide](TROUBLESHOOTING.md)

