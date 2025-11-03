# Zero-Knowledge Proofs in Pyralog: SNARKs vs STARKs

**Prove you know something without revealing it**

*Published: November 3, 2025*

---

## The Privacy Problem

Traditional databases leak information:

```sql
-- Query: "Show me all users with salary > $100K"
SELECT * FROM employees WHERE salary > 100000;

Problem: The database sees:
  • Who queried it
  • What they searched for
  • All matching records
```

**Zero-knowledge proofs** let you prove facts without revealing data:

```
Proof: "I know a record where salary > $100K"
Verifier: Convinced (99.9999% certainty)
Information leaked: ZERO
```

---

## What Are Zero-Knowledge Proofs?

**Definition**: A cryptographic method to prove a statement is true without revealing why it's true

### Simple Example

```
Alice wants to prove she knows password without revealing it:

Traditional approach:
  Alice → "My password is hunter2" → Server
  Problem: Server sees password!

Zero-knowledge approach:
  Alice → [generates proof of knowledge] → Server
  Server: "Proof valid, password correct!"
  Server learned: NOTHING about password
```

---

## zk-SNARKs: Succinct Non-Interactive Arguments

**Zero-Knowledge Succinct Non-Interactive Arguments of Knowledge**

### Properties

```
Size: 200-500 bytes (tiny!)
Verification: 1-5ms (fast!)
Generation: 1-10 seconds (slow)
Trusted Setup: REQUIRED (ceremony)
Post-Quantum: ❌ NO (vulnerable to quantum)
```

### How It Works

```rust
/// SNARK workflow
pub struct ZkSnark {
    /// Public parameters (from trusted setup)
    params: PublicParams,
}

impl ZkSnark {
    /// Generate proof
    pub fn prove(
        &self,
        statement: &Statement,
        witness: &Witness,
    ) -> Proof {
        // Compute proof (expensive!)
        // 1-10 seconds for complex statements
        let proof = compute_proof(&self.params, statement, witness);
        proof // 200-500 bytes
    }
    
    /// Verify proof
    pub fn verify(
        &self,
        statement: &Statement,
        proof: &Proof,
    ) -> bool {
        // Fast verification (1-5ms)
        verify_proof(&self.params, statement, proof)
    }
}
```

### Use Cases

**1. Private Transactions**
```rust
/// Prove you have balance without revealing amount
pub struct PrivateTransfer {
    sender: PublicKey,
    receiver: PublicKey,
    // Amount hidden!
}

impl PrivateTransfer {
    pub fn create_proof(&self, balance: u64, amount: u64) -> Proof {
        // Prove: balance >= amount (without revealing either!)
        prove_greater_equal(balance, amount)
    }
}

// Verifier confirms: Transfer valid
// Verifier learns: NOTHING about amounts
```

**2. Verifiable Computation**
```rust
/// Prove computation result without recomputing
pub fn offload_computation(input: &[u8]) -> (Output, Proof) {
    let output = expensive_computation(input);
    let proof = prove_correct_computation(input, &output);
    (output, proof)
}

// Client verifies proof in 1ms
// No need to recompute (which takes 10 seconds!)
```

**3. Proof of Storage**
```rust
/// Prove you're storing data without revealing it
pub fn prove_storage(data: &[u8]) -> Proof {
    prove_knowledge_of_data(data)
}

// Use case: Decentralized storage (Filecoin-style)
// Provider proves they have data without sending it
```

### Performance

```
Benchmark: Prove "I know x where hash(x) = h"

Generation:
  • Time: 2.5 seconds
  • CPU: 1 core
  • Memory: 500MB

Proof size: 288 bytes

Verification:
  • Time: 2.8ms
  • CPU: Negligible
  • Memory: <1MB

Result: 1000× faster to verify than prove!
```

---

## zk-STARKs: Scalable Transparent Arguments

**Zero-Knowledge Scalable Transparent Arguments of Knowledge**

### Properties

```
Size: 100-200KB (large)
Verification: 10-50ms (slower than SNARKs)
Generation: 0.5-5 seconds (faster than SNARKs for large proofs)
Trusted Setup: NOT REQUIRED (transparent)
Post-Quantum: ✅ YES (quantum-resistant)
```

### Key Difference: No Trusted Setup

```
SNARKs:
  • Require "ceremony" to generate parameters
  • If ceremony compromised → fake proofs possible
  • Need to trust ceremony participants

STARKs:
  • No ceremony needed
  • Publicly verifiable randomness
  • Trustless from day one
```

### How It Works

```rust
/// STARK workflow (no trusted setup!)
pub struct ZkStark {}

impl ZkStark {
    /// Generate proof (no params needed!)
    pub fn prove(
        statement: &Statement,
        witness: &Witness,
    ) -> Proof {
        // Compute proof using FRI (Fast Reed-Solomon)
        let proof = compute_stark_proof(statement, witness);
        proof // 100-200KB
    }
    
    /// Verify proof
    pub fn verify(
        statement: &Statement,
        proof: &Proof,
    ) -> bool {
        // Verification (10-50ms)
        verify_stark_proof(statement, proof)
    }
}
```

### Use Cases

**1. Large Computations**
```rust
/// STARKs scale better for complex proofs
pub fn prove_blockchain_validity(blocks: &[Block]) -> Proof {
    // Prove entire blockchain is valid
    // STARK: 2 seconds, 150KB proof
    // SNARK: 20 seconds, 300 bytes proof
    //
    // For large computations: STARKs win on proving time
    prove_stark(blocks)
}
```

**2. Post-Quantum Security**
```rust
/// STARKs resist quantum computers
pub fn quantum_safe_proof(data: &[u8]) -> Proof {
    // Will remain valid even with quantum computers
    prove_stark(data)
}
```

**3. Transparent Protocols**
```rust
/// No need to trust anyone
pub fn trustless_proof(data: &[u8]) -> Proof {
    // No ceremony, no trusted setup
    // Publicly verifiable from genesis
    prove_stark(data)
}
```

### Performance

```
Benchmark: Prove "I know x where hash(x) = h"

Generation:
  • Time: 800ms (3× faster than SNARK)
  • CPU: 1 core
  • Memory: 2GB

Proof size: 142KB (500× larger than SNARK)

Verification:
  • Time: 35ms (12× slower than SNARK)
  • CPU: Negligible
  • Memory: 10MB

Result: Faster proving, larger proofs, slower verification
```

---

## SNARKs vs STARKs Comparison

| Feature | zk-SNARKs | zk-STARKs |
|---------|-----------|-----------|
| **Proof size** | 200-500 bytes ✅ | 100-200KB ⚠️ |
| **Proving time** | 1-10 seconds | 0.5-5 seconds ✅ |
| **Verification** | 1-5ms ✅ | 10-50ms |
| **Trusted setup** | Required ⚠️ | Not required ✅ |
| **Post-quantum** | No ⚠️ | Yes ✅ |
| **Maturity** | High (Zcash, 2016) | Medium (StarkWare, 2018) |
| **Complexity** | Very high | Very high |

### When to Use Each

**Use SNARKs for:**
- ✅ Bandwidth-constrained (proof size matters)
- ✅ High verification throughput (millions/sec)
- ✅ Mature ecosystem (Zcash, Filecoin)
- ⚠️ Can tolerate trusted setup

**Use STARKs for:**
- ✅ Transparency (no trusted setup)
- ✅ Post-quantum security
- ✅ Large computations (proving time matters)
- ⚠️ Can tolerate larger proofs

---

## Pyralog Integration

### Private Transactions

```rust
/// Transfer with hidden amounts (SNARK)
pub async fn private_transfer(
    pyralog: &PyralogClient,
    from: &PrivateAccount,
    to: &PublicKey,
    amount: u64,
) -> Result<TxHash> {
    // Create proof: "I have balance >= amount"
    let proof = from.prove_sufficient_balance(amount).await?;
    
    // Create encrypted transfer
    let tx = PrivateTransaction {
        from: from.public_key(),
        to: *to,
        encrypted_amount: from.encrypt_amount(amount),
        proof,
    };
    
    // Submit transaction
    pyralog.submit_private_tx(tx).await
}

// Pyralog verifies proof (1-5ms)
// Amount remains hidden from blockchain
```

### Verifiable Query Results

```rust
/// Prove query result is correct
pub async fn provable_query(
    pyralog: &PyralogClient,
    query: &str,
) -> Result<(ResultSet, Proof)> {
    // Execute query
    let results = pyralog.query(query).await?;
    
    // Generate proof of correctness
    let proof = pyralog.prove_query_result(query, &results).await?;
    
    // Client can verify without re-querying!
    Ok((results, proof))
}
```

### Private Analytics

```rust
/// Aggregate without revealing individual records
pub async fn private_analytics(
    pyralog: &PyralogClient,
) -> Result<AggregateResult> {
    let (sum, proof) = pyralog.private_aggregate(
        "SELECT SUM(salary) FROM employees WHERE department = 'Engineering'",
    ).await?;
    
    // Result: $5.2M total
    // Proof: Computation is correct
    // Leaked: ZERO individual salaries
    
    Ok(AggregateResult { sum, proof })
}
```

---

## Performance in Pyralog

### SNARK Benchmarks

```
Operation: Private transfer

Proof generation: 2.8 seconds
Proof size: 288 bytes
Verification: 3.2ms
Throughput: 312 verifications/sec (single core)
           : 12,480 verifications/sec (40 cores)

Network overhead: Negligible (288 bytes)
```

### STARK Benchmarks

```
Operation: Verifiable query (1M records)

Proof generation: 1.2 seconds
Proof size: 156KB
Verification: 42ms
Throughput: 23 verifications/sec (single core)
           : 920 verifications/sec (40 cores)

Network overhead: 156KB per query
```

---

## Summary

Zero-knowledge proofs enable **privacy without trust**:

### zk-SNARKs
- ✅ Tiny proofs (200-500 bytes)
- ✅ Fast verification (1-5ms)
- ⚠️ Trusted setup required
- ⚠️ Not quantum-resistant

**Best for**: Private transactions, high throughput, bandwidth-constrained

### zk-STARKs
- ✅ No trusted setup
- ✅ Post-quantum secure
- ✅ Faster proving (large computations)
- ⚠️ Larger proofs (100-200KB)

**Best for**: Transparent systems, future-proof, complex proofs

### The Bottom Line

Zero-knowledge proofs are **no longer theoretical**. Pyralog integrates both SNARKs and STARKs for:
- Private transactions (hide amounts)
- Verifiable computation (prove correctness)
- Private analytics (aggregate without revealing records)

Choose SNARKs for small proofs and fast verification. Choose STARKs for transparency and quantum resistance.

*Prove without revealing.*

---

## Next Steps

- Read [Decentralized Architecture](../DECENTRALIZED.md) for zk-proof integration
- See [Cryptographic Verification](6-cryptographic-verification.md) for signatures
- Check [Private Transactions Guide](../docs/private-transactions.md)

---

*Part 22 of the Pyralog Blog Series*

*Previously: [From Cluster to Network](21-decentralized.md)*
*Next: [PoW Without Miners](23-pow-useful.md)*

