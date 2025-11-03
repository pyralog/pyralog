# Pyralog Architecture Diagrams

Visual representations of Pyralog's architecture, data flows, and key concepts.

---

## Diagram Index

### Core Architecture
- [system-architecture.mmd](system-architecture.mmd) - Complete system overview
- [component-relationships.mmd](component-relationships.mmd) - How components interact
- [data-flow.mmd](data-flow.mmd) - Write and read paths

### Novel Primitives
- [shen-ring.mmd](shen-ring.mmd) - The Five Rings architecture
- [obelisk-pharaoh-scarab.mmd](obelisk-pharaoh-scarab.mmd) - Coordination primitives
- [exactly-once.mmd](exactly-once.mmd) - Exactly-once semantics flow

### Storage & Indexing
- [lsm-storage.mmd](lsm-storage.mmd) - LSM tree structure
- [pphm-build.mmd](pphm-build.mmd) - PPHM build pipeline
- [deduplication-layers.mmd](deduplication-layers.mmd) - Multi-layer deduplication

### Distributed Systems
- [consensus.mmd](consensus.mmd) - Raft consensus protocol
- [replication.mmd](replication.mmd) - Data replication strategies
- [failover.mmd](failover.mmd) - Failover and recovery

### Actor Model
- [actor-topology.mmd](actor-topology.mmd) - Actor hierarchy and supervision
- [message-flow.mmd](message-flow.mmd) - Message passing patterns

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

