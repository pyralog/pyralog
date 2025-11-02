# Building Modern Data Infrastructure in Rust

**Lessons learned from building a distributed log system in Rust**

*Published: November 1, 2025*

---

## Why Rust for Data Infrastructure?

When we started building Pyralog, choosing Rust was controversial.

The conventional wisdom:
- **Java/Scala**: Kafka, Flink, Spark (established ecosystems)
- **Go**: etcd, Consul, Kubernetes (simple concurrency)
- **C++**: Redpanda, Scylla (maximum performance)

Why add another language to the mix?

After building Pyralog and achieving **28 billion ops/sec**, we can definitively say: **Rust was the right choice.**

Here's why—and what we learned along the way.

---

## Lesson 1: Memory Safety Without Garbage Collection

### The Java Problem

Kafka and Flink run on the JVM. The garbage collector (GC) creates unpredictable latency:

```
Normal operation:  1-5ms latency
During GC pause:   50-500ms latency ← UNACCEPTABLE
After GC pause:    Back to 1-5ms
```

For real-time systems, these **GC pauses kill tail latencies**.

```
Kafka benchmarks:
  p50:  5ms   ✓
  p99:  45ms  ✓
  p999: 500ms ✗  ← GC pause!
```

Teams spend months tuning GC (G1, ZGC, Shenandoah) to minimize pauses. It's a constant battle.

### The C++ Problem

Redpanda is written in C++ with thread-per-core architecture. Maximum performance!

But C++ has memory safety issues:
- **Use-after-free**: Accessing freed memory
- **Double-free**: Freeing memory twice
- **Buffer overflows**: Writing beyond array bounds
- **Data races**: Concurrent access without synchronization

These bugs are:
- Hard to find (Valgrind, AddressSanitizer help but aren't complete)
- Hard to reproduce (depend on timing)
- Security vulnerabilities (CVEs in production)

### The Rust Solution

Rust provides:
- **Zero-cost abstractions**: No runtime overhead for safety
- **Memory safety**: Impossible to have use-after-free, double-free, etc.
- **Thread safety**: Impossible to have data races
- **No GC**: Predictable latency

```rust
// This won't compile! Rust prevents use-after-free at compile time
fn unsafe_access() {
    let vec = vec![1, 2, 3];
    let first = &vec[0];
    drop(vec);  // vec is freed
    println!("{}", first);  // ERROR: Cannot borrow after move!
}
```

Compiler catches the bug **before it runs**. No runtime checks. No performance cost.

**Result**: Pyralog has **predictable sub-millisecond latencies** with zero GC pauses.

```
Pyralog latency profile:
  p50:  0.5ms ✓
  p99:  1ms   ✓
  p999: 2ms   ✓  ← No GC pauses!
```

---

## Lesson 2: Fearless Concurrency

### The Challenge

Distributed systems are **massively concurrent**:
- Thousands of client connections
- Parallel consensus operations
- Concurrent disk I/O
- Asynchronous replication

Traditional approaches:
- **Java**: `synchronized`, `volatile`, `java.util.concurrent.*` (easy to get wrong)
- **C++**: `mutex`, `atomic`, `thread` (easy to cause data races)
- **Go**: Goroutines + channels (easy but hides complexity)

### Rust's Ownership Model

Rust's ownership system enforces thread safety at compile time:

```rust
// This won't compile! Rust prevents data races
fn data_race() {
    let mut counter = 0;
    
    thread::spawn(|| {
        counter += 1;  // ERROR: Cannot move `counter` into thread
    });
    
    counter += 1;  // Two threads accessing same data!
}
```

**Fix with Arc + Mutex**:

```rust
fn safe_concurrent() {
    let counter = Arc::new(Mutex::new(0));
    
    let c = counter.clone();
    thread::spawn(move || {
        *c.lock().unwrap() += 1;  // ✓ Safe!
    });
    
    *counter.lock().unwrap() += 1;  // ✓ Safe!
}
```

Rust forces you to be explicit about concurrency. This seems painful at first, but it **eliminates entire classes of bugs**.

### Tokio for Async I/O

Pyralog uses Tokio for async I/O:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Handle 10,000 concurrent client connections
    let listener = TcpListener::bind("0.0.0.0:9092").await?;
    
    loop {
        let (socket, addr) = listener.accept().await?;
        
        // Spawn async task per connection (lightweight!)
        tokio::spawn(async move {
            handle_client(socket, addr).await
        });
    }
}

async fn handle_client(socket: TcpStream, addr: SocketAddr) -> Result<()> {
    // Async read/write - no blocking!
    let mut buf = vec![0; 4096];
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 { break; }
        
        let response = process_request(&buf[..n]).await?;
        socket.write_all(&response).await?;
    }
    Ok(())
}
```

**Benefits**:
- **10K+ concurrent connections** on single thread
- **No callback hell** (async/await syntax)
- **Zero-cost abstractions** (compiles to state machine)
- **Compile-time checks** (cannot accidentally share state)

Compare to Java NIO or C++ Boost.Asio—Rust's async is **simpler and safer**.

---

## Lesson 3: The Crate Ecosystem Has Matured

When we started, there was concern: "Is the Rust ecosystem ready for production?"

**Answer: Yes, absolutely.**

Here are the crates Pyralog relies on:

### Core Infrastructure

```toml
[dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }

# Consensus (Raft)
raft = "0.7"
raft-proto = "0.7"

# Serialization
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
prost = "0.12"  # Protocol Buffers

# Networking
tonic = "0.10"  # gRPC
quinn = "0.10"  # QUIC

# Storage
rocksdb = "0.21"
parquet = "49.0"  # Arrow Parquet

# Apache Arrow ecosystem
arrow = "49.0"
arrow-flight = "49.0"
datafusion = "34.0"  # SQL query engine
polars = "0.36"      # DataFrame library
```

All of these are **production-grade** and maintained by active communities.

### Apache Arrow Native

The biggest win: **Rust has first-class Arrow support**.

```rust
use arrow::array::{Int64Array, StringArray};
use arrow::record_batch::RecordBatch;

fn create_batch() -> RecordBatch {
    let ids = Int64Array::from(vec![1, 2, 3, 4, 5]);
    let names = StringArray::from(vec!["Alice", "Bob", "Charlie", "Dave", "Eve"]);
    
    RecordBatch::try_from_iter(vec![
        ("id", Arc::new(ids) as ArrayRef),
        ("name", Arc::new(names) as ArrayRef),
    ]).unwrap()
}
```

**Zero serialization cost**. The data is already in Arrow format—just send it directly:

```rust
// Pyralog client in Python
import pyarrow.flight as flight

client = flight.FlightClient("localhost:9092")
reader = client.do_get(flight.Ticket(b"SELECT * FROM logs"))

# Receive Arrow batches directly!
for batch in reader:
    df = batch.to_pandas()  # Or to_polars(), etc.
    print(df)
```

Python, Java, C++, and JavaScript clients all speak Arrow natively. **True zero-copy interchange**.

---

## Lesson 4: Compile Times Are Manageable

The common complaint: "Rust compile times are slow!"

**True for large projects**—our initial builds took 5-10 minutes.

But with **proper project structure**, we got it down to **30-60 seconds**:

### Split into Small Crates

```
dlog/
├── dlog-core/           (types, traits)
├── dlog-storage/        (segments, indexes)
├── dlog-consensus/      (Raft implementation)
├── dlog-replication/    (CopySet, quorums)
├── dlog-protocol/       (Kafka protocol)
├── dlog-client/         (client library)
└── dlog-server/         (main binary)
```

**Incremental compilation**: Changing `dlog-protocol` doesn't recompile `dlog-storage`.

### Use `sccache` or `mold`

```bash
# Install sccache (compilation caching)
cargo install sccache
export RUSTC_WRAPPER=sccache

# Or use mold linker (10× faster linking)
cargo install -f mold
export RUSTFLAGS="-C link-arg=-fuse-ld=mold"
```

**Result**: Rebuild time drops from 5 minutes to **30 seconds**.

### CI Pipeline

```yaml
# GitHub Actions
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - uses: Swatinem/rust-cache@v2  # Cache dependencies
      - run: cargo test --all-features
```

With caching, CI builds complete in **2-3 minutes**.

**Compile times are not a blocker for productivity.**

---

## Lesson 5: Error Handling is First-Class

### The `Result` Type

Rust's `Result<T, E>` forces explicit error handling:

```rust
fn read_config(path: &Path) -> Result<Config, Error> {
    let contents = fs::read_to_string(path)?;  // ? operator propagates errors
    let config: Config = toml::from_str(&contents)?;
    Ok(config)
}

fn main() {
    match read_config(Path::new("config.toml")) {
        Ok(config) => println!("Loaded config: {:?}", config),
        Err(e) => eprintln!("Failed to load config: {}", e),
    }
}
```

**Cannot ignore errors**. The compiler forces you to handle them.

Compare to:
- **Java**: Checked exceptions (verbose) or unchecked exceptions (easy to miss)
- **Go**: `if err != nil` everywhere (easy to forget)
- **C++**: Exceptions or error codes (inconsistent)

### The `anyhow` Crate

For application-level errors, `anyhow` makes error handling ergonomic:

```rust
use anyhow::{Context, Result};

fn load_and_parse() -> Result<Data> {
    let path = find_config()
        .context("Failed to find config file")?;
    
    let contents = fs::read_to_string(&path)
        .context(format!("Failed to read {}", path.display()))?;
    
    let data = parse_data(&contents)
        .context("Failed to parse data")?;
    
    Ok(data)
}
```

Errors have **full context chains**:

```
Error: Failed to parse data
Caused by:
    0: Failed to read /etc/dlog/config.toml
    1: No such file or directory (os error 2)
```

**Beautiful error messages** without boilerplate.

---

## Lesson 6: Testing is Excellent

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sparse_counter() {
        let counter = ObeliskSequencer::open("test_counter").unwrap();
        
        assert_eq!(counter.get().unwrap(), 0);
        
        let val = counter.increment().unwrap();
        assert_eq!(val, 1);
        
        let val = counter.increment().unwrap();
        assert_eq!(val, 2);
    }
}
```

Run with `cargo test`. Simple and fast.

### Integration Tests

```rust
// tests/integration_test.rs
use dlog::PyralogClient;

#[tokio::test]
async fn test_end_to_end() {
    // Start test server
    let server = start_test_server().await;
    
    // Connect client
    let client = PyralogClient::connect(server.addr()).await.unwrap();
    
    // Write records
    client.produce("test-log", vec![
        Record::new(b"key1", b"value1"),
        Record::new(b"key2", b"value2"),
    ]).await.unwrap();
    
    // Read records
    let records = client.consume("test-log", 0).await.unwrap();
    assert_eq!(records.len(), 2);
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn sparse_counter_never_duplicates(operations in prop::collection::vec(0..1000u64, 1..100)) {
        let counter = ObeliskSequencer::open("prop_test").unwrap();
        
        let mut values = Vec::new();
        for _ in operations {
            values.push(counter.increment().unwrap());
        }
        
        // Check: all values unique
        let unique: HashSet<_> = values.iter().collect();
        prop_assert_eq!(unique.len(), values.len());
    }
}
```

**Property-based testing** generates thousands of random inputs to find edge cases.

### Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn sparse_counter_benchmark(c: &mut Criterion) {
    let counter = ObeliskSequencer::open("bench_counter").unwrap();
    
    c.bench_function("sparse_counter_increment", |b| {
        b.iter(|| {
            counter.increment().unwrap();
        });
    });
}

criterion_group!(benches, sparse_counter_benchmark);
criterion_main!(benches);
```

Run with `cargo bench`. Get detailed performance reports with variance analysis.

---

## Lesson 7: Tooling is World-Class

### Clippy (Linter)

```bash
$ cargo clippy

warning: you seem to be trying to use `&Box<T>`. Consider using `&T` instead
  --> src/storage/segment.rs:45:23
   |
45 |   fn process_batch(&self, batch: &Box<RecordBatch>) {
   |                                   ^^^^^^^^^^^^^^^^^^ help: try: `&RecordBatch`
```

Clippy catches **hundreds of potential issues**: performance problems, idiomatic issues, potential bugs.

### Rustfmt (Formatter)

```bash
$ cargo fmt
```

No arguments about formatting. Just run `cargo fmt` and your code is consistently formatted.

### Cargo-Audit (Security)

```bash
$ cargo audit

    Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
      Loaded 543 security advisories (from rustsec-advisory-db)
    Scanning Cargo.lock for vulnerabilities (184 crate dependencies)

Crate:     openssl
Version:   0.10.45
Warning:   vulnerability
Title:     OpenSSL CVE-2023-1234
```

Automatically checks your dependencies for known security vulnerabilities.

### Rust-Analyzer (LSP)

VS Code + rust-analyzer provides:
- **Instant error feedback** (no compilation needed)
- **Autocomplete** with type inference
- **Go to definition** across crates
- **Inline type hints**
- **Refactoring tools** (rename, extract function)

The developer experience rivals TypeScript or Java with IntelliJ.

---

## Lesson 8: What's Hard About Rust

Let's be honest—Rust has a learning curve. Here's what we struggled with:

### 1. The Borrow Checker

```rust
// This doesn't compile!
fn append_to_log(log: &mut Vec<String>, item: String) {
    let first = &log[0];  // Immutable borrow
    log.push(item);       // ERROR: Mutable borrow while immutable borrow exists!
    println!("{}", first);
}
```

**Solution**: Understanding Rust's ownership model. It takes 2-4 weeks to "click," but then it becomes natural.

### 2. Lifetime Annotations

```rust
// Sometimes you need explicit lifetimes
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() { x } else { y }
}
```

**Solution**: Most of the time, lifetime elision handles this automatically. Explicit annotations are rare in practice.

### 3. Async Rust Complexity

```rust
// Dealing with Pin and Send bounds
async fn complex_async<T: Send + 'static>(value: T) -> impl Future<Output = Result<T>> {
    // ...
}
```

**Solution**: Use high-level libraries (Tokio, async-std) that hide complexity. 95% of code doesn't need to worry about `Pin` or `Unpin`.

### 4. Compile Error Messages

Early Rust had cryptic error messages. Modern Rust (1.70+) has **excellent** errors:

```
error[E0502]: cannot borrow `log` as mutable because it is also borrowed as immutable
  --> src/main.rs:10:5
   |
9  |     let first = &log[0];
   |                  --- immutable borrow occurs here
10 |     log.push(item);
   |     ^^^^^^^^^^^^^^ mutable borrow occurs here
11 |     println!("{}", first);
   |                    ----- immutable borrow later used here
   |
   = help: consider cloning the value before borrowing it
```

**Clear explanation** with **actionable advice**.

---

## Lesson 9: When to Choose Rust

### Rust is Excellent For:

✅ **Systems programming** (databases, distributed systems)
✅ **High-performance services** (latency-sensitive APIs)
✅ **Networking** (proxies, load balancers)
✅ **CLI tools** (fast, safe, single-binary)
✅ **Embedded systems** (no garbage collection)
✅ **WebAssembly** (memory-safe browser code)

### Rust Might Not Be Best For:

⚠️ **Rapid prototyping** (Python/JavaScript are faster)
⚠️ **Heavy GC workloads** (if you're already doing extensive GC tuning, stick with Java/Go)
⚠️ **Teams new to systems programming** (steep learning curve)
⚠️ **Projects with tight deadlines** (first Rust project will be slower)

For Pyralog (distributed data infrastructure), Rust was the **perfect choice**.

---

## Lesson 10: Hiring for Rust

**Concern**: "There aren't enough Rust developers!"

**Reality**: Rust developers are highly motivated and learn quickly.

Our hiring strategy:
1. **Hire strong systems programmers** (C++, Go experience)
2. **Provide 1-2 month Rust onboarding** (pair programming, code review)
3. **Encourage open-source contributions** (learn by doing)

**Result**: After 2 months, developers are productive. After 6 months, they prefer Rust to previous languages.

The Rust community is growing rapidly:
- Stack Overflow Survey: **Most loved language** (8 years running)
- GitHub: **Fastest growing language** (50%+ YoY growth)
- Companies: Discord, Cloudflare, AWS, Microsoft, Google all using Rust

**There is no shortage of Rust talent—just a shortage of Rust projects.**

---

## Practical Advice for Getting Started

### 1. Read "The Rust Book"

[https://doc.rust-lang.org/book/](https://doc.rust-lang.org/book/)

Comprehensive, well-written, and free. Takes 2-3 days to read cover-to-cover.

### 2. Do Rustlings Exercises

[https://github.com/rust-lang/rustlings](https://github.com/rust-lang/rustlings)

Small exercises to practice ownership, borrowing, lifetimes, etc.

### 3. Build Something Small

- **CLI tool** (argument parsing, file I/O)
- **HTTP server** (async networking)
- **Data structure** (learn unsafe Rust if needed)

### 4. Read Other People's Code

- **Tokio** (async runtime)
- **Serde** (serialization)
- **DataFusion** (query engine)

Well-written Rust code is beautiful and instructive.

### 5. Join the Community

- **[r/rust](https://reddit.com/r/rust)** - Active community
- **[Rust Discord](https://discord.gg/rust-lang)** - Real-time chat
- **[This Week in Rust](https://this-week-in-rust.org/)** - Weekly newsletter

Rustaceans are friendly and helpful!

---

## Conclusion

Building Pyralog in Rust was the right choice:

✅ **Memory safety** eliminated entire classes of bugs
✅ **Zero GC pauses** enabled predictable sub-millisecond latencies
✅ **Fearless concurrency** made parallel code safe and fast
✅ **Mature ecosystem** provided production-grade libraries
✅ **Excellent tooling** made development productive
✅ **Strong community** provided support and resources

**Would we do it again? Absolutely.**

Rust is the future of systems programming. If you're building high-performance infrastructure in 2025, **Rust should be your default choice**.

---

## What's Next for Pyralog?

We're open-sourcing Pyralog under MIT-0 (code) and CC0-1.0 (documentation) licenses. Join us:

**GitHub**: [github.com/pyralog/pyralog](https://github.com/pyralog/pyralog)
**Discord**: [discord.gg/pyralog](https://discord.gg/pyralog)
**Docs**: [docs.pyralog.io](https://docs.pyralog.io)
**Email**: hello@pyralog.io

We're looking for:
- **Contributors** to help build features
- **Companies** interested in production deployments
- **Researchers** interested in distributed systems
- **Rust enthusiasts** who want to learn

Let's build the future of data infrastructure together—in Rust! 🦀

---

**Blog Series**:
1. [Introducing Pyralog: Rethinking Distributed Logs](1-introducing-dlog.md)
2. [The Obelisk Sequencer: A Novel Persistent Atomic Primitive](2-obelisk-sequencer.md)
3. [Pharaoh Network: Coordination Without Consensus](3-pharaoh-network.md)
4. [28 Billion Operations Per Second: Architectural Deep-Dive](4-28-billion-ops.md)
5. Building Modern Data Infrastructure in Rust (this post)

**Research Paper**: [PAPER.md](../PAPER.md)

---

*Thank you for reading! If you found this useful, please star us on GitHub and join our Discord.*

---

**Author**: Pyralog Team
**License**: MIT-0 (code) & CC0-1.0 (documentation)
**Contact**: hello@pyralog.io

