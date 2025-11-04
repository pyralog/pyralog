# Pyralog Architecture Diagrams

Visual representations of Pyralog's architecture, data flows, and key concepts.

---

## Diagram Index

### Core Architecture (5)
- [system-architecture.mmd](system-architecture.mmd) - Complete system overview
- [component-relationships.mmd](component-relationships.mmd) - How components interact
- [two-tier-architecture.mmd](two-tier-architecture.mmd) - Obelisk vs Pyramid nodes ⭐
- [data-flow.mmd](data-flow.mmd) - Write and read paths
- [deployment-topologies.mmd](deployment-topologies.mmd) - Single vs multi-cluster

### Novel Primitives (3)
- [obelisk-pharaoh-scarab.mmd](obelisk-pharaoh-scarab.mmd) - The three core primitives ⭐
- [shen-ring.mmd](shen-ring.mmd) - The Five Rings architecture
- [exactly-once.mmd](exactly-once.mmd) - Exactly-once semantics flow

### Query & Execution (4)
- [query-execution.mmd](query-execution.mmd) - SQL to results pipeline ⭐
- [client-routing.mmd](client-routing.mmd) - Smart client pattern ⭐
- [transaction-flow.mmd](transaction-flow.mmd) - Distributed ACID transactions ⭐
- [batuta-compilation.mmd](batuta-compilation.mmd) - Batuta language pipeline

### Storage & Indexing (4)
- [lsm-storage.mmd](lsm-storage.mmd) - LSM tree structure
- [hybrid-storage.mmd](hybrid-storage.mmd) - LSM + file references
- [pphm-build.mmd](pphm-build.mmd) - PPHM build pipeline
- [deduplication-layers.mmd](deduplication-layers.mmd) - Multi-layer deduplication

### Distributed Systems (3)
- [consensus.mmd](consensus.mmd) - Raft consensus protocol
- [replication.mmd](replication.mmd) - CopySet replication strategies
- [failover.mmd](failover.mmd) - Recovery and failover flows

### Actor Model (2)
- [actor-topology.mmd](actor-topology.mmd) - Actor hierarchy and supervision
- [message-flow.mmd](message-flow.mmd) - Message passing patterns

### Multi-Model & Data Flow (4)
- [multi-model-joins.mmd](multi-model-joins.mmd) - Cross-model zero-copy joins
- [zero-copy-flow.mmd](zero-copy-flow.mmd) - End-to-end zero-copy
- [category-theory-schema.mmd](category-theory-schema.mmd) - Schema as category
- [vlsn-partitioning.mmd](vlsn-partitioning.mmd) - Client-side VLSN pattern

**Total: 25 diagrams** (⭐ = high priority)

---

## Viewing Diagrams

### GitHub
GitHub automatically renders `.mmd` (Mermaid) files in the web interface.

### VS Code
Install the "Mermaid Preview" extension:
```bash
code --install-extension bierner.markdown-mermaid
```

### Command Line
```bash
# Install mermaid-cli
npm install -g @mermaid-js/mermaid-cli

# Generate PNG
mmdc -i diagrams/system-architecture.mmd -o diagrams/system-architecture.png

# Generate SVG
mmdc -i diagrams/system-architecture.mmd -o diagrams/system-architecture.svg
```

### Online
Paste diagram content into [Mermaid Live Editor](https://mermaid.live/)

---

## Diagram Conventions

### Colors
- **Blue** (#4A90E2): Core infrastructure (Obelisk, Pharaoh, Scarab)
- **Green** (#7CB342): Storage layer (LSM, segments)
- **Orange** (#F57C00): Coordination (Raft, consensus)
- **Purple** (#9C27B0): Application layer (Batuta, actors)
- **Red** (#E53935): Critical paths, failures
- **Gray** (#757575): External systems, interfaces

### Shapes
- **Rectangle**: Components, services
- **Cylinder**: Storage, databases
- **Circle**: Actors, processes
- **Diamond**: Decision points
- **Hexagon**: External systems

---

## Contributing Diagrams

When adding new diagrams:

1. **Use Mermaid format** for compatibility
2. **Keep it simple** - one concept per diagram
3. **Add labels** to all nodes and edges
4. **Use consistent colors** per convention
5. **Update this README** with new diagram
6. **Test rendering** on GitHub before committing

---

## Generating All Diagrams

```bash
#!/bin/bash
# Generate all diagrams as PNG and SVG

for file in diagrams/*.mmd; do
    base=$(basename "$file" .mmd)
    echo "Generating $base..."
    mmdc -i "$file" -o "diagrams/$base.png" -b transparent
    mmdc -i "$file" -o "diagrams/$base.svg" -b transparent
done
```

Save as `generate-diagrams.sh` and run:
```bash
chmod +x generate-diagrams.sh
./generate-diagrams.sh
```

---

## License

All diagrams are released under the same license as Pyralog documentation (MIT-0).

