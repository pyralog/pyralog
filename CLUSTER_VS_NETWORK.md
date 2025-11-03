# Pyralog Cluster vs Pyralog Network

**Understanding the hierarchy: Single cluster vs decentralized network of clusters**

---

## Quick Summary

```
Pyralog Network (Global)
    ↓
├── Pyralog Cluster 1 (Datacenter A)
│   ├── Pyramid Nodes (storage, consensus, compute)
│   └── Pharaoh Network (Obelisk nodes for coordination)
│
├── Pyralog Cluster 2 (Datacenter B)
│   ├── Pyramid Nodes
│   └── Pharaoh Network
│
└── Pyralog Cluster N (Datacenter N)
    ├── Pyramid Nodes
    └── Pharaoh Network
```

---

## 🔺 Pyralog Cluster

**Definition**: A single distributed computing cluster

### What It Is
- **One logical cluster** in a single datacenter/region
- Made up of **Pyramid nodes** (🔺) for storage, consensus, and compute
- Uses **Pharaoh Network** (☀️ Obelisk nodes) for coordination
- Strong consistency within the cluster via Raft

### Architecture
```
┌─────────────────────────────────────────────────────┐
│           🔺 Pyralog Cluster (Datacenter 1)         │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌───────────────────────────────────────────┐    │
│  │  🗿 Pharaoh Network (Obelisk Nodes)       │    │
│  │  • Coordination layer                     │    │
│  │  • Scarab ID generation                   │    │
│  │  • Session IDs, epochs, TSO               │    │
│  │  • Scales horizontally                    │    │
│  └───────────────────────────────────────────┘    │
│                     ↓ provides IDs                 │
│  ┌───────────────────────────────────────────┐    │
│  │  🔺 Pyramid Nodes (100s-1000s)            │    │
│  │  • Storage (LSM trees)                    │    │
│  │  • Consensus (Raft per partition)         │    │
│  │  • Compute (queries, actors, Batuta)      │    │
│  │  • Scales horizontally                    │    │
│  └───────────────────────────────────────────┘    │
│                                                     │
│  Characteristics:                                   │
│  • Strong consistency (Raft)                       │
│  • Low latency (same datacenter)                   │
│  • High throughput (500M writes/sec)               │
│  • Single administrative domain                    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Use Cases
- **Single datacenter deployment**
- **Regional database**
- **High-performance computing**
- **Low-latency applications**
- **Traditional distributed database**

### Scaling
- **Vertical**: Add more Pyramid nodes for capacity
- **Vertical**: Add more Obelisk nodes for coordination throughput
- **Limit**: Network bandwidth within datacenter
- **Typical**: 10-1000+ Pyramid nodes per cluster

---

## 🌐 Pyralog Network

**Definition**: Multiple Pyralog Clusters forming a Decentralized Autonomous Database

### What It Is
- **Federation of multiple Pyralog Clusters**
- Each cluster is independent and autonomous
- Clusters communicate peer-to-peer
- Decentralized coordination (no central authority)
- Global distribution across datacenters/regions
- See [DADBS.md](DADBS.md) for complete architecture

### Architecture
```
┌───────────────────────────────────────────────────────────────┐
│               🌐 Pyralog Network (Global)                      │
│         Decentralized Autonomous Database System               │
├───────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │ 🔺 Cluster US   │  │ 🔺 Cluster EU   │  │ 🔺 Cluster  │ │
│  │    (N. America) │  │    (Europe)     │  │    ASIA      │ │
│  ├─────────────────┤  ├─────────────────┤  ├──────────────┤ │
│  │ • Pyramid nodes │  │ • Pyramid nodes │  │ • Pyramid    │ │
│  │ • Pharaoh Net   │  │ • Pharaoh Net   │  │   nodes      │ │
│  │ • Raft (local)  │  │ • Raft (local)  │  │ • Pharaoh    │ │
│  │ • Autonomous    │  │ • Autonomous    │  │   Net        │ │
│  └────────┬────────┘  └────────┬────────┘  └──────┬───────┘ │
│           │                    │                   │         │
│           └────────────────────┼───────────────────┘         │
│                                │                             │
│           ┌────────────────────▼────────────────────┐        │
│           │  Cross-Cluster Coordination Layer       │        │
│           │  • Consensus: Raft/PBFT/Tendermint      │        │
│           │  • Replication: CRDTs, Vector Clocks    │        │
│           │  • Discovery: Gossip, DHT               │        │
│           │  • Governance: On-chain voting          │        │
│           │  • Economics: Token incentives          │        │
│           └─────────────────────────────────────────┘        │
│                                                               │
│  Characteristics:                                             │
│  • Eventual consistency (global)                             │
│  • High availability (geo-redundant)                         │
│  • Byzantine fault tolerance                                 │
│  • Autonomous operation                                      │
│  • No single point of control                                │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

### Use Cases
- **Global databases** (multi-region)
- **Decentralized applications** (DApps)
- **Censorship-resistant systems**
- **Multi-organization collaboration**
- **Edge computing networks**
- **Blockchain-like systems** (but with database features)

### Scaling
- **Horizontal**: Add more Pyralog Clusters (new regions/datacenters)
- **Geographic**: Place clusters close to users
- **Limit**: Cross-datacenter latency, global coordination overhead
- **Typical**: 3-100+ clusters globally

---

## Comparison

| Aspect | Pyralog Cluster | Pyralog Network |
|--------|-----------------|-----------------|
| **Scope** | Single datacenter/region | Multiple datacenters/global |
| **Nodes** | Pyramid + Obelisk | Multiple clusters |
| **Consistency** | Strong (Raft per partition) | Eventual (CRDTs, consensus) |
| **Latency** | Low (< 1ms within DC) | High (cross-region, 10-200ms) |
| **Throughput** | 500M writes/sec per cluster | Aggregated across clusters |
| **Fault Tolerance** | Node failures | Cluster failures, datacenter outages |
| **Governance** | Single admin | Decentralized (on-chain voting) |
| **Autonomy** | Coordinated | Autonomous |
| **Trust Model** | Trusted environment | Byzantine fault tolerant |
| **Use Case** | Regional database | Global decentralized database |

---

## Consistency Models

### Within a Pyralog Cluster (Strong)
```
Write to Cluster US:
  1. Client → Pyramid node (leader)
  2. Leader → Raft consensus (within cluster)
  3. Majority ACK (< 5ms)
  4. Client receives confirmation
  
Result: STRONGLY CONSISTENT within cluster
```

### Across Pyralog Network (Eventual)
```
Write to Cluster US:
  1. Write committed in Cluster US (strong consistency)
  2. Asynchronous replication to Cluster EU (eventual)
  3. Asynchronous replication to Cluster ASIA (eventual)
  4. Conflict resolution via CRDTs or consensus
  
Result: EVENTUALLY CONSISTENT across network
Time to consistency: seconds to minutes (depending on topology)
```

---

## Deployment Scenarios

### Scenario 1: Single Cluster (Traditional)
```
Use Case: Regional SaaS application
Setup: One Pyralog Cluster in AWS us-east-1
Nodes: 100 Pyramid nodes + 5 Obelisk nodes
Consistency: Strong (Raft)
Latency: <1ms
Cost: Moderate
```

### Scenario 2: Multi-Cluster, Centralized (Geo-Distribution)
```
Use Case: Global application with multi-region
Setup: 3 Pyralog Clusters (US, EU, ASIA)
Coordination: Centralized control plane
Replication: Active-passive or active-active
Consistency: Strong per region, eventual global
Latency: <1ms local, 50-200ms cross-region
Cost: High
```

### Scenario 3: Pyralog Network (Decentralized)
```
Use Case: Decentralized autonomous database
Setup: 10+ Pyralog Clusters (multiple organizations)
Coordination: Decentralized (no single owner)
Governance: On-chain voting
Consensus: PBFT or Tendermint
Consistency: Eventual (CRDTs)
Latency: Varies by topology
Cost: Distributed across participants
Benefits: Censorship-resistant, autonomous
```

---

## Cross-Cluster Coordination

### Gossip-Based Discovery
```rust
pub struct ClusterDiscovery {
    /// Known clusters in the network
    clusters: HashMap<ClusterId, ClusterInfo>,
    /// Gossip protocol for cluster membership
    gossip: GossipProtocol,
}

impl ClusterDiscovery {
    pub async fn discover_clusters(&mut self) {
        // Gossip with peer clusters
        for peer in self.gossip.select_peers(3) {
            let peer_clusters = peer.get_known_clusters().await;
            self.merge_cluster_info(peer_clusters);
        }
    }
}
```

### Cross-Cluster Replication
```rust
pub struct CrossClusterReplicator {
    /// Local cluster ID
    local_cluster: ClusterId,
    /// Remote clusters to replicate to
    remote_clusters: Vec<ClusterEndpoint>,
    /// CRDT for conflict resolution
    crdt: CvRDT,
}

impl CrossClusterReplicator {
    pub async fn replicate(&self, record: Record) {
        // Replicate to remote clusters (async)
        for remote in &self.remote_clusters {
            tokio::spawn(async move {
                remote.replicate(record.clone()).await.ok();
            });
        }
    }
}
```

### Consensus Across Clusters
```rust
pub enum NetworkConsensus {
    /// Raft across cluster leaders
    Raft(RaftNetwork),
    /// PBFT for Byzantine environments
    PBFT(PBFTNetwork),
    /// Tendermint for blockchain-style consensus
    Tendermint(TendermintNetwork),
    /// Proof of Stake for economic incentives
    PoS(PoSNetwork),
}
```

---

## Migration Paths

### Path 1: Single Cluster → Multi-Cluster
```
1. Deploy second Pyralog Cluster in new region
2. Configure cross-cluster replication
3. Enable geo-routing (read local, write home)
4. Gradually move to active-active
```

### Path 2: Multi-Cluster → Pyralog Network
```
1. Enable decentralized discovery (gossip)
2. Deploy consensus layer (PBFT/Tendermint)
3. Implement governance (voting, proposals)
4. Add economic layer (tokens, incentives)
5. Remove central control plane
```

---

## When to Use What

### Use Pyralog Cluster When:
✅ Single datacenter/region sufficient  
✅ Strong consistency required  
✅ Low latency critical  
✅ Traditional database use case  
✅ Single organization/trust domain  

### Use Pyralog Network When:
✅ Global distribution required  
✅ Multi-datacenter deployment  
✅ Eventual consistency acceptable  
✅ Decentralized control desired  
✅ Byzantine fault tolerance needed  
✅ Multiple organizations collaborating  
✅ Censorship resistance important  

---

## Summary

**Pyralog Cluster**:
- 🔺 One distributed computing cluster
- Strong consistency, low latency
- Traditional distributed database
- Single administrative domain

**Pyralog Network**:
- 🌐 Multiple Pyralog Clusters
- Decentralized Autonomous Database
- Global distribution, eventual consistency
- Multi-organization, Byzantine fault tolerant
- See [DADBS.md](DADBS.md) for complete details

---

## See Also

- [NODES.md](NODES.md) - Obelisk and Pyramid node architecture
- [DADBS.md](DADBS.md) - Decentralized Autonomous Database Systems
- [BRANDING.md](BRANDING.md) - Terminology and naming conventions
- [ARCHITECTURE.md](ARCHITECTURE.md) - System internals

