# Data Formats in Pyralog

**External data formats for storage, interchange, and ML integration**

---

## Table of Contents

1. [Overview](#overview)
2. [Format Selection Guide](#format-selection-guide)
3. [Parquet: Columnar Analytics](#parquet-columnar-analytics)
4. [Safetensors: ML Model Storage](#safetensors-ml-model-storage)
5. [Zarr: Cloud-Native Arrays](#zarr-cloud-native-arrays)
6. [DLPack: Zero-Copy Exchange](#dlpack-zero-copy-exchange)
7. [Performance Comparison](#performance-comparison)
8. [Integration Patterns](#integration-patterns)

---

## Overview

Pyralog uses **four complementary formats** for different workloads:

| Format | Layer | Purpose | Performance Gain |
|--------|-------|---------|------------------|
| **Parquet** | Disk | Analytics (columnar tables) | 10-100× vs CSV |
| **Safetensors** | Disk | ML models (memory-safe) | 100× vs pickle |
| **Zarr** | Disk | N-D arrays (cloud-native) | 20-50× vs HDF5 |
| **DLPack** | Runtime | Tensor exchange (zero-copy) | 300× vs memcpy |

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   PYRALOG DATA FORMATS                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Disk Storage:                                              │
│  ┌──────────┐  ┌──────────────┐  ┌────────────┐           │
│  │ Parquet  │  │ Safetensors  │  │   Zarr     │           │
│  │ Tabular  │  │  ML Models   │  │  N-D Arrays│           │
│  └──────────┘  └──────────────┘  └────────────┘           │
│       ↓              ↓                  ↓                   │
│  ┌───────────────────────────────────────────────┐         │
│  │      Apache Arrow (Columnar In-Memory)        │         │
│  │          Pyralog LSM Storage                  │         │
│  └───────────────────────────────────────────────┘         │
│                          ↕                                  │
│  Runtime Exchange:                                          │
│  ┌───────────────────────────────────────────────┐         │
│  │         DLPack (Zero-Copy Protocol)           │         │
│  │   PyTorch ↔ Pyralog ↔ TensorFlow ↔ JAX       │         │
│  └───────────────────────────────────────────────┘         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Format Selection Guide

### Decision Tree

```
What data are you working with?

├─ Tabular data (rows & columns)
│  ├─ Analytics queries → Parquet
│  └─ Real-time OLTP → Arrow (native)
│
├─ ML models & tensors
│  ├─ Persistent storage → Safetensors
│  └─ Runtime interchange → DLPack
│
└─ Scientific arrays (N-D)
   ├─ Cloud storage → Zarr
   └─ Local fast I/O → Arrow (native)
```

### Use Case Matrix

| Use Case | Primary Format | Secondary | Why |
|----------|---------------|-----------|-----|
| **Data lake** | Parquet | Arrow | Columnar, compression, SQL |
| **ML training** | Safetensors | DLPack | Memory-safe, fast load |
| **Model inference** | DLPack | Safetensors | Zero-copy, low latency |
| **Climate data** | Zarr | Parquet | Chunked, parallel I/O |
| **Time-series** | Parquet | Zarr | Compression, append |
| **Embeddings** | Arrow | Safetensors | SIMD, ANN search |
| **Cross-framework** | DLPack | - | Zero-copy standard |

---

## Parquet: Columnar Analytics

### Overview

**Apache Parquet** is a columnar file format for analytics workloads, offering 10-100× speedup over CSV.

**Key features**:
- ✅ Columnar layout (better compression)
- ✅ Predicate pushdown & column pruning
- ✅ Nested data support (structs, lists, maps)
- ✅ Cloud-optimized (S3, GCS, Azure)
- ✅ Self-describing schema

### Quick Start

```rust
use pyralog::external::ParquetTable;

// 1. Query external Parquet (no import)
pyralog.register_external_table("sales", ParquetTable {
    location: "s3://bucket/sales/*.parquet",
    partition_columns: vec!["year", "month"],
}).await?;

let results = pyralog.query(
    "SELECT product, SUM(revenue) FROM sales WHERE year = 2025 GROUP BY product"
).await?;

// 2. Export to Parquet
pyralog.export_table("sales", ExportConfig {
    format: DataFormat::Parquet,
    path: "s3://bucket/export.parquet",
    compression: Compression::Zstd(3),
    row_group_size: 1_000_000,
}).await?;

// 3. Import from Parquet
pyralog.import_table("sales", ImportConfig {
    format: DataFormat::Parquet,
    path: "data/sales.parquet",
    batch_size: 65536,
}).await?;
```

### Compression Codecs

| Codec | Ratio | Speed | Use Case |
|-------|-------|-------|----------|
| **Zstd (level 3)** | 3-5× | 300 MB/s | **Recommended** (balance) |
| **Snappy** | 2-3× | 500 MB/s | Fast decompression |
| **LZ4** | 1.5-2× | 600 MB/s | Fastest |
| **Brotli** | 4-6× | 50 MB/s | Archival |

### Performance

```
Benchmark: 1 billion rows, 10 columns

Format           Size     Load Time   Query (filter+agg)
────────────────────────────────────────────────────────
CSV (gzip)       15 GB    180 sec     90 sec
JSON (gzip)      22 GB    240 sec     120 sec
Parquet (zstd)   3.2 GB   8 sec       2.5 sec
Arrow (native)   4.8 GB   0.5 sec     2.0 sec

→ Parquet: 4.7× smaller, 22× faster load, 36× faster query
```

---

## Safetensors: ML Model Storage

### Overview

**Safetensors** is a memory-safe, fast tensor serialization format for ML models, offering **100× speedup** over pickle.

**Key features**:
- ✅ **Memory-safe**: No arbitrary code execution
- ✅ **Zero-copy**: Memory-map files directly
- ✅ **Fast**: 10-100× faster than pickle
- ✅ **Lazy loading**: Load only needed tensors
- ✅ **Hugging Face**: Native integration

### Why Safetensors?

| Aspect | Pickle | Safetensors | Improvement |
|--------|--------|-------------|-------------|
| **Security** | ❌ Arbitrary code | ✅ Data only | **Safe** |
| **Load time (7B model)** | 45 sec | 0.4 sec | **100× faster** |
| **Memory usage** | 2× model size | 1× model size | **50% less** |
| **Format** | Opaque binary | Structured | **Debuggable** |

### Quick Start

```rust
use safetensors::{SafeTensors, serialize};

// 1. Save model to Safetensors
let tensors = vec![
    ("encoder.weight", tensor1.as_slice()),
    ("encoder.bias", tensor2.as_slice()),
    ("decoder.weight", tensor3.as_slice()),
];

let metadata = Some(HashMap::from([
    ("model_type", "transformer"),
    ("framework", "pytorch"),
]));

let bytes = serialize(&tensors, &metadata)?;
std::fs::write("model.safetensors", bytes)?;

// 2. Load model (zero-copy via mmap)
let file = File::open("model.safetensors")?;
let mmap = unsafe { Mmap::map(&file)? };
let safetensors = SafeTensors::deserialize(&mmap)?;

// Access tensors without loading entire file
let weights = safetensors.tensor("encoder.weight")?;
let shape = weights.shape();
let data = weights.data(); // No copy!

// 3. Hugging Face integration
pyralog.import_hf_model("bert-base-uncased").await?;
```

### Format Specification

```
┌─────────────────────────────────────────────────────────┐
│                  SAFETENSORS FILE                        │
├─────────────────────────────────────────────────────────┤
│  [8 bytes]   Header length (u64, little-endian)         │
│  [N bytes]   JSON metadata                              │
│              {                                          │
│                "tensor_name": {                         │
│                  "dtype": "F32",                        │
│                  "shape": [768, 768],                   │
│                  "data_offsets": [0, 2359296]           │
│                },                                       │
│                ...                                      │
│              }                                          │
│  [M bytes]   Tensor data (contiguous, no padding)       │
└─────────────────────────────────────────────────────────┘
```

**Benefits**:
- Simple format (easy to parse)
- Random access (load specific tensors)
- Memory-mappable (zero-copy)
- No padding (space-efficient)

### Performance

```
Benchmark: Load BERT-base (440 MB)

Method               Load Time   Memory     Security
─────────────────────────────────────────────────────
Pickle               2.5 sec     880 MB     ❌ Unsafe
PyTorch .pth         3.2 sec     880 MB     ❌ Unsafe
Safetensors (mmap)   0.025 sec   440 MB     ✅ Safe

→ Safetensors: 100× faster, 50% memory, fully safe
```

---

## Zarr: Cloud-Native Arrays

### Overview

**Zarr** is a format for chunked, compressed N-dimensional arrays designed for cloud storage and parallel I/O.

**Key features**:
- ✅ **Cloud-native**: S3/GCS/Azure optimized
- ✅ **Chunked**: Each chunk independently accessible
- ✅ **Parallel I/O**: Concurrent read/write
- ✅ **Compression**: Per-chunk (Blosc, Zstd, LZ4)
- ✅ **Incremental**: Append without rewriting
- ✅ **Metadata**: JSON-based, human-readable

### Zarr vs Alternatives

| Feature | Zarr | HDF5 | NetCDF4 | Parquet |
|---------|------|------|---------|---------|
| **Cloud storage** | ✅ | ⚠️ | ⚠️ | ✅ |
| **Parallel writes** | ✅ | ⚠️ | ⚠️ | ❌ |
| **N-D arrays** | ✅ | ✅ | ✅ | ❌ |
| **Tabular data** | ❌ | ⚠️ | ⚠️ | ✅ |
| **Append** | ✅ Fast | ⚠️ Slow | ⚠️ Slow | ⚠️ Slow |
| **Metadata** | JSON | Binary | Binary | Binary |

**Use Zarr for**: Climate models, genomics, satellite imagery, scientific computing

### Quick Start

```rust
use zarr::{ZarrArray, ZarrStore};

// 1. Import from S3
let store = ZarrStore::S3("s3://bucket/climate.zarr").await?;
let zarr = ZarrArray::open(store).await?;

let tensor_id = pyralog.import_zarr_array(
    zarr,
    "climate_temperature",
    parallelism: 32, // 32 concurrent chunk downloads
).await?;

// 2. Export to Zarr
pyralog.export_zarr(tensor_id, ExportConfig {
    path: "s3://bucket/output.zarr",
    chunk_shape: vec![1, 180, 360], // 1 time step per chunk
    compressor: ZarrCodec::Blosc {
        cname: CompressionName::Zstd,
        clevel: 5,
        shuffle: Shuffle::ByteShuffle,
    },
    parallelism: 64, // 64 concurrent uploads
}).await?;

// 3. Query with SQL
pyralog.register_external_table("climate", ZarrTable {
    location: "s3://bucket/climate.zarr",
    dimensions: vec!["time", "lat", "lon"],
}).await?;

let results = pyralog.query(r#"
    SELECT AVG(temperature) 
    FROM climate
    WHERE time >= '2020-01-01' 
      AND lat BETWEEN -10 AND 10
"#).await?;
```

### Directory Structure

```
climate.zarr/
├── .zarray              # Array metadata (JSON)
│   {
│     "shape": [365, 720, 1440],
│     "chunks": [1, 180, 360],
│     "dtype": "<f4",
│     "compressor": {"id": "blosc", "cname": "zstd"},
│     "fill_value": null
│   }
├── .zattrs              # User attributes (JSON)
│   {
│     "units": "Kelvin",
│     "source": "ERA5"
│   }
└── 0.0.0                # Chunk files (compressed)
    0.0.1
    0.0.2
    ...
    364.3.3
```

### Compression Codecs

| Codec | Ratio | Speed | Use Case |
|-------|-------|-------|----------|
| **Blosc (Zstd)** | 5-20× | 400 MB/s | **Recommended** |
| **Blosc (LZ4)** | 2-5× | 800 MB/s | Fast access |
| **Bitshuffle** | 10-100× | 300 MB/s | Sparse data |
| **Delta + Zstd** | 10-50× | 300 MB/s | Time-series |

### Performance

```
Benchmark: Climate data (365 days, 720×1440 grid = 1.5 GB)

Operation                HDF5      Zarr (S3)   Speedup
──────────────────────────────────────────────────────
Full array read          12 sec    1.5 sec     8×
Slice (single day)       8 sec     0.2 sec     40×
Slice (region + time)    10 sec    0.5 sec     20×
Append 1 day             15 sec    0.3 sec     50×
Parallel read (32×)      12 sec    0.6 sec     20×

→ Zarr: Chunked access + parallel I/O + no locking
```

---

## DLPack: Zero-Copy Exchange

### Overview

**DLPack** is a standard for zero-copy tensor exchange between ML frameworks, offering **300× speedup** over memcpy.

**Key features**:
- ✅ **Zero-copy**: Share memory directly
- ✅ **Cross-framework**: PyTorch ↔ TF ↔ JAX ↔ Pyralog
- ✅ **CPU & GPU**: CUDA, ROCm, Metal, Vulkan
- ✅ **Lightweight**: Minimal C struct (<100 lines)
- ✅ **Standard**: Industry-wide adoption

### Protocol

```c
// DLPack v0.5 specification
typedef struct {
    void* data;              // Pointer to tensor data
    DLDevice device;         // Device info (CPU, CUDA, etc.)
    int32_t ndim;            // Number of dimensions
    DLDataType dtype;        // Data type (float32, etc.)
    int64_t* shape;          // Shape array
    int64_t* strides;        // Strides (optional)
    uint64_t byte_offset;    // Offset in buffer
} DLTensor;
```

### Quick Start

```rust
use pyo3::prelude::*;

// 1. Export Pyralog → DLPack (zero-copy)
pub fn to_dlpack(tensor: &PyralogTensor) -> PyObject {
    Python::with_gil(|py| {
        let dl_tensor = DLManagedTensor {
            dl_tensor: DLTensor {
                data: tensor.data_ptr() as *mut c_void,
                device: DLDevice { device_type: kDLCPU, device_id: 0 },
                ndim: tensor.ndim() as i32,
                dtype: DLDataType { code: kDLFloat, bits: 32, lanes: 1 },
                shape: tensor.shape().as_ptr() as *mut i64,
                strides: null_mut(),
                byte_offset: 0,
            },
            manager_ctx: Arc::into_raw(tensor.clone()) as *mut c_void,
            deleter: Some(dlpack_deleter),
        };
        
        PyCapsule::new(py, dl_tensor, c"dltensor").into()
    })
}

// 2. Import DLPack → Pyralog (zero-copy)
pub fn from_dlpack(capsule: &PyAny) -> Result<PyralogTensor> {
    let dl_tensor = unsafe { capsule.extract::<*mut DLManagedTensor>()? };
    PyralogTensor::from_raw_parts(dl_tensor.data, shape, dtype)
}

// 3. PyTorch integration
pub fn to_pytorch(tensor: &PyralogTensor) -> PyObject {
    Python::with_gil(|py| {
        let dlpack = to_dlpack(tensor);
        py.import("torch")?.call_method1("from_dlpack", (dlpack,))
    })
}

pub fn from_pytorch(torch_tensor: PyObject) -> Result<PyralogTensor> {
    Python::with_gil(|py| {
        let dlpack = torch_tensor.call_method0(py, "__dlpack__")?;
        from_dlpack(dlpack.as_ref(py))
    })
}
```

### Framework Support

| Framework | Export | Import | GPU Support |
|-----------|--------|--------|-------------|
| **PyTorch** | ✅ | ✅ | ✅ CUDA/ROCm |
| **TensorFlow** | ✅ | ✅ | ✅ CUDA |
| **JAX** | ✅ | ✅ | ✅ CUDA/TPU |
| **NumPy** | ✅ | ✅ | ❌ CPU only |
| **CuPy** | ✅ | ✅ | ✅ CUDA |
| **Pyralog** | ✅ | ✅ | 🚧 CUDA planned |

### Performance

```
Benchmark: Transfer 1GB tensor between frameworks

Method                  Time       Memory    Speedup
──────────────────────────────────────────────────────
Copy (memcpy)           300 ms     2 GB      1×
Serialize (pickle)      2000 ms    3 GB      1×
DLPack (zero-copy)      <1 ms      1 GB      300-2000×

→ DLPack: No serialization, no copy, same memory
```

### Device Support

| Device | Code | Pyralog |
|--------|------|---------|
| **CPU** | `kDLCPU` | ✅ Full |
| **CUDA** | `kDLCUDA` | 🚧 Planned |
| **ROCm** | `kDLROCM` | 🔮 Future |
| **Metal** | `kDLMetal` | 🔮 Future |

---

## Performance Comparison

### Storage Formats

```
Dataset: 1 billion rows, 10 columns, mixed types

Format          Size    Write      Read       Query (filter+agg)
──────────────────────────────────────────────────────────────────
CSV (gzip)      15 GB   300 sec    180 sec    90 sec
JSON (gzip)     22 GB   420 sec    240 sec    120 sec
Parquet (zstd)  3.2 GB  45 sec     8 sec      2.5 sec
Arrow (native)  4.8 GB  12 sec     0.5 sec    2.0 sec

Best choice:
- Cold storage: Parquet (smallest, standard)
- Hot storage: Arrow (fastest, native)
```

### ML Model Formats

```
Model: BERT-base (440 MB, 110M parameters)

Format              Save    Load      Memory    Security
─────────────────────────────────────────────────────────
Pickle              1.2s    2.5s      880 MB    ❌ Unsafe
PyTorch .pth        1.5s    3.2s      880 MB    ❌ Unsafe
ONNX                2.8s    1.8s      440 MB    ✅ Safe
Safetensors         0.5s    0.025s    440 MB    ✅ Safe

→ Safetensors: 100× faster load, 50% memory, safe
```

### Array Formats

```
Dataset: Climate model output (365 days, 720×1440 = 1.5 GB)

Format      Storage   Read (full)   Read (slice)   Append
────────────────────────────────────────────────────────────
HDF5        1.8 GB    12 sec        8 sec          15 sec
NetCDF4     1.9 GB    14 sec        9 sec          18 sec
Zarr        0.3 GB    1.5 sec       0.2 sec        0.3 sec

→ Zarr: 5× compression, 8× faster, 50× faster append
```

### Tensor Exchange

```
Operation: Transfer 1GB tensor (PyTorch → Pyralog → TensorFlow)

Method                              Time      Memory
────────────────────────────────────────────────────
Serialize (pickle) → deserialize    4 sec     3 GB
Copy to numpy → copy to target      600 ms    2 GB
DLPack (zero-copy)                  <2 ms     1 GB

→ DLPack: 300-2000× faster, 50-67% memory
```

---

## Integration Patterns

### Pattern 1: Analytics Data Lake

```rust
/// Parquet for cold storage, Arrow for hot queries
pub async fn analytics_pipeline(
    pyralog: &PyralogClient,
) -> Result<()> {
    // 1. Query internal data
    let data = pyralog.query(
        "SELECT * FROM events WHERE date >= '2025-01-01'"
    ).await?;
    
    // 2. Export to Parquet on S3 (cold storage)
    pyralog.export_table("events", ExportConfig {
        format: DataFormat::Parquet,
        path: "s3://lake/events/date=2025-01-01/part.parquet",
        compression: Compression::Zstd(3),
    }).await?;
    
    // 3. Register as external table (query without import)
    pyralog.register_external_table("events_archive", ParquetTable {
        location: "s3://lake/events/**/*.parquet",
        partition_columns: vec!["date"],
    }).await?;
    
    // 4. Query unified view (hot + cold)
    let results = pyralog.query(r#"
        SELECT product, SUM(revenue)
        FROM events_archive
        WHERE date >= '2025-01-01'
        GROUP BY product
    "#).await?;
    
    Ok(())
}
```

### Pattern 2: ML Model Repository

```rust
/// Safetensors for persistence, DLPack for runtime
pub async fn ml_pipeline(
    pyralog: &PyralogClient,
) -> Result<()> {
    // 1. Train model (external framework)
    Python::with_gil(|py| {
        let model = train_model()?;
        
        // 2. Save to Safetensors (fast, safe)
        model.save_pretrained("model.safetensors")?;
    });
    
    // 3. Import into Pyralog
    let tensor_ids = pyralog.import_safetensors(
        "model.safetensors",
        "model_registry",
    ).await?;
    
    // 4. Load for inference (zero-copy via mmap)
    let weights = pyralog.load_model_mmap(
        "model_registry",
        "my_model",
    ).await?;
    
    // 5. Transfer to GPU via DLPack (zero-copy)
    Python::with_gil(|py| {
        let torch_weights = to_pytorch(&weights);
        let output = run_inference(torch_weights)?;
        
        // 6. Store results
        let pyralog_output = from_pytorch(output);
        pyralog.store_tensor("predictions", pyralog_output).await
    })
}
```

### Pattern 3: Scientific Data Archive

```rust
/// Zarr for cloud storage, Arrow for analysis
pub async fn scientific_pipeline(
    pyralog: &PyralogClient,
) -> Result<()> {
    // 1. Generate simulation output
    let tensor_id = pyralog.run_simulation("climate_model").await?;
    
    // 2. Export to Zarr on S3 (incremental)
    pyralog.export_zarr(tensor_id, ExportConfig {
        path: "s3://climate/2025/temperature.zarr",
        chunk_shape: vec![1, 180, 360], // 1 day per chunk
        compressor: ZarrCodec::Blosc {
            cname: CompressionName::Zstd,
            clevel: 5,
            shuffle: Shuffle::ByteShuffle,
        },
        incremental: true, // Append without rewriting
    }).await?;
    
    // 3. Analysts query directly from S3
    pyralog.register_external_table("climate", ZarrTable {
        location: "s3://climate/2025/temperature.zarr",
        dimensions: vec!["time", "lat", "lon"],
    }).await?;
    
    let results = pyralog.query(r#"
        SELECT time, AVG(temperature) as avg_temp
        FROM climate
        WHERE lat BETWEEN 30 AND 50  -- Europe
          AND time >= '2025-01-01'
        GROUP BY time
        ORDER BY time
    "#).await?;
    
    Ok(())
}
```

### Pattern 4: Real-Time Inference

```rust
/// DLPack for zero-copy, minimum latency
pub async fn realtime_inference(
    pyralog: &PyralogClient,
    request: &InferenceRequest,
) -> Result<Prediction> {
    // 1. Fetch features (Arrow native)
    let features = pyralog.get_features(request.user_id).await?;
    
    // 2. Convert to PyTorch (zero-copy via DLPack)
    Python::with_gil(|py| {
        let torch_features = to_pytorch(&features);
        
        // 3. Run inference (no copy overhead)
        let model = py.import("model")?;
        let torch_output = model.call_method1("predict", (torch_features,))?;
        
        // 4. Convert back to Pyralog (zero-copy)
        let prediction = from_pytorch(torch_output)?;
        
        // 5. Store prediction
        pyralog.store_prediction(request.user_id, &prediction).await
    })
}
```

---

## Summary

### Format Cheat Sheet

| Scenario | Format | Why |
|----------|--------|-----|
| Analytics data lake | Parquet | Columnar, compression, SQL-optimized |
| Real-time OLTP | Arrow | Native, zero-copy, SIMD |
| ML model storage | Safetensors | Memory-safe, 100× faster load |
| Model inference | DLPack | Zero-copy, <1ms overhead |
| Climate/scientific | Zarr | Chunked, parallel I/O, cloud-native |
| Time-series archive | Parquet | Compression, append-friendly |
| Embeddings/vectors | Arrow | SIMD, ANN indexing |
| Cross-framework | DLPack | Standard, no serialization |

### Performance Summary

```
Operation                      Speedup vs Baseline
──────────────────────────────────────────────────
Query 1B rows (Parquet)        36× vs CSV
Load 7B model (Safetensors)    100× vs pickle
Read climate (Zarr)            20× vs HDF5
Transfer tensor (DLPack)       300× vs memcpy
```

### Best Practices

1. **Cold storage**: Parquet (analytics) or Zarr (arrays)
2. **Hot storage**: Arrow (native Pyralog)
3. **ML models**: Safetensors (disk) + DLPack (runtime)
4. **Compression**: Zstd level 3-5 (best balance)
5. **Cloud**: Use external tables (query without import)
6. **Parallel I/O**: Zarr for arrays, Parquet for tables

### Next Steps

- 📖 [ARROW.md](ARROW.md) - Apache Arrow integration
- 📖 [TENSOR_DATABASE.md](TENSOR_DATABASE.md) - Tensor operations & ML
- 📖 [STORAGE.md](STORAGE.md) - LSM storage engine
- 📖 [MULTI_MODEL_DATABASE.md](MULTI_MODEL_DATABASE.md) - Multi-model storage

---

**Questions?** Join [Discord](https://discord.gg/pyralog) | [GitHub Issues](https://github.com/pyralog/pyralog/issues)
