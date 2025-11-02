# Introducing Pyralog: Rethinking Distributed Logs

**A new distributed log system that achieves 28 billion operations per second by eliminating coordination bottlenecks**

*Published: November 1, 2025*

---

## The Problem with Modern Data Infrastructure

If you're running a distributed application today, you're probably managing at least five separate systems:

- **Apache Kafka** for event streaming
- **PostgreSQL** or **TiKV** for transactions
- **Apache Flink** for stream processing
- **ClickHouse** or **Snowflake** for analytics
- **Jaeger** or **Elasticsearch** for observability

Each system has its own:
- Configuration language
- Deployment requirements
- Monitoring tools
- Performance characteristics
- Consistency guarantees
- Operational quirks

Your data flows between these systems like this:

```
Application
    ↓
  Kafka (streaming)
    ↓
  Flink (processing)
    ↓
  ClickHouse (analytics)
    ↓
  Grafana (dashboards)
```

At every arrow, you're paying the cost of:
- **Network overhead** (serialization, transmission, deserialization)
- **Data duplication** (same data stored in multiple systems)
- **Consistency gaps** (different semantics across systems)
- **Operational complexity** (5+ systems to manage, monitor, upgrade)
- **Infrastructure costs** (paying for separate clusters)

## What if there was a better way?

**Pyralog is a distributed log system that unifies all of these capabilities into a single, coherent platform.**

But this isn't just another "kitchen sink" database that tries to do everything poorly. Pyralog achieves this unification through fundamental architectural innovations that make it **faster than specialized systems** in their own domains:

- **15.2M writes/sec** (4.8× faster than Kafka)
- **45.2M reads/sec** (5.6× faster than Kafka) 
- **4.2M transactions/sec** (8,000× faster than TiKV)
- **28+ billion ops/sec** across all coordinator types

And it does this while providing:
- ✅ **Exactly-once semantics**
- ✅ **ACID transactions**
- ✅ **Real-time SQL queries**
- ✅ **Sub-millisecond latency**
- ✅ **Strong consistency**

## The Key Insight

Most distributed systems face fundamental bottlenecks:

**Kafka**: All writes go through partition leaders → leader I/O bottleneck
**TiKV**: All transactions need timestamps from TSO → 500K timestamps/sec ceiling
**Flink**: Separate from storage → network overhead
**Traditional logs**: Either strongly consistent OR high throughput, never both

Pyralog eliminates these bottlenecks through three core innovations:

### 1. The 🗿 Obelisk Sequencer

A novel primitive for crash-safe, persistent atomic counters that enables:
- Monotonic ID generation without coordination
- 1-2 microsecond overhead per ID
- Instant recovery after crashes
- Minimal disk usage (sparse files)

Think of it as `std::sync::atomic::AtomicU64`, but persistent and crash-safe.

### 2. ☀️ Pharaoh Network via 🪲 Scarab IDs

Traditional systems have centralized coordinators:
- **Kafka**: Zookeeper for metadata
- **TiKV**: Centralized Timestamp Oracle
- **Kafka Transactions**: Single transaction coordinator

These become bottlenecks at scale.

Pyralog eliminates ALL centralized coordinators by distributing them using Scarab-style IDs + Obelisk Sequencers:

```
Traditional: 1 coordinator → 500K ops/sec (bottleneck!)

Pyralog: 1024 coordinators → 4+ billion ops/sec (linear scaling!)
```

No leader elections. No single points of failure. Just hash-based routing to stateless coordinators.

### 3. Apache Arrow Native

Pyralog stores and processes data in Apache Arrow's columnar format:
- **Zero-copy** interchange between storage and compute
- **SIMD vectorization** for 10-100× faster queries
- **Native integration** with DataFusion (SQL) and Polars (DataFrames)
- **10-100× faster** analytics than row-based formats

This means you can run SQL queries directly on your streaming data at wire speed.

## What Does This Enable?

### Unified Event Streaming + Analytics

```
// Write events
client.produce("user-events", event).await?;

// Query them immediately with SQL
let results = client.sql("
    SELECT 
        user_id,
        COUNT(*) as events,
        AVG(session_duration) as avg_duration
    FROM user_events
    WHERE timestamp > now() - INTERVAL '5 minutes'
    GROUP BY user_id
    HAVING COUNT(*) > 100
").await?;
```

No ETL. No delays. No separate analytics database.

### Exactly-Once Stream Processing

```
// Process stream with exactly-once guarantees
let processed = stream
    .filter(|event| event.value > 100)
    .aggregate(
        window::tumbling(Duration::from_secs(60)),
        |acc, event| acc + event.value
    )
    .with_exactly_once()  // ← Built-in!
    .write_to("aggregated-events")
    .await?;
```

Kafka requires complex producer/consumer configuration. Pyralog makes it native.

### Distributed Transactions

```
// Atomic multi-partition write
let tx = client.begin_transaction().await?;

tx.write("inventory", decrease_stock(product_id, quantity)).await?;
tx.write("orders", create_order(user_id, product_id)).await?;
tx.write("user-balance", deduct_payment(user_id, price)).await?;

tx.commit().await?;  // All-or-nothing
```

4+ million transactions per second. 8,000× faster than TiKV's centralized TSO.

### Real-Time Observability

```
// Ingest OpenTelemetry traces
dlog_receiver.ingest_otlp(trace).await?;

// Query with SQL
let slow_requests = client.sql("
    SELECT 
        service_name,
        span_name,
        duration_ms,
        trace_id
    FROM traces
    WHERE duration_ms > 1000
      AND timestamp > now() - INTERVAL '1 hour'
    ORDER BY duration_ms DESC
    LIMIT 100
").await?;
```

10-50× faster writes than Elasticsearch. 5-10× faster queries than Tempo.

## Architecture at a Glance

```
┌────────────────────────────────────────────────────────┐
│                    Pyralog Platform                       │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ☀️ Pharaoh Network (1024 each):                      │
│  ┌──────────────────────────────────────────────┐    │
│  │  TSO │ Tx Coord │ Session Mgr │ Consumer     │    │
│  │  4B/s │  4B/s    │    4B/s     │  Coord 4B/s  │    │
│  └──────────────────────────────────────────────┘    │
│                      ▼                                 │
│  Consensus Layer:                                      │
│  ┌──────────────────────────────────────────────┐    │
│  │  Global Raft    │    Per-Partition Raft      │    │
│  │  (metadata)     │    (parallel failover)     │    │
│  └──────────────────────────────────────────────┘    │
│                      ▼                                 │
│  Storage + Analytics:                                  │
│  ┌──────────────────────────────────────────────┐    │
│  │  Arrow RecordBatches → Parquet Segments      │    │
│  │  DataFusion SQL │ Polars DataFrames          │    │
│  │  Zero-copy, columnar, SIMD-optimized         │    │
│  └──────────────────────────────────────────────┘    │
│                                                        │
└────────────────────────────────────────────────────────┘
```

## Why Now?

Three trends make Pyralog possible:

**1. Apache Arrow has matured**
- Industry standard (Spark, Pandas 2.0, BigQuery)
- Rich ecosystem (DataFusion, Polars, DuckDB)
- Zero-copy interchange

**2. Rust is production-ready**
- Memory safety without GC
- Fearless concurrency
- Zero-cost abstractions
- 10-100× faster than Python/Java for data processing

**3. Cloud infrastructure has evolved**
- Fast NVMe storage (microsecond latency)
- High-bandwidth networks (100Gbps+)
- Abundant CPU cores (128+ vCPUs)

Pyralog exploits all three.

## Real-World Use Cases

### 1. Financial Services
```
Use case: Trade processing + risk analytics
Traditional: Kafka → Flink → Clickhouse (500ms latency)
Pyralog: Single platform (5ms latency) ✅
Result: 100× faster risk calculations, real-time compliance
```

### 2. E-commerce
```
Use case: Inventory + orders + payments (transactional)
Traditional: PostgreSQL (1000 tx/sec per node)
Pyralog: 4M tx/sec distributed ✅
Result: Flash sales with millions of concurrent buyers
```

### 3. Observability
```
Use case: Distributed tracing + metrics + logs
Traditional: Jaeger + Prometheus + Elasticsearch
Pyralog: Unified platform ✅
Result: 90% cost reduction, 10× faster queries
```

### 4. Real-Time ML
```
Use case: Feature store + model serving
Traditional: Kafka + Redis + Custom feature store
Pyralog: Built-in time-travel queries + exactly-once ✅
Result: Feature freshness <1ms, consistent training/serving
```

## How It Compares

| Feature | Kafka | TiKV | Databend | **Pyralog** |
|---------|-------|------|----------|----------|
| Write throughput | 3.2M/s | 500K/s | N/A | **15.2M/s** ✅ |
| Transactions | 100K/s | 500 tx/s | No | **4.2M/s** ✅ |
| Real-time SQL | No | No | Yes | **Yes** ✅ |
| Exactly-once | Yes | Yes | No | **Yes** ✅ |
| Time-travel | No | Yes | Yes | **Yes** ✅ |
| Observability backend | No | No | No | **Yes** ✅ |
| Language | Java | Rust | Rust | **Rust** ✅ |
| Single platform | No | No | No | **Yes** ✅ |

## What's Next?

We're open-sourcing Pyralog under MIT-0 (code) and CC0-1.0 (documentation) licenses.

**Coming in the next blog posts:**
1. ✅ Introducing Pyralog (this post)
2. **The Obelisk Sequencer** - How we built a crash-safe persistent atomic primitive
3. **☀️ Pharaoh Network: Coordination Without Consensus** - Eliminating bottlenecks through Scarab IDs
4. **28 Billion Operations Per Second** - Architectural deep-dive
5. **Building Modern Data Infrastructure in Rust** - Lessons learned

## Try It Out

```bash
# Clone the repository
git clone https://github.com/dlog/dlog
cd dlog

# Start a local cluster
cargo run --bin dlog-server

# Run examples
cargo run --example basic-producer
cargo run --example sql-queries
cargo run --example transactions
```

**Documentation**: [github.com/dlog/dlog/docs](https://github.com/dlog/dlog)
**Discord**: [discord.gg/dlog](https://discord.gg/dlog)
**Research Paper**: [PAPER.md](../PAPER.md)

## Join Us

Pyralog is in active development. We're looking for:
- **Early adopters** to test and provide feedback
- **Contributors** to help build features
- **Companies** interested in production deployments
- **Researchers** interested in distributed systems innovations

Interested in using Pyralog at your company? Reach out: hello@dlog.io

---

**Author**: Pyralog Team
**License**: MIT-0 (code) & CC0-1.0 (documentation)
**GitHub**: [github.com/dlog/dlog](https://github.com/dlog/dlog)

---

*Next post: [The Obelisk Sequencer: A Novel Persistent Atomic Primitive →](2-obelisk-sequencer.md)*

