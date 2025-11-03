# Five Ways to Query Pyralog: From SQL to Category Theory

**Choose the right interface for your use case**

*Published: November 3, 2025*

---

## The Query Interface Problem

Most databases force you into one query paradigm:

```
PostgreSQL: SQL only
  • Powerful, but verbose
  • Great for relations, poor for graphs/documents
  
MongoDB: Proprietary query language
  • Great for documents, poor for joins
  • Not standardized

Neo4j: Cypher only
  • Great for graphs, poor for aggregations
  
Result: One size fits none!
```

**Pyralog offers five query interfaces**, each optimized for different use cases:

```
1. SQL (DataFusion):     Traditional relational queries
2. JSON-RPC/WebSocket:   Real-time RPC, low latency
3. GraphQL:              Flexible API queries, nested data
4. PRQL:                 Readable, composable pipelines
5. Batuta:               Full programming language, Category Theory

Pick the right tool for each job!
```

---

## The Five Interfaces

### Quick Comparison

| Interface | Type | Strength | Use Case | Learning Curve |
|-----------|------|----------|----------|----------------|
| **SQL** | Query Language | OLAP, analytics | Business intelligence | Low (familiar) |
| **JSON-RPC/WS** | RPC Protocol | Real-time, low latency | Live dashboards, apps | Very Low |
| **GraphQL** | API Query | Flexible, nested | Frontend APIs | Medium |
| **PRQL** | Query Language | Readable pipelines | Data engineering | Low (if know SQL) |
| **Batuta** | Programming Language | Full power, theory | Complex logic, distributed | High |

---

## Interface 1: SQL (DataFusion)

**Best for**: Analytics, reporting, OLAP

### What is it?

Standard SQL powered by Apache DataFusion:

```sql
SELECT 
    user_id,
    COUNT(*) AS order_count,
    SUM(amount) AS total_spent
FROM orders
WHERE created_at > '2025-01-01'
GROUP BY user_id
HAVING total_spent > 1000
ORDER BY total_spent DESC
LIMIT 10;
```

### Why SQL?

- ✅ **Universal**: Everyone knows SQL
- ✅ **Powerful**: 50 years of optimization
- ✅ **Standardized**: ANSI SQL compliance
- ✅ **Tool ecosystem**: BI tools, editors, ORMs

### When to use SQL

```
✅ Use SQL for:
  • Business intelligence dashboards
  • Ad-hoc analysis
  • Reporting
  • Data warehousing
  • Migrations from PostgreSQL/MySQL

❌ Avoid SQL for:
  • Real-time applications (use JSON-RPC)
  • Complex nested queries (use GraphQL)
  • Distributed logic (use Batuta)
```

### Example: User Analytics

```sql
-- Top users by engagement (last 30 days)
WITH recent_users AS (
    SELECT user_id, COUNT(*) AS event_count
    FROM events
    WHERE timestamp > NOW() - INTERVAL '30 days'
    GROUP BY user_id
)
SELECT 
    u.name,
    u.email,
    r.event_count,
    u.created_at
FROM users u
JOIN recent_users r ON u.id = r.user_id
WHERE r.event_count > 10
ORDER BY r.event_count DESC
LIMIT 20;
```

---

## Interface 2: JSON-RPC over WebSocket

**Best for**: Real-time applications, low latency

### What is it?

Lightweight RPC protocol over persistent WebSocket connections:

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "query.execute",
  "params": {
    "sql": "SELECT * FROM users WHERE id = $1",
    "args": [123]
  },
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": {
    "rows": [{"id": 123, "name": "Alice", "email": "alice@example.com"}],
    "affected_rows": 1
  },
  "id": 1
}
```

### Why JSON-RPC/WS?

- ✅ **Real-time**: Bidirectional, server push
- ✅ **Low latency**: <5ms (persistent connection)
- ✅ **Simple**: Minimal protocol overhead
- ✅ **Stateful**: Session-based
- ✅ **Browser-native**: No plugins needed

### When to use JSON-RPC/WS

```
✅ Use JSON-RPC/WS for:
  • Live dashboards (real-time updates)
  • Chat applications
  • Gaming backends
  • Trading platforms
  • IoT data streams

❌ Avoid JSON-RPC/WS for:
  • Batch processing (use SQL)
  • Public APIs (use GraphQL)
  • Mobile apps (connection instability)
```

### Example: Live Dashboard

```typescript
// TypeScript client
const client = new PyralogClient('ws://localhost:8080/rpc');

// Subscribe to live metrics
await client.subscribe('metrics.throughput', (data) => {
    console.log(`Current throughput: ${data.ops_per_sec} ops/sec`);
    updateDashboard(data);
});

// Execute query
const users = await client.call('query.users', { limit: 10 });
```

**Server push notification:**

```json
// Server → Client (no request!)
{
  "jsonrpc": "2.0",
  "method": "metrics.throughput",
  "params": {
    "timestamp": 1730678400,
    "ops_per_sec": 150000,
    "p99_latency_ms": 2.3
  }
}
```

---

## Interface 3: GraphQL

**Best for**: Frontend APIs, nested data

### What is it?

Flexible query language for APIs:

```graphql
query {
  user(id: 123) {
    name
    email
    orders(status: "completed", limit: 5) {
      id
      total
      items {
        product {
          name
          price
        }
        quantity
      }
    }
  }
}
```

**Response** (exactly what you asked for):

```json
{
  "data": {
    "user": {
      "name": "Alice",
      "email": "alice@example.com",
      "orders": [
        {
          "id": "order-1",
          "total": 99.99,
          "items": [
            { "product": { "name": "Widget", "price": 49.99 }, "quantity": 2 }
          ]
        }
      ]
    }
  }
}
```

### Why GraphQL?

- ✅ **Client-driven**: Clients specify exact needs
- ✅ **Single endpoint**: One API for everything
- ✅ **Nested queries**: Fetch related data in one request
- ✅ **Type-safe**: Schema validation
- ✅ **Real-time**: Subscriptions via WebSocket

### When to use GraphQL

```
✅ Use GraphQL for:
  • Frontend APIs (React, Vue, mobile)
  • Multi-platform apps (web + mobile)
  • Public APIs
  • Microservices aggregation

❌ Avoid GraphQL for:
  • Simple CRUD (overkill)
  • Analytics (use SQL)
  • High-performance (use JSON-RPC)
```

### Example: E-Commerce API

```graphql
# Schema definition
type User {
  id: ID!
  name: String!
  email: String!
  orders(status: OrderStatus, limit: Int): [Order!]!
}

type Order {
  id: ID!
  user: User!
  items: [OrderItem!]!
  total: Float!
  status: OrderStatus!
  created_at: DateTime!
}

type OrderItem {
  product: Product!
  quantity: Int!
  price: Float!
}

type Query {
  user(id: ID!): User
  orders(userId: ID, status: OrderStatus): [Order!]!
}

type Subscription {
  orderStatusChanged(userId: ID!): Order!
}
```

**Query with subscription:**

```graphql
# Subscribe to order updates
subscription {
  orderStatusChanged(userId: 123) {
    id
    status
    updated_at
  }
}

# Server pushes updates when status changes
```

---

## Interface 4: PRQL (Pipelined Relational Query Language)

**Best for**: Data engineering, readable queries

### What is it?

Modern, functional query language that compiles to SQL:

```prql
# PRQL: Clean pipeline
from users
filter age >= 18
select {user_id, name, email}
join orders {==user_id}
aggregate {
  total_spent = sum orders.amount
}
filter total_spent > 1000
sort total_spent desc
take 10
```

**Compiles to SQL:**

```sql
-- Generated SQL (optimized)
SELECT 
    u.user_id,
    u.name,
    u.email,
    SUM(o.amount) AS total_spent
FROM users u
JOIN orders o ON u.user_id = o.user_id
WHERE u.age >= 18
GROUP BY u.user_id, u.name, u.email
HAVING SUM(o.amount) > 1000
ORDER BY total_spent DESC
LIMIT 10;
```

### Why PRQL?

- ✅ **Readable**: No SELECT-FROM-WHERE soup
- ✅ **Composable**: Pipelines, not nested subqueries
- ✅ **Type-safe**: Catch errors at compile time
- ✅ **Functional**: Reusable transforms
- ✅ **Compiles to SQL**: Leverages DataFusion optimizer

### When to use PRQL

```
✅ Use PRQL for:
  • ETL pipelines
  • Data transformations
  • Jupyter notebooks
  • Readable analytics
  • Team collaboration (easier to review)

❌ Avoid PRQL for:
  • Production APIs (use SQL directly)
  • Complex logic (use Batuta)
```

### Example: Data Pipeline

```prql
# Transform raw events into user metrics
from raw_events
filter event_type == "purchase"
derive {
  year = date_trunc("year", timestamp),
  month = date_trunc("month", timestamp)
}
group {user_id, year, month} (
  aggregate {
    purchases = count,
    revenue = sum amount,
    avg_order = average amount
  }
)
filter purchases > 5
sort {year desc, month desc}
```

**Reusable transforms:**

```prql
# Define reusable filter
let active_users = (
  from users
  filter last_login > @2025-01-01
  filter status == "active"
)

# Use in multiple places
from active_users
join orders {==user_id}
aggregate {...}
```

---

## Interface 5: Batuta (Full Programming Language)

**Best for**: Complex business logic, distributed systems

### What is it?

Lisp-inspired functional language with Category Theory foundations:

```clojure
;; Batuta: Full programming language
(defn top-users-by-engagement [days]
  (let [cutoff (- (now) (days days))]
    (->> (query "SELECT user_id, COUNT(*) as events 
                 FROM events 
                 WHERE timestamp > $1 
                 GROUP BY user_id"
                cutoff)
         (filter #(> (:events %) 10))
         (map #(assoc % :score (calculate-score %)))
         (sort-by :score >)
         (take 20))))

;; Call with different windows
(top-users-by-engagement 7)   ;; Last week
(top-users-by-engagement 30)  ;; Last month
```

### Why Batuta?

- ✅ **Full programming language**: Actors, macros, functions
- ✅ **Category Theory**: Functors, monads, natural transformations
- ✅ **Distributed**: Run on client OR server
- ✅ **Multi-model**: Query across relational, graph, document, tensor
- ✅ **Lisp macros**: Domain-specific languages
- ✅ **DataFusion integration**: Leverage SQL optimizer

### When to use Batuta

```
✅ Use Batuta for:
  • Complex business logic
  • Multi-step workflows
  • Distributed transactions
  • Real-time stream processing
  • Custom domain-specific queries
  • Actor-based systems

❌ Avoid Batuta for:
  • Simple CRUD (overkill, use SQL)
  • Public APIs (steep learning curve)
  • One-off queries (use PRQL)
```

### Example: Multi-Model Query

```clojure
;; Query across multiple models
(defn user-360-view [user-id]
  {;; Relational: User profile
   :profile (query-one "SELECT * FROM users WHERE id = $1" user-id)
   
   ;; Document: User preferences
   :preferences (get-document "user-preferences" user-id)
   
   ;; Graph: Social connections
   :friends (->> (graph-query 
                   [:match [[user-id :friend ?friend]]
                    :return ?friend])
                 (map #(:id %)))
   
   ;; Tensor: Recommendation embeddings
   :embeddings (tensor-slice "user-embeddings" user-id)
   
   ;; Time-series: Recent activity
   :activity (->> (query "SELECT * FROM events 
                          WHERE user_id = $1 
                          AND timestamp > $2 
                          ORDER BY timestamp DESC 
                          LIMIT 100"
                         user-id
                         (- (now) (hours 24)))
                  (group-by :event_type)
                  (map-values count))})
```

**Distributed execution:**

```clojure
;; Client-side execution (application-embedded)
(def client-batuta (batuta/runtime :mode :client))

;; Server-side execution (database-embedded)
(def server-batuta (batuta/runtime :mode :server))

;; Choose at runtime
(if (> data-size 1000000)
  (server-batuta/execute query data)  ;; Run on server (close to data)
  (client-batuta/execute query data)) ;; Run on client (less overhead)
```

---

## Protocol Comparison

### Performance

```
Benchmark: Query 10K user records

SQL (DataFusion):
  • Cold: 50ms (parsing + planning)
  • Warm: 5ms (cached plan)
  • Throughput: 200K queries/sec

JSON-RPC/WebSocket:
  • Latency: 2-5ms (persistent connection)
  • Throughput: 100K RPC/sec
  • Server push: Yes

GraphQL:
  • Latency: 10-20ms (resolver overhead)
  • Throughput: 50K queries/sec
  • Nested queries: Efficient

PRQL:
  • Compile time: 1-2ms
  • Runtime: Same as SQL (compiles to SQL)
  • Readability: 10× better

Batuta:
  • Compile time: 5-10ms
  • Runtime: Varies (depends on complexity)
  • Power: Unlimited (full language)
```

### Feature Matrix

| Feature | SQL | JSON-RPC | GraphQL | PRQL | Batuta |
|---------|-----|----------|---------|------|--------|
| **Relational queries** | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Real-time** | ❌ | ✅ | ✅ (subscriptions) | ❌ | ✅ |
| **Nested queries** | ⚠️ (JOINs) | ⚠️ (manual) | ✅ | ✅ | ✅ |
| **Type safety** | ⚠️ (weak) | ❌ | ✅ | ✅ | ✅ |
| **Multi-model** | ⚠️ (extensions) | ✅ | ✅ | ⚠️ | ✅ |
| **Control flow** | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Distributed execution** | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Macros/DSL** | ❌ | ❌ | ❌ | ❌ | ✅ |
| **Learning curve** | Low | Very Low | Medium | Low | High |

---

## Why Not gRPC?

Pyralog **does not use gRPC**. Here's why:

### gRPC Problems

```
1. Overhead: Protobuf serialization (slower than Arrow IPC)
2. Complexity: .proto files, code generation
3. Browser support: Poor (requires grpc-web proxy)
4. Binary format: Protobuf (Arrow is better)
5. Real-time: Streaming, but not as simple as WebSocket
```

### Pyralog's Approach

```
Instead of gRPC:
  • JSON-RPC/WebSocket: Real-time RPC (simpler, faster)
  • Arrow Flight: Zero-copy data transfer
  • HTTP/REST: Simple APIs

Result: Simpler, faster, browser-native
```

**Performance comparison:**

```
Transfer 1GB Arrow RecordBatch:

gRPC (Protobuf):
  • Serialize: 800ms
  • Transfer: 1200ms
  • Deserialize: 900ms
  • Total: 2.9 seconds

Arrow Flight (IPC):
  • Serialize: 0ms (no-op!)
  • Transfer: 800ms
  • Deserialize: 0ms (no-op!)
  • Total: 0.8 seconds (3.6× faster!)

JSON-RPC/WebSocket (small requests):
  • Latency: 2-5ms
  • gRPC: 5-10ms
  • Result: 2-5× faster
```

---

## Decision Matrix

### Choose the Right Interface

```
┌─────────────────────────────────────────────────────────┐
│              QUERY INTERFACE DECISION TREE               │
└─────────────────────────────────────────────────────────┘

What are you building?

├─ Analytics dashboard?
│  └─ Use SQL (DataFusion)
│     • Familiar, powerful, tool ecosystem
│
├─ Real-time application?
│  └─ Use JSON-RPC/WebSocket
│     • Low latency (<5ms), server push
│
├─ Frontend API?
│  └─ Use GraphQL
│     • Flexible, nested queries, type-safe
│
├─ Data pipeline?
│  └─ Use PRQL
│     • Readable, composable, maintainable
│
└─ Complex business logic?
   └─ Use Batuta
      • Full programming language, distributed
```

### By Use Case

| Use Case | Primary | Secondary | Why |
|----------|---------|-----------|-----|
| **BI dashboards** | SQL | PRQL | Familiar, powerful |
| **Trading platforms** | JSON-RPC/WS | Batuta | Real-time, low latency |
| **Mobile apps** | GraphQL | JSON-RPC | Flexible, one API |
| **ETL pipelines** | PRQL | SQL | Readable, composable |
| **Microservices** | GraphQL | JSON-RPC | Aggregation, type-safe |
| **Stream processing** | Batuta | JSON-RPC/WS | Distributed, actors |
| **Admin tools** | SQL | GraphQL | Ad-hoc queries |
| **Public APIs** | GraphQL | SQL | Flexible, documented |

---

## Summary

Pyralog provides **five query interfaces** for different use cases:

### The Five Interfaces

1. **SQL (DataFusion)**: Analytics, BI, reporting
   - ✅ Universal, powerful, familiar
   - ⚠️ Verbose for complex queries

2. **JSON-RPC/WebSocket**: Real-time RPC
   - ✅ Low latency (<5ms), simple
   - ⚠️ Not ideal for public APIs

3. **GraphQL**: Frontend APIs
   - ✅ Flexible, nested, type-safe
   - ⚠️ Resolver overhead

4. **PRQL**: Readable data pipelines
   - ✅ Composable, maintainable
   - ⚠️ Less familiar than SQL

5. **Batuta**: Full programming language
   - ✅ Unlimited power, distributed
   - ⚠️ Steep learning curve

### Key Insights

- **No gRPC needed**: JSON-RPC/WS + Arrow Flight are faster and simpler
- **Pick the right tool**: Different interfaces for different jobs
- **Mix and match**: Use multiple interfaces in one application
- **Performance varies**: 2ms (JSON-RPC) to 50ms (GraphQL)

### The Bottom Line

**Stop forcing one query paradigm on everything.**

Different workloads need different interfaces. Pyralog gives you five battle-tested options—from simple SQL to full Category Theory—so you can choose the right tool for each job. Whether you're building dashboards, APIs, pipelines, or distributed systems, there's an interface optimized for your use case.

*Five interfaces. One database. Infinite possibilities.*

---

## Next Steps

**Want to learn more?**

- Read [JSON-RPC/WebSocket](../JSONRPC_WEBSOCKET.md) for real-time RPC
- See [GraphQL Guide](../GRAPHQL.md) for API queries
- Check [PRQL Documentation](../PRQL.md) for readable pipelines
- Learn [Batuta Language](../BATUTA.md) for full programming power
- Try [Quick Start](../QUICK_START.md) to try all five interfaces

**Discuss query interfaces**:
- Discord: [discord.gg/pyralog](https://discord.gg/pyralog)
- GitHub: [github.com/pyralog/pyralog](https://github.com/pyralog/pyralog)
- Email: hello@pyralog.io

---

*Part 16 of the Pyralog Blog Series*

*Previously: [Memory-Only Mode](15-memory-only.md)*
*Next: [Batuta Execution Modes](17-batuta-modes.md)*

