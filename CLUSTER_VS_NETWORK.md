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
    /// Raft across cluster leaders (trusted environment)
    Raft(RaftNetwork),
    /// PBFT for Byzantine environments
    PBFT(PBFTNetwork),
    /// Tendermint for blockchain-style consensus
    Tendermint(TendermintNetwork),
    /// Proof of Work for permissionless networks
    PoW(PoWNetwork),
    /// Proof of Stake for economic incentives
    PoS(PoSNetwork),
}
```

#### Proof of Work (PoW)

**What it is**: Miners compete to solve computational puzzles to propose blocks.

**When to use**:
- Fully permissionless network (anyone can join)
- Maximum decentralization desired
- Sybil attack resistance needed
- No trusted parties

**Trade-offs**:
- ❌ High energy consumption
- ❌ Slow finality (10+ minutes)
- ❌ Vulnerable to 51% attacks
- ✅ Truly permissionless
- ✅ Proven security model (Bitcoin)

**Implementation**:
```rust
pub struct PoWNetwork {
    /// Mining difficulty (adjusts based on hashrate)
    difficulty: u64,
    /// Block time target (e.g., 10 minutes)
    target_block_time: Duration,
    /// Current blockchain
    chain: Blockchain,
}

impl PoWNetwork {
    pub async fn mine_block(&self, transactions: Vec<Transaction>) -> Block {
        let mut nonce = 0u64;
        let prev_hash = self.chain.last_block_hash();
        
        loop {
            let block = Block {
                transactions: transactions.clone(),
                prev_hash,
                timestamp: SystemTime::now(),
                nonce,
            };
            
            let hash = blake3::hash(&block.serialize());
            
            // Check if hash meets difficulty target
            if hash_meets_difficulty(&hash, self.difficulty) {
                return block;
            }
            
            nonce += 1;
        }
    }
    
    pub async fn adjust_difficulty(&mut self) {
        // Adjust every 2016 blocks (like Bitcoin)
        if self.chain.len() % 2016 == 0 {
            let actual_time = self.chain.last_2016_blocks_time();
            let target_time = self.target_block_time * 2016;
            
            if actual_time < target_time {
                self.difficulty += 1; // Increase difficulty
            } else {
                self.difficulty -= 1; // Decrease difficulty
            }
        }
    }
}
```

**Example Use Case**:
```
Public Pyralog Network:
- Anyone can run a cluster
- Miners validate cross-cluster transactions
- Block rewards incentivize participation
- Fully censorship-resistant
```

**Alternative: PoW Without Miners**

PoW can also be used without dedicated miners for other purposes:

1. **Anti-Spam/DoS Protection**:
   ```rust
   pub struct RequestPoW {
       difficulty: u32,  // Small, e.g., 20 bits
   }
   
   impl RequestPoW {
       pub fn verify_request(&self, req: &Request) -> bool {
           // Client must solve small PoW per request
           let hash = blake3::hash(&req.serialize());
           hash_meets_difficulty(&hash, self.difficulty)
       }
   }
   ```
   - Clients solve small PoW puzzle per request
   - Prevents request flooding without rate limits
   - No dedicated miners needed

2. **Rate Limiting**:
   ```rust
   pub struct ComputationalRateLimit {
       cost_per_operation: Duration,  // e.g., 10ms of work
   }
   ```
   - Small computational cost per operation
   - Self-regulating system load
   - Pay with computation, not tokens or quotas

3. **Sybil Resistance**:
   ```rust
   pub struct ClusterIdentity {
       cluster_id: ClusterId,
       pow_proof: PoWProof,  // One-time cost to join network
   }
   ```
   - PoW required to create cluster identity
   - Prevents cheap identity attacks
   - One-time cost per cluster join (not continuous mining)

4. **Priority Queue**:
   ```rust
   pub struct PriorityRequest {
       request: Request,
       pow_work: u64,  // Higher work = higher priority
   }
   ```
   - Higher PoW effort = higher priority in queue
   - Pay with computation, not tokens
   - Fair resource allocation without payment

5. **Time-Lock Puzzles**:
   ```rust
   pub struct TimeLockPuzzle {
       data: EncryptedData,
       difficulty: u64,  // Sequential computation required
   }
   ```
   - Data released after X computation time
   - Verifiable Delay Functions (VDFs)
   - No miners, just sequential work to unlock

6. **Useful PoW**:
   ```rust
   pub enum UsefulWork {
       MLTraining(ModelParams),
       Simulation(SimulationParams),
       Cryptanalysis(CryptoParams),
   }
   ```
   - Compute actual useful work instead of arbitrary hashes
   - ML training, scientific simulations, etc.
   - Side benefit of computation
   - Not wasteful like traditional mining

**When to Use PoW Without Miners**:
- ✅ Need spam/DoS protection
- ✅ Want computational rate limiting
- ✅ Require Sybil resistance
- ✅ Fair queuing without payment
- ✅ Don't want to run validator infrastructure
- ❌ Need fast finality (use PoS instead)
- ❌ Need economic incentives (use PoS instead)

#### Proof of Stake (PoS)

**What it is**: Validators stake tokens to participate in consensus; validators are selected based on stake.

**When to use**:
- Energy efficiency important
- Fast finality needed (seconds, not minutes)
- Economic incentives for good behavior
- Semi-permissioned network

**Trade-offs**:
- ✅ Energy efficient (99.9% less than PoW)
- ✅ Fast finality (2-6 seconds)
- ✅ Economic security (staked capital)
- ❌ "Nothing at stake" problem (solved by slashing)
- ❌ Initial distribution challenge

**Implementation**:
```rust
pub struct PoSNetwork {
    /// Validators and their stakes
    validators: HashMap<ValidatorId, Stake>,
    /// Current epoch
    epoch: u64,
    /// Slashing conditions
    slashing_rules: SlashingRules,
}

pub struct Stake {
    /// Amount staked (in tokens)
    amount: u128,
    /// Validator public key
    pub_key: PublicKey,
    /// Whether currently active
    active: bool,
}

impl PoSNetwork {
    pub fn select_validator(&self, slot: u64) -> ValidatorId {
        // Weighted random selection based on stake
        let total_stake: u128 = self.validators.values()
            .filter(|v| v.active)
            .map(|v| v.amount)
            .sum();
        
        let mut rng = self.deterministic_rng(self.epoch, slot);
        let target = rng.gen_range(0..total_stake);
        
        let mut cumulative = 0u128;
        for (id, stake) in &self.validators {
            if !stake.active { continue; }
            cumulative += stake.amount;
            if cumulative >= target {
                return *id;
            }
        }
        
        unreachable!()
    }
    
    pub async fn propose_block(
        &self, 
        validator: ValidatorId, 
        transactions: Vec<Transaction>
    ) -> Result<Block, Error> {
        // Validator must have stake
        let stake = self.validators.get(&validator)
            .ok_or(Error::NotValidator)?;
        
        if !stake.active {
            return Err(Error::InactiveValidator);
        }
        
        let block = Block {
            proposer: validator,
            transactions,
            epoch: self.epoch,
            timestamp: SystemTime::now(),
        };
        
        Ok(block)
    }
    
    pub async fn attest(&self, validator: ValidatorId, block_hash: Hash) -> Attestation {
        // Validator signs block hash
        let stake = &self.validators[&validator];
        let signature = stake.pub_key.sign(&block_hash);
        
        Attestation {
            validator,
            block_hash,
            signature,
            stake_weight: stake.amount,
        }
    }
    
    pub async fn finalize_block(&mut self, block: Block, attestations: Vec<Attestation>) -> Result<(), Error> {
        // Check if 2/3 of stake has attested
        let total_attesting_stake: u128 = attestations.iter()
            .map(|a| a.stake_weight)
            .sum();
        
        let total_stake: u128 = self.validators.values()
            .filter(|v| v.active)
            .map(|v| v.amount)
            .sum();
        
        if total_attesting_stake * 3 >= total_stake * 2 {
            // Block finalized!
            self.apply_block(block).await?;
            Ok(())
        } else {
            Err(Error::InsufficientAttestations)
        }
    }
    
    pub async fn slash_validator(&mut self, validator: ValidatorId, reason: SlashingReason) {
        if let Some(stake) = self.validators.get_mut(&validator) {
            match reason {
                SlashingReason::DoubleSign => {
                    // Slash 100% of stake
                    stake.amount = 0;
                    stake.active = false;
                }
                SlashingReason::Downtime => {
                    // Slash 1% of stake
                    stake.amount = stake.amount * 99 / 100;
                }
            }
        }
    }
}

pub enum SlashingReason {
    /// Validator signed two conflicting blocks
    DoubleSign,
    /// Validator offline for extended period
    Downtime,
}
```

**Example Use Case**:
```
Enterprise Pyralog Network:
- Known organizations run clusters
- Stake $1M+ in tokens to become validator
- Fast finality (3-6 seconds)
- Slashing for misbehavior
- Energy efficient
```

#### Comparison: PoW vs PoS

| Aspect | Proof of Work | Proof of Stake |
|--------|---------------|----------------|
| **Energy** | High (mining hardware) | Low (standard servers) |
| **Finality** | Probabilistic (10+ min) | Fast (2-6 seconds) |
| **Security** | Computational work | Economic stake |
| **Entry Barrier** | Mining hardware | Token ownership |
| **Attack Cost** | 51% hashrate | 51% of staked tokens |
| **Permissionless** | ✅ Fully | ⚠️ Semi (need tokens) |
| **Maturity** | Proven (Bitcoin, 15+ years) | Newer (Ethereum, 2+ years) |
| **Best For** | Public, permissionless | Semi-permissioned, fast |

#### Hybrid Approaches

Some Pyralog Networks may combine multiple consensus mechanisms:

```rust
pub enum HybridConsensus {
    /// PoW for block proposal, PoS for finalization
    PoWPoS {
        pow: PoWNetwork,
        pos: PoSNetwork,
    },
    
    /// PoS for normal operation, PoW as fallback
    PoSWithPoWFallback {
        pos: PoSNetwork,
        pow_threshold: Duration, // Switch to PoW if PoS stalls
    },
    
    /// Raft within trusted zones, PoS across zones
    ZonedConsensus {
        intra_zone: RaftNetwork,
        inter_zone: PoSNetwork,
    },
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

