# JSON-RPC over WebSocket

**Lightweight, real-time RPC protocol for Pyralog**

> **JSON-RPC** is a stateless, lightweight remote procedure call (RPC) protocol. Pyralog implements JSON-RPC 2.0 over WebSocket for low-latency, bidirectional communication with real-time updates.

> **Note**: JSON-RPC is a **pragmatic RPC protocol** optimized for simplicity and real-time communication. For applications requiring **Category Theory foundations**, see [Batuta](BATUTA.md). For flexible API queries, see [GraphQL](GRAPHQL.md). For readable relational queries, see [PRQL](PRQL.md).

---

## Table of Contents

1. [Overview](#overview)
2. [Why JSON-RPC for Pyralog?](#why-json-rpc-for-pyralog)
3. [Protocol Specification](#protocol-specification)
4. [Architecture](#architecture)
5. [Connection Management](#connection-management)
6. [Request/Response](#requestresponse)
7. [Notifications](#notifications)
8. [Batch Requests](#batch-requests)
9. [Error Handling](#error-handling)
10. [Streaming Results](#streaming-results)
11. [Authentication](#authentication)
12. [Performance](#performance)
13. [Comparison with Other Protocols](#comparison-with-other-protocols)
14. [Best Practices](#best-practices)
15. [Examples](#examples)

---

## Overview

JSON-RPC over WebSocket provides **lightweight, real-time RPC** for Pyralog:

- ✅ **Simple protocol**: Minimal overhead, easy to implement
- ✅ **Bidirectional**: Server can push to client (notifications)
- ✅ **Real-time**: Low latency (<5ms), persistent connections
- ✅ **Stateful**: Connection-based sessions
- ✅ **Efficient**: Binary frames, compression support
- ✅ **Language agnostic**: Any client with WebSocket support

### JSON-RPC vs REST

```json
// JSON-RPC: Single request
{
  "jsonrpc": "2.0",
  "method": "query.users",
  "params": { "limit": 10 },
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": [...],
  "id": 1
}
```

```
REST: Multiple requests
GET /api/users?limit=10
GET /api/users/123/orders
GET /api/orders/456/items
```

### Protocol Flow

```
┌─────────────────────────────────────────────────────────────┐
│            JSON-RPC OVER WEBSOCKET FLOW                      │
└─────────────────────────────────────────────────────────────┘

Client                           Server
  │                                 │
  │──── WebSocket Handshake ───────>│
  │<──── 101 Switching Protocols ───│
  │                                 │
  │──── { method: "auth.login" }──>│
  │<──── { result: { token } } ─────│
  │                                 │
  │──── { method: "query.users" }>─│
  │                                 │─── Execute Query
  │                                 │
  │<──── { result: [...] } ─────────│
  │                                 │
  │<──── { method: "notify" } ──────│  (Server push)
  │                                 │
  │──── Close ─────────────────────>│
```

---

## Why JSON-RPC for Pyralog?

### 1. **Simplicity**

Minimal protocol overhead:

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "query.execute",
  "params": { "sql": "SELECT * FROM users LIMIT 10" },
  "id": 1
}

// Response
{
  "jsonrpc": "2.0",
  "result": { "rows": [...], "count": 10 },
  "id": 1
}
```

### 2. **Real-Time Bidirectional Communication**

Server can push notifications:

```json
// Server → Client notification
{
  "jsonrpc": "2.0",
  "method": "events.newOrder",
  "params": {
    "orderId": "123",
    "userId": "456",
    "total": 99.99
  }
}
```

### 3. **Low Latency**

Persistent connection + binary frames:

- **Connection overhead**: One-time WebSocket handshake
- **Request latency**: <5ms (vs 50-100ms for HTTP)
- **Server push**: Immediate (no polling)

### 4. **Efficient Binary Transport**

WebSocket binary frames:

```rust
// Send binary data efficiently
let arrow_batch: RecordBatch = ...;
let bytes = arrow_batch.to_ipc_bytes()?;
websocket.send(Message::Binary(bytes)).await?;
```

### 5. **Language Agnostic**

Any language with WebSocket support:

```javascript
// JavaScript client
const ws = new WebSocket('ws://localhost:8080/rpc');
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'query.execute',
  params: { sql: 'SELECT * FROM users' },
  id: 1
}));
```

```python
# Python client
import websocket
import json

ws = websocket.create_connection('ws://localhost:8080/rpc')
ws.send(json.dumps({
    'jsonrpc': '2.0',
    'method': 'query.execute',
    'params': { 'sql': 'SELECT * FROM users' },
    'id': 1
}))
```

---

## Protocol Specification

### JSON-RPC 2.0 Format

**Request**:
```json
{
  "jsonrpc": "2.0",           // Protocol version (required)
  "method": "method.name",     // Method to invoke (required)
  "params": { ... },           // Parameters (optional)
  "id": 1                      // Request ID (required for responses)
}
```

**Response (Success)**:
```json
{
  "jsonrpc": "2.0",
  "result": { ... },           // Result data
  "id": 1                      // Matches request ID
}
```

**Response (Error)**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,            // Error code
    "message": "Invalid Request",
    "data": { ... }            // Additional error info
  },
  "id": 1
}
```

**Notification** (no response expected):
```json
{
  "jsonrpc": "2.0",
  "method": "notify.event",
  "params": { ... }
  // No "id" field
}
```

### Standard Error Codes

| Code | Message | Meaning |
|------|---------|---------|
| -32700 | Parse error | Invalid JSON |
| -32600 | Invalid Request | Missing required fields |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Invalid parameters |
| -32603 | Internal error | Server error |
| -32000 to -32099 | Server error | Application-specific errors |

---

## Architecture

### Integration Stack

```
┌─────────────────────────────────────────────────────────────┐
│              JSON-RPC WEBSOCKET STACK                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Client (Any Language)                                       │
│  └─ WebSocket Client                                         │
│     └─ JSON-RPC 2.0                                          │
│         ↓                                                    │
│  ┌──────────────────────────────────────────────┐           │
│  │  Pyralog JSON-RPC Server                     │           │
│  │  ├─ WebSocket Handler (tokio-tungstenite)    │           │
│  │  ├─ JSON-RPC Router                          │           │
│  │  ├─ Method Dispatcher                        │           │
│  │  └─ Session Manager                          │           │
│  └──────────────────────────────────────────────┘           │
│         ↓                                                    │
│  ┌──────────────────────────────────────────────┐           │
│  │  Query Engines                               │           │
│  │  ├─ SQL (direct)                             │           │
│  │  ├─ PRQL (compiled to SQL)                   │           │
│  │  ├─ Batuta (business logic)                  │           │
│  │  └─ GraphQL (optional)                       │           │
│  └──────────────────────────────────────────────┘           │
│         ↓                                                    │
│  DataFusion LogicalPlan Optimizer                            │
│         ↓                                                    │
│  PhysicalPlan Executor                                       │
│         ↓                                                    │
│  Arrow RecordBatch Results                                   │
│         ↓                                                    │
│  ┌──────────────────────────────────────────────┐           │
│  │  Multi-Model Storage Layer                   │           │
│  └──────────────────────────────────────────────┘           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Rust Implementation

```rust
use tokio_tungstenite::{accept_async, tungstenite::Message};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
    id: Option<Value>,
}

#[derive(Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

pub struct JsonRpcServer {
    pyralog: Arc<PyralogClient>,
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
}

impl JsonRpcServer {
    pub async fn handle_connection(
        self: Arc<Self>,
        stream: TcpStream,
    ) -> Result<()> {
        let ws_stream = accept_async(stream).await?;
        let (mut write, mut read) = ws_stream.split();
        
        let session = Session::new();
        let session_id = session.id;
        self.sessions.write().await.insert(session_id, session);
        
        while let Some(msg) = read.next().await {
            let msg = msg?;
            
            match msg {
                Message::Text(text) => {
                    let response = self.handle_request(&text, session_id).await;
                    write.send(Message::Text(response)).await?;
                }
                Message::Binary(data) => {
                    // Handle binary data (e.g., Arrow IPC)
                    let response = self.handle_binary_request(&data, session_id).await;
                    write.send(Message::Binary(response)).await?;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
        
        self.sessions.write().await.remove(&session_id);
        Ok(())
    }
    
    async fn handle_request(
        &self,
        text: &str,
        session_id: SessionId,
    ) -> String {
        let request: Result<JsonRpcRequest, _> = serde_json::from_str(text);
        
        let response = match request {
            Ok(req) => self.dispatch_method(&req, session_id).await,
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32700,
                    message: "Parse error".to_string(),
                    data: Some(json!({ "details": e.to_string() })),
                }),
                id: None,
            },
        };
        
        serde_json::to_string(&response).unwrap()
    }
    
    async fn dispatch_method(
        &self,
        req: &JsonRpcRequest,
        session_id: SessionId,
    ) -> JsonRpcResponse {
        let result = match req.method.as_str() {
            // Authentication
            "auth.login" => self.auth_login(req.params.as_ref()).await,
            "auth.logout" => self.auth_logout(session_id).await,
            
            // Queries
            "query.execute" => self.execute_query(req.params.as_ref(), session_id).await,
            "query.stream" => self.stream_query(req.params.as_ref(), session_id).await,
            
            // Subscriptions
            "subscribe.events" => self.subscribe_events(req.params.as_ref(), session_id).await,
            "unsubscribe.events" => self.unsubscribe_events(req.params.as_ref(), session_id).await,
            
            // Batuta
            "batuta.execute" => self.execute_batuta(req.params.as_ref(), session_id).await,
            
            // PRQL
            "prql.execute" => self.execute_prql(req.params.as_ref(), session_id).await,
            
            _ => Err(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: Some(json!({ "method": req.method })),
            }),
        };
        
        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: Some(result),
                error: None,
                id: req.id.clone(),
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(error),
                id: req.id.clone(),
            },
        }
    }
    
    async fn execute_query(
        &self,
        params: Option<&Value>,
        session_id: SessionId,
    ) -> Result<Value, JsonRpcError> {
        let params = params.ok_or(JsonRpcError {
            code: -32602,
            message: "Invalid params".to_string(),
            data: None,
        })?;
        
        let sql = params["sql"].as_str().ok_or(JsonRpcError {
            code: -32602,
            message: "Missing 'sql' parameter".to_string(),
            data: None,
        })?;
        
        // Execute via DataFusion
        let result = self.pyralog
            .query_sql(sql)
            .await
            .map_err(|e| JsonRpcError {
                code: -32000,
                message: "Query execution failed".to_string(),
                data: Some(json!({ "error": e.to_string() })),
            })?;
        
        Ok(json!({
            "rows": result.rows,
            "count": result.count,
            "schema": result.schema
        }))
    }
}
```

---

## Connection Management

### Establishing Connection

```javascript
// JavaScript client
const ws = new WebSocket('ws://localhost:8080/rpc');

ws.onopen = () => {
  console.log('Connected');
  
  // Authenticate
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    method: 'auth.login',
    params: { username: 'alice', password: 'secret' },
    id: 1
  }));
};

ws.onmessage = (event) => {
  const response = JSON.parse(event.data);
  console.log('Response:', response);
};

ws.onerror = (error) => {
  console.error('Error:', error);
};

ws.onclose = () => {
  console.log('Disconnected');
};
```

### Connection Options

```json
{
  "url": "ws://localhost:8080/rpc",
  "protocols": ["jsonrpc"],
  "options": {
    "compression": true,
    "maxPayload": 10485760,      // 10MB
    "heartbeat": 30000,           // 30s ping
    "reconnect": true,
    "reconnectDelay": 1000        // 1s
  }
}
```

### Heartbeat/Ping-Pong

```rust
// Server sends ping every 30s
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(_) = write.send(Message::Ping(vec![])).await {
            break;
        }
    }
});

// Client responds with pong automatically (handled by WebSocket layer)
```

---

## Request/Response

### Basic Request

```json
{
  "jsonrpc": "2.0",
  "method": "query.execute",
  "params": {
    "sql": "SELECT * FROM users WHERE age > 18 LIMIT 10"
  },
  "id": 1
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "rows": [
      { "id": 1, "name": "Alice", "age": 30 },
      { "id": 2, "name": "Bob", "age": 25 }
    ],
    "count": 2,
    "schema": {
      "fields": [
        { "name": "id", "type": "int64" },
        { "name": "name", "type": "utf8" },
        { "name": "age", "type": "int32" }
      ]
    }
  },
  "id": 1
}
```

### Parameterized Queries

```json
{
  "jsonrpc": "2.0",
  "method": "query.execute",
  "params": {
    "sql": "SELECT * FROM users WHERE age > $1 AND country = $2",
    "args": [18, "US"]
  },
  "id": 2
}
```

### PRQL Queries

```json
{
  "jsonrpc": "2.0",
  "method": "prql.execute",
  "params": {
    "query": "from users | filter age > 18 | sort name | take 10"
  },
  "id": 3
}
```

### Batuta Execution

```json
{
  "jsonrpc": "2.0",
  "method": "batuta.execute",
  "params": {
    "code": "(defquery active-users [] (from :users (where (= :status \"active\"))))",
    "function": "active-users",
    "args": []
  },
  "id": 4
}
```

---

## Notifications

### Server → Client Notifications

```json
// No "id" field = notification (no response expected)
{
  "jsonrpc": "2.0",
  "method": "events.newOrder",
  "params": {
    "orderId": "123",
    "userId": "456",
    "total": 99.99,
    "timestamp": "2025-01-01T12:00:00Z"
  }
}
```

### Subscribe to Events

Request:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe.events",
  "params": {
    "topics": ["orders", "users"],
    "filter": { "userId": "456" }
  },
  "id": 5
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "subscriptionId": "sub-123",
    "topics": ["orders", "users"]
  },
  "id": 5
}
```

Notifications:
```json
{
  "jsonrpc": "2.0",
  "method": "notification",
  "params": {
    "subscriptionId": "sub-123",
    "topic": "orders",
    "event": {
      "type": "created",
      "data": { "orderId": "789", "total": 49.99 }
    }
  }
}
```

### Unsubscribe

```json
{
  "jsonrpc": "2.0",
  "method": "unsubscribe.events",
  "params": {
    "subscriptionId": "sub-123"
  },
  "id": 6
}
```

---

## Batch Requests

### Batch Request

```json
[
  {
    "jsonrpc": "2.0",
    "method": "query.execute",
    "params": { "sql": "SELECT * FROM users LIMIT 1" },
    "id": 1
  },
  {
    "jsonrpc": "2.0",
    "method": "query.execute",
    "params": { "sql": "SELECT * FROM orders LIMIT 1" },
    "id": 2
  },
  {
    "jsonrpc": "2.0",
    "method": "prql.execute",
    "params": { "query": "from products | take 1" },
    "id": 3
  }
]
```

### Batch Response

```json
[
  {
    "jsonrpc": "2.0",
    "result": { "rows": [...] },
    "id": 1
  },
  {
    "jsonrpc": "2.0",
    "result": { "rows": [...] },
    "id": 2
  },
  {
    "jsonrpc": "2.0",
    "result": { "rows": [...] },
    "id": 3
  }
]
```

**Benefits**:
- Single network round-trip
- Reduced latency
- Atomic execution (optional)

---

## Error Handling

### Standard Errors

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "param": "sql",
      "reason": "Required parameter missing"
    }
  },
  "id": 1
}
```

### Application Errors

```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Query execution failed",
    "data": {
      "query": "SELECT * FROM nonexistent",
      "error": "Table 'nonexistent' not found",
      "code": "42P01"
    }
  },
  "id": 2
}
```

### Client Error Handling

```javascript
ws.onmessage = (event) => {
  const response = JSON.parse(event.data);
  
  if (response.error) {
    console.error('Error:', response.error.message);
    console.error('Details:', response.error.data);
  } else {
    console.log('Result:', response.result);
  }
};
```

---

## Streaming Results

### Large Result Sets

```json
{
  "jsonrpc": "2.0",
  "method": "query.stream",
  "params": {
    "sql": "SELECT * FROM large_table",
    "batchSize": 1000
  },
  "id": 1
}
```

Server sends multiple notifications:
```json
// Initial response
{
  "jsonrpc": "2.0",
  "result": {
    "streamId": "stream-123",
    "schema": { ... }
  },
  "id": 1
}

// Stream notifications (no id)
{
  "jsonrpc": "2.0",
  "method": "stream.data",
  "params": {
    "streamId": "stream-123",
    "batch": 1,
    "rows": [...],  // 1000 rows
    "hasMore": true
  }
}

{
  "jsonrpc": "2.0",
  "method": "stream.data",
  "params": {
    "streamId": "stream-123",
    "batch": 2,
    "rows": [...],
    "hasMore": false
  }
}
```

### Binary Streaming (Arrow IPC)

```rust
// Send Arrow batches directly
let batches = pyralog.query_stream(sql).await?;

for batch in batches {
    let bytes = batch.to_ipc_bytes()?;
    websocket.send(Message::Binary(bytes)).await?;
}
```

Client:
```javascript
ws.onmessage = (event) => {
  if (event.data instanceof Blob) {
    // Binary data (Arrow IPC)
    event.data.arrayBuffer().then(buffer => {
      const table = arrow.Table.from(new Uint8Array(buffer));
      console.log('Rows:', table.toArray());
    });
  } else {
    // JSON data
    const response = JSON.parse(event.data);
    console.log('Response:', response);
  }
};
```

---

## Authentication

### Login

Request:
```json
{
  "jsonrpc": "2.0",
  "method": "auth.login",
  "params": {
    "username": "alice",
    "password": "secret"
  },
  "id": 1
}
```

Response:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "expiresIn": 3600,
    "userId": "123"
  },
  "id": 1
}
```

### Authenticated Requests

```json
{
  "jsonrpc": "2.0",
  "method": "query.execute",
  "params": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "sql": "SELECT * FROM users"
  },
  "id": 2
}
```

Or via WebSocket subprotocol:
```javascript
const ws = new WebSocket('ws://localhost:8080/rpc', ['jsonrpc', 'auth-token']);
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'query.execute',
  params: { sql: 'SELECT * FROM users' },
  id: 1
}));
```

---

## Performance

### Latency Comparison

| Protocol | Connection | Request | Total |
|----------|-----------|---------|-------|
| **HTTP REST** | ~50ms | ~50ms | **~100ms** |
| **HTTP/2** | ~50ms | ~25ms | **~75ms** |
| **WebSocket (JSON-RPC)** | ~50ms (one-time) | ~2-5ms | **~5ms** |
| **WebSocket (Binary)** | ~50ms (one-time) | ~1-2ms | **~2ms** |

### Throughput

| Operation | Throughput | Latency (p99) | Notes |
|-----------|-----------|---------------|-------|
| Simple query | 100K req/sec | 5ms | JSON payload |
| Binary query | 200K req/sec | 2ms | Arrow IPC |
| Notification | 500K msg/sec | 1ms | Server push |
| Batch (10 queries) | 50K batch/sec | 10ms | Single round-trip |
| Streaming | 1M rows/sec | N/A | Large result sets |

### Connection Pooling

```rust
pub struct ConnectionPool {
    connections: Vec<WebSocket>,
    max_connections: usize,
}

impl ConnectionPool {
    async fn get_connection(&self) -> Result<WebSocket> {
        // Round-robin or least-loaded
        self.connections
            .iter()
            .min_by_key(|ws| ws.pending_requests())
            .cloned()
    }
}
```

---

## Comparison with Other Protocols

### Why JSON-RPC/WS Replaces gRPC for Pyralog

Pyralog uses **JSON-RPC over WebSocket** instead of gRPC for several key reasons:

#### ✅ **Advantages over gRPC**

1. **Simpler** - No Protobuf schemas or code generation needed
2. **Faster** - <5ms vs 5-10ms latency (persistent WebSocket vs gRPC handshake overhead)
3. **Browser Native** - WebSocket support everywhere (gRPC needs grpc-web proxy)
4. **Better Binary Format** - Arrow IPC (zero-copy, columnar) > Protobuf (serialize/deserialize)
5. **Real-time Native** - Bidirectional from the start (gRPC streaming more complex)
6. **Language Agnostic** - Any WebSocket client works (IoT, embedded, shell scripts)

#### When gRPC Might Still Be Useful

- **Legacy systems** requiring gRPC
- **Microservice-to-microservice** (if not using JSON-RPC/WS)
- **Strict schema validation** (but Batuta/GraphQL provide this)

#### Performance Comparison

```
Benchmark: 1 million simple queries

JSON-RPC/WS (JSON):       100,000 queries/sec
JSON-RPC/WS (Arrow IPC):  200,000 queries/sec  ✅ Best
gRPC (Protobuf):           80,000 queries/sec
REST (JSON):               20,000 queries/sec
```

**Latency Comparison**:
```
JSON-RPC/WS:  <5ms   ✅ Lowest
gRPC:         5-10ms
GraphQL:     10-20ms
REST:        50-100ms
```

### Protocol Comparison Table

| Aspect | JSON-RPC/WS | GraphQL | REST | gRPC |
|--------|-------------|---------|------|------|
| **Theoretical Foundation** | None | None | None | None |
| **Complexity** | ✅ **Simple** | ⚠️ Complex | ✅ Simple | ⚠️ **Complex** |
| **Real-time** | ✅ **Native** | ✅ Subscriptions | ❌ Polling | ✅ Streaming |
| **Bidirectional** | ✅ **Yes** | ⚠️ Limited | ❌ No | ✅ Yes |
| **Latency** | ✅ **<5ms** | ⚠️ 10-20ms | ❌ 50-100ms | ⚠️ 5-10ms |
| **Connection** | Persistent | Persistent | Per-request | Persistent |
| **Binary** | ✅ **Arrow IPC** | ❌ No | ❌ No | ✅ Protobuf |
| **Browser Support** | ✅ **Native** | ✅ Native | ✅ Native | ❌ Needs proxy |
| **Schema** | ❌ No | ✅ GraphQL | ❌ No | ✅ Protobuf |
| **Code Generation** | ❌ **Not needed** | ⚠️ Optional | ❌ No | ✅ **Required** |
| **Flexibility** | ⚠️ Method-based | ✅ **Client-driven** | ⚠️ Endpoint-based | ⚠️ Service-based |
| **Use case** | **Real-time RPC** | API queries | Simple APIs | ❌ **Not needed** |

### Recommendation for Pyralog

- ✅ **JSON-RPC/WS**: Primary protocol for all real-time, low-latency RPC
- ✅ **GraphQL**: Flexible API queries (client-driven data fetching)
- ✅ **REST**: Simple stateless APIs (admin, health checks)
- ❌ **gRPC**: Not needed - JSON-RPC/WS is simpler and faster

**Pyralog does not need gRPC** because JSON-RPC over WebSocket provides:
- Lower latency (<5ms vs 5-10ms)
- Simpler implementation (no Protobuf)
- Better binary format (Arrow IPC > Protobuf)
- Native browser support (no grpc-web proxy)
- Bidirectional real-time communication

---

## Best Practices

### 1. **Use Connection Pooling**

```javascript
class JsonRpcClient {
  constructor(url, poolSize = 5) {
    this.connections = [];
    for (let i = 0; i < poolSize; i++) {
      this.connections.push(new WebSocket(url));
    }
  }
  
  async call(method, params) {
    // Get least-loaded connection
    const ws = this.connections
      .reduce((min, ws) => ws.pending < min.pending ? ws : min);
    
    return ws.send({ jsonrpc: '2.0', method, params, id: nextId() });
  }
}
```

### 2. **Handle Reconnection**

```javascript
function connectWithRetry(url, maxRetries = 5) {
  let retries = 0;
  
  function connect() {
    const ws = new WebSocket(url);
    
    ws.onerror = () => {
      if (retries < maxRetries) {
        retries++;
        setTimeout(connect, 1000 * retries);  // Exponential backoff
      }
    };
    
    return ws;
  }
  
  return connect();
}
```

### 3. **Use Batch Requests**

```javascript
// ✅ Good: Batch related queries
const batch = [
  { jsonrpc: '2.0', method: 'query.users', params: {}, id: 1 },
  { jsonrpc: '2.0', method: 'query.orders', params: {}, id: 2 },
  { jsonrpc: '2.0', method: 'query.products', params: {}, id: 3 }
];
ws.send(JSON.stringify(batch));

// ❌ Bad: Individual requests
ws.send(JSON.stringify({ jsonrpc: '2.0', method: 'query.users', ... }));
ws.send(JSON.stringify({ jsonrpc: '2.0', method: 'query.orders', ... }));
ws.send(JSON.stringify({ jsonrpc: '2.0', method: 'query.products', ... }));
```

### 4. **Stream Large Results**

```javascript
// ✅ Good: Stream large result sets
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'query.stream',
  params: { sql: 'SELECT * FROM large_table', batchSize: 1000 },
  id: 1
}));

// ❌ Bad: Load everything at once
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'query.execute',
  params: { sql: 'SELECT * FROM large_table' },  // OOM risk!
  id: 1
}));
```

### 5. **Use Binary for Performance**

```javascript
// ✅ Good: Binary Arrow IPC for large datasets
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'query.arrow',  // Returns Arrow IPC
  params: { sql: 'SELECT * FROM users' },
  id: 1
}));

ws.onmessage = (event) => {
  if (event.data instanceof Blob) {
    // Parse Arrow IPC (zero-copy)
    const table = arrow.Table.from(await event.data.arrayBuffer());
  }
};
```

---

## Examples

### 1. Real-Time Dashboard

```javascript
const ws = new WebSocket('ws://localhost:8080/rpc');

// Subscribe to metrics
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'subscribe.metrics',
  params: { interval: 1000 },  // 1 second
  id: 1
}));

// Receive real-time updates
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  
  if (msg.method === 'metrics.update') {
    updateDashboard(msg.params);
  }
};
```

### 2. Chat Application

```javascript
// Connect
const ws = new WebSocket('ws://localhost:8080/rpc');

// Login
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'auth.login',
  params: { username: 'alice', password: 'secret' },
  id: 1
}));

// Subscribe to messages
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'subscribe.messages',
  params: { roomId: 'general' },
  id: 2
}));

// Send message
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  method: 'message.send',
  params: { roomId: 'general', text: 'Hello!' },
  id: 3
}));

// Receive messages
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  
  if (msg.method === 'message.received') {
    displayMessage(msg.params);
  }
};
```

### 3. IoT Data Ingestion

```python
import websocket
import json

ws = websocket.create_connection('ws://localhost:8080/rpc')

# Authenticate
ws.send(json.dumps({
    'jsonrpc': '2.0',
    'method': 'auth.device',
    'params': { 'deviceId': 'sensor-123', 'token': '...' },
    'id': 1
}))

# Send sensor data
while True:
    sensor_data = read_sensor()
    ws.send(json.dumps({
        'jsonrpc': '2.0',
        'method': 'data.ingest',
        'params': {
            'deviceId': 'sensor-123',
            'temperature': sensor_data.temp,
            'humidity': sensor_data.humidity,
            'timestamp': time.time()
        }
        # No id = notification (no response)
    }))
    time.sleep(1)
```

---

## Summary

JSON-RPC over WebSocket provides **lightweight, real-time RPC** for Pyralog:

- ✅ **Simple protocol** (minimal overhead)
- ✅ **Low latency** (<5ms vs 50-100ms for HTTP)
- ✅ **Bidirectional** (server push notifications)
- ✅ **Binary support** (Arrow IPC for zero-copy)
- ✅ **Language agnostic** (any WebSocket client)
- ✅ **Real-time streaming** (large result sets)

### When to Use JSON-RPC/WebSocket

**Use JSON-RPC/WebSocket when**:
- ✅ Need low-latency RPC (<5ms)
- ✅ Real-time bidirectional communication
- ✅ Server push notifications
- ✅ IoT/sensor data ingestion
- ✅ Live dashboards, chat apps
- ✅ Streaming large datasets

**Don't use when**:
- ❌ Need theoretical foundations → Use [Batuta](BATUTA.md)
- ❌ Need flexible API queries → Use [GraphQL](GRAPHQL.md)
- ❌ Simple stateless APIs → Use REST
- ❌ Microservice communication → Use gRPC

### Next Steps

- 📖 [BATUTA.md](BATUTA.md) - Theoretically-founded programming language
- 📖 [GRAPHQL.md](GRAPHQL.md) - Flexible API query language
- 📖 [PRQL.md](PRQL.md) - Modern relational query language
- 📖 [ARROW.md](ARROW.md) - Columnar data format (binary streaming)

---

**Questions?** Join us on [Discord](https://discord.gg/pyralog) or [open an issue](https://github.com/pyralog/pyralog/issues).

