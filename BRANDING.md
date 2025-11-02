# DLog Branding Guide

> **Theme**: Ancient Egyptian Architecture & Symbolism  
> **Core Values**: Permanence · Power · Precision · Monumentality

---

## 🎨 Brand Identity

DLog's branding draws from **ancient Egyptian civilization**—a culture renowned for:

- **Engineering Excellence**: Pyramids and monuments that lasted millennia
- **Precision**: Advanced mathematics and astronomy
- **Distributed Coordination**: Managing vast territories without modern communication
- **Permanence**: Stone architecture, immutable hieroglyphics, eternal legacy

These values mirror DLog's technical characteristics:
- **Crash-safe primitives** (Obelisk Sequencer)
- **Distributed coordination** without consensus (Pharaoh Network)
- **Immutable, time-ordered data** (Scarab IDs)

---

## 🏛️ The Three Pillars

DLog's architecture is embodied by three Egyptian symbols:

### 🗿 Obelisk Sequencer

**Symbol**: Monument · Primitive · Foundation  
**What It Represents**: The fundamental building block—tall, permanent, unshakeable

**Technical Meaning**:
- Crash-safe persistent atomic counters
- Minimal disk overhead (~1-2μs per increment)
- Instant recovery after crashes
- Foundation for all distributed coordination

**Visual Metaphor**:
- Obelisks marked important locations in ancient Egypt
- Single solid piece of stone (monolithic, atomic)
- Tall and visible from afar (observable state)
- Survived thousands of years (durability)

**Usage in Documentation**:
```markdown
🗿 Obelisk Sequencer enables crash-safe counting...
The 🗿 Obelisk Sequencer primitive...
Built on the Obelisk Sequencer (🗿)...
```

---

### ☀️ Pharaoh Network

**Symbol**: Sun God (Ra) · Ruler · Coordinator  
**What It Represents**: Divine power radiating to all points—central authority that reaches everywhere

**Technical Meaning**:
- Distributed coordination without centralized bottlenecks
- 1024 independent coordinator nodes
- Snowflake-style architecture eliminating single points of failure
- Linear horizontal scalability

**Visual Metaphor**:
- Pharaohs = divine rulers (coordinators)
- Ra = sun god (rays reaching all points = distributed)
- Royal authority without physical presence everywhere (remote coordination)
- Pyramids as tombs = lasting coordination records

**Usage in Documentation**:
```markdown
☀️ Pharaoh Network eliminates bottlenecks...
The ☀️ Pharaoh Network pattern...
Built on Pharaoh Network (☀️)...
```

---

### 🪲 Scarab IDs

**Symbol**: Sacred Beetle · Seal · Identity  
**What It Represents**: Unique identification and authentication—the scarab as royal seal

**Technical Meaning**:
- 64-bit distributed unique IDs
- Time-ordered, globally unique
- No coordination required
- Inspired by Twitter's Snowflake (2010)

**Visual Metaphor**:
- Scarab = sacred beetle, symbol of transformation
- Used as **seals** for authentication and identity
- Pressed into clay/wax = immutable record
- Each scarab seal unique = globally unique IDs
- Symbol of eternal life = permanent identifiers

**Usage in Documentation**:
```markdown
🪲 Scarab IDs provide unique identifiers...
The 🪲 Scarab ID algorithm...
Using Scarab IDs (🪲)...
```

---

### 🎼 Batuta Language

**Symbol**: Conductor's Baton · Orchestration · Direction  
**What It Represents**: The conductor guiding the orchestra—orchestrating distributed data operations

**Technical Meaning**:
- Dynamic programming language (Clojure + Elixir inspired)
- Actor-first queries and data processing
- Compiles to Rust and WebAssembly
- Lisp syntax with macros and metaprogramming
- Native DLog integration

**Visual Metaphor**:
- Batuta = Spanish/Portuguese/Italian for "baton"
- Conductor = orchestrator (like pharaoh coordinates)
- Baton directs musicians = language directs data operations
- Harmony through coordination = distributed computation

**Why Not Egyptian?**:
- Batuta represents the **human interface** to DLog
- While infrastructure is Egyptian (permanent, monumental), the language is **orchestral** (expressive, artistic)
- Bridges the gap between raw power (🗿☀️🪲) and human expression (🎼)
- Complements rather than replaces the Egyptian theme

**Usage in Documentation**:
```markdown
🎼 Batuta provides an expressive interface...
The 🎼 Batuta language compiles to Rust...
Query data using Batuta (🎼)...
```

**Relationship to Egyptian Theme**:
```
Egyptian Foundation (Infrastructure)
    🗿 Obelisk Sequencer
    ☀️ Pharaoh Network  
    🪲 Scarab IDs
         ↓
    Powers
         ↓
🎼 Batuta (Human Interface)
    Orchestrates the infrastructure
    Expressive query language
    Actor-based concurrency
```

---

## 🎭 Brand Architecture

### The Hierarchy

```
┌─────────────────────────────────────────────────┐
│              DLog Platform                      │
│         (Unified Data Platform)                 │
└─────────────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
    Egyptian                🎼 Batuta
  Infrastructure          (Interface)
        │
    ┌───┼───┐
    │   │   │
┌───▼┐ ┌▼──┐ ┌▼───┐
│🗿  │ │☀️ │ │🪲  │
│Seq.│ │Net│ │IDs │
└────┘ └───┘ └────┘
Found  Coord  Ident
```

### How They Work Together

```rust
// 🗿 Obelisk Sequencer provides the primitive
let counter = ObeliskSequencer::open("counter.seq")?;

// 🪲 Scarab IDs use Obelisk for crash-safety
let scarab = ScarabGenerator::new(worker_id, counter);
let unique_id = scarab.next_id()?;  // Globally unique!

// ☀️ Pharaoh Network uses Scarab IDs for distribution
// 1024 coordinators × Scarab IDs = no bottlenecks
let coordinator_id = hash(key) % 1024;
let pharaoh_node = pharaoh_network.get(coordinator_id);
```

**🎼 Batuta orchestrates it all**:

```clojure
;; Batuta code - expressive and powerful
(defquery user-activity
  "Find active users using DLog primitives"
  [db time-range]
  (->> (dlog/query db
         {:find [?user ?count]
          :where [[?e :event/user ?user]
                  [?e :event/timestamp ?t]
                  [(>= ?t ~(:start time-range))]]})
       (group-by first)
       (map (fn [[user events]] 
              {:user user 
               :count (count events)
               :id (dlog/scarab-id)}))))  ; 🪲 Scarab IDs
```

**Visual Flow**:
```
        🎼 Batuta (orchestrates)
              ↓
         queries via
              ↓
🗿 Obelisk (primitive)
    ↓ enables
🪲 Scarab (identifiers)
    ↓ powers
☀️ Pharaoh (coordination)
    ↓ achieves
28 Billion ops/sec
```

---

## 🎨 Visual Identity

### Color Palette

**Primary Colors** (Egyptian-inspired):

```
Gold (Ra/Sun):        #D4AF37  ■  Authority, coordination
Sandstone:            #C2B280  ■  Foundation, stability
Lapis Blue:           #15317E  ■  Ancient Egyptian royal blue
Papyrus Green:        #3B5323  ■  Data, records
Terracotta:           #C04000  ■  Energy, performance
```

**Neutral Colors**:

```
Desert Sand:          #EDC9AF  ■  Backgrounds
Dark Stone:           #2F2F2F  ■  Text, code blocks
Pure White:           #FFFFFF  ■  Clean space
```

**Accent Colors**:

```
Scarab Turquoise:     #30D5C8  ■  Highlights, links
Hieroglyph Black:     #000000  ■  Strong emphasis
```

### Typography

**Headings**: 
- Clean, modern sans-serif (system fonts)
- Bold weight for prominence
- All-caps for major sections (optional)

**Body Text**:
- High readability
- Monospace for code
- Clear hierarchy

**Code Blocks**:
- Monospace (Fira Code, JetBrains Mono, Consolas, Monaco)
- Syntax highlighting with Egyptian palette

---

## 📝 Writing Style

### Tone

**Confident · Precise · Monumental**

- **Do**: "DLog achieves 28 billion operations per second"
- **Don't**: "DLog might be able to handle billions of operations"

**Technical but Accessible**

- **Do**: "The Obelisk Sequencer uses sparse files for crash-safe counters"
- **Don't**: "It's kinda like a counter but with files or something"

**Inspired by Ancient Engineering**

- **Do**: "Built to last millennia, like the pyramids"
- **Don't**: "It's pretty durable"

### Voice

**DLog is**:
- ✅ Ambitious (28 billion ops/sec)
- ✅ Precise (exact numbers, benchmarks)
- ✅ Confident (definitive statements)
- ✅ Educational (explains complex concepts)
- ✅ Monumental (big ideas, lasting impact)

**DLog is NOT**:
- ❌ Arrogant (doesn't belittle competitors)
- ❌ Vague (no hand-waving)
- ❌ Hype-driven (no buzzwords without substance)
- ❌ Apologetic (doesn't hedge unnecessarily)

---

## 🏷️ Terminology Guidelines

### Core Terms (Always Capitalized)

- **Obelisk Sequencer** (not "obelisk sequencer" or "Obelisk")
- **Pharaoh Network** (not "pharaoh network" or "Pharaoh")
- **Scarab IDs** (not "scarab IDs" or "Scarab")
- **Batuta** (not "batuta" or "BATUTA")
- **DLog** (not "dlog" or "DLOG" or "D-Log")

### When to Use Icons

**Always use icons** in:
- Headings introducing the concept
- Feature lists
- Architecture diagrams
- Quick references

**Example**:
```markdown
## 🗿 Obelisk Sequencer: Crash-Safe Counters

DLog's Obelisk Sequencer provides:
- Atomic increments
- Crash safety
- Instant recovery

## 🎼 Batuta: Orchestrating Data Operations

Query DLog using Batuta's expressive syntax:
- Lisp macros
- Actor-first concurrency
- Compiles to Rust
```

**Don't overuse** in:
- Body paragraphs (mention icon once)
- Every sentence
- Code comments (use names)

**Example** (good balance):
```markdown
The 🗿 Obelisk Sequencer is a crash-safe persistent atomic counter. 
It uses sparse files to represent counter values as file sizes. When 
the Obelisk Sequencer crashes and restarts, it reads the file size 
to recover the counter value instantly.
```

---

## 📐 Logo & Iconography

### DLog Platform Icon

**🔺 Pyramid** represents DLog as a platform:
- Most iconic Egyptian symbol
- Represents **layered architecture** (data layer, coordination layer, interface layer)
- **Timeless monument** = built to last millennia
- **Solid foundation** = stable infrastructure
- **Points upward** = scaling to the sky
- **Hierarchical** = clear system organization

### Component Icons

**The Four Component Icons**:
- 🗿 = Moai (Easter Island head) represents obelisk/monument
- ☀️ = Sun with rays represents pharaoh/coordination
- 🪲 = Beetle represents scarab/identity
- 🎼 = Musical score represents batuta/orchestration

**Primary Logo**:
```
       🔺
      DLog
    ────────
   🗿 ☀️ 🪲 🎼
```

**Alternative - Pyramid Structure**:
```
        🔺 DLog
       ─────────
      🎼 Batuta
     ───────────
    🗿  ☀️  🪲
  Infrastructure
```

**Text Logo (Horizontal)**:
```
🔺 DLog  |  🗿 Obelisk · ☀️ Pharaoh · 🪲 Scarab · 🎼 Batuta
```

**Simple Stack**:
```
   🔺 DLog
   ───────
   🗿☀️🪲🎼
```

### ASCII Art Diagrams

Use ASCII art liberally for:
- Architecture diagrams
- Data flow
- System topology
- Performance comparisons

**Example**:
```
┌──────────────────────────────────────┐
│   🎼 Batuta Query Interface          │
│  ┌────────────────────────────────┐ │
│  │  ☀️ Pharaoh Network            │ │
│  │ ┌────────────────────────────┐ │ │
│  │ │ 1024 Coordinator Nodes     │ │ │
│  │ │ (🪲 Scarab IDs)            │ │ │
│  │ │ (🗿 Obelisk)               │ │ │
│  │ └────────────────────────────┘ │ │
│  └────────────────────────────────┘ │
└──────────────────────────────────────┘
```

---

## 🌍 Attribution & Credits

### Inspiration Sources

**Always credit**:
- Twitter's Snowflake (2010) - inspiration for Scarab IDs
- TiKV's Percolator - transaction protocol
- **Clojure** (Rich Hickey) - Batuta's Lisp syntax, persistent data structures, macros
- **Elixir/Erlang/OTP** (José Valim, Joe Armstrong) - Batuta's actor model, supervision trees
- **Zig** (Andrew Kelley) - Batuta's error handling (`Result!Type`)
- **Pony** (Sylvan Clebsch) - Batuta's reference capabilities
- Stella Language - actor-reactor model
- WireGuard - networking protocol

**Format**:
```markdown
🪲 Scarab IDs are inspired by Twitter's Snowflake algorithm (2010)
```

### Egyptian Theme

**Acknowledge**:
- Ancient Egyptian civilization
- Not appropriation—respectful homage to engineering excellence
- Educational context provided

### Multi-Cultural Design

**Why both Egyptian and Latin?**:
- **Egyptian theme** (🗿☀️🪲) = Infrastructure layer (permanent, monumental, low-level)
- **Latin/Romance theme** (🎼 Batuta) = Human interface (expressive, artistic, high-level)
- Represents the **duality of systems**:
  - Stone foundations (Egyptian engineering)
  - Artistic expression (Latin musical tradition)
- Both cultures valued **precision, beauty, and lasting impact**

---

## 📊 Use Cases for Branding

### 1. Documentation

**Project README**:
```markdown
# 🔺 DLog

> Built to Last Millennia

DLog is a unified distributed data platform...

## Components
- 🗿 Obelisk Sequencer
- ☀️ Pharaoh Network
- 🪲 Scarab IDs
- 🎼 Batuta Language
```

**Section Headings**:
```markdown
# 🗿 Obelisk Sequencer

## Overview
The Obelisk Sequencer is a crash-safe...

## Architecture
Built on sparse files...
```

### 2. Blog Posts

**Titles**:
- "The Obelisk Sequencer: A Novel Persistent Atomic Primitive"
- "Pharaoh Network: Coordination Without Consensus"
- "Scarab IDs: Distributed Identity at Scale"

### 3. Conference Talks

**Slide Structure**:
1. Title slide: "🔺 DLog: Built to Last Millennia"
2. Problem: Centralized bottlenecks
3. Solution: Four components (🗿☀️🪲🎼)
4. Each component gets dedicated slides with icon
5. Pyramid architecture diagram

### 4. Social Media

**Twitter/X**:
```
🔺 DLog achieves 28B ops/sec with:
🗿 Obelisk Sequencer - crash-safe counters
☀️ Pharaoh Network - distributed coordination
🪲 Scarab IDs - unique identifiers
🎼 Batuta - expressive query language

Built to last millennia. Zero bottlenecks.
```

**GitHub README Badge Ideas**:
```markdown
![DLog](https://img.shields.io/badge/🔺-DLog-C2B280)
![Obelisk](https://img.shields.io/badge/🗿-Obelisk-D4AF37)
![Pharaoh](https://img.shields.io/badge/☀️-Pharaoh-15317E)
![Scarab](https://img.shields.io/badge/🪲-Scarab-30D5C8)
![Batuta](https://img.shields.io/badge/🎼-Batuta-9B59B6)
```

### 5. Code Documentation

**Rust Doc Comments**:
```rust
/// The Obelisk Sequencer (🗿) provides crash-safe persistent
/// atomic counters with minimal disk overhead.
///
/// # Architecture
/// Uses sparse files where file size = counter value...
pub struct ObeliskSequencer {
    // ...
}
```

---

## 🎯 Brand Positioning

### Taglines

**Primary**:
> **🔺 DLog: Built to Last Millennia**

**Alternatives**:
- "🔺 DLog: Monumental Performance, Eternal Data"
- "🔺 DLog: The Distributed Log for the Ages"
- "🔺 DLog: Solid Foundation, Infinite Scale"
- "🔺 DLog: 28 Billion Operations Per Second. Zero Compromises."

### Elevator Pitch

**30 seconds**:
```
🔺 DLog is a unified distributed data platform inspired by ancient 
Egyptian engineering. Like the pyramids, it's built to last—with 
crash-safe primitives (🗿 Obelisk Sequencer), distributed 
coordination (☀️ Pharaoh Network), unique identifiers (🪲 Scarab IDs),
and an expressive language (🎼 Batuta). It achieves 28 billion 
operations per second without centralized bottlenecks.
```

**5 seconds**:
```
🔺 DLog: Distributed data infrastructure with zero bottlenecks.
28 billion ops/sec. Built to last millennia.
```

---

## ✅ Brand Checklist

When creating DLog content, ensure:

- [ ] Egyptian theme is consistent (if using metaphors for infrastructure)
- [ ] Four components (🗿☀️🪲🎼) are properly introduced when relevant
- [ ] Icons used appropriately (not overused)
- [ ] Batuta code examples use proper Lisp syntax
- [ ] Technical precision maintained
- [ ] Performance numbers cited with sources
- [ ] Proper attribution to inspirations (Clojure, Elixir, Zig, Pony, etc.)
- [ ] Confident but not arrogant tone
- [ ] Architecture diagrams use ASCII art
- [ ] Code examples are production-quality
- [ ] Educational value provided

---

## 🚫 What to Avoid

### Don't

❌ **Mix metaphors**: Don't introduce Greek/Roman gods alongside Egyptian (Batuta's Latin origin is intentional exception)  
❌ **Overuse icons**: Not every mention needs 🗿☀️🪲🎼  
❌ **Claim perfection**: Acknowledge trade-offs  
❌ **Belittle competitors**: Compare objectively  
❌ **Use buzzwords**: No "revolutionary" without proof  
❌ **Forget attribution**: Credit Twitter, TiKV, Clojure, Elixir, Zig, Pony, etc.  
❌ **Inconsistent naming**: Always "Obelisk Sequencer", not "Obelisk"; "Batuta", not "batuta"

### Do

✅ **Stay consistent**: Egyptian theme for infrastructure, Batuta for language  
✅ **Be precise**: Exact numbers, benchmarks  
✅ **Educate**: Explain complex concepts clearly  
✅ **Show code**: Real Rust and Batuta examples  
✅ **Benchmark**: Production-validated numbers  
✅ **Acknowledge**: Credit all inspirations (Clojure, Elixir, Zig, Pony, etc.)  
✅ **Maintain hierarchy**: Foundation → Coordination → Orchestration → Scale

---

## 📚 Resources

### Internal Documents

- [README.md](README.md) - Project overview
- [PAPER.md](PAPER.md) - Academic research paper
- [BATUTA.md](BATUTA.md) - Batuta language specification
- [Blog Series](blog/README.md) - Technical deep-dives
  - [The Obelisk Sequencer](blog/2-obelisk-sequencer.md)
  - [Pharaoh Network](blog/3-pharaoh-network.md)
  - [Batuta Language](blog/8-batuta-language.md)

### External References

- Twitter's Snowflake: [Engineering Blog Post (2010)](https://blog.twitter.com/engineering/en_us/a/2010/announcing-snowflake)
- Ancient Egyptian Architecture: Educational context
- Distributed Systems Theory: Consensus, replication, coordination

---

## 📝 Version History

- **v1.2** (2025-11-02): Added DLog platform icon
  - Selected 🔺 Pyramid as primary DLog icon
  - Represents layered architecture and lasting monument
  - Updated all logos and branding examples
  - Added pyramid structure diagrams

- **v1.1** (2025-11-02): Added Batuta language
  - Added 🎼 Batuta as fourth component
  - Multi-cultural design philosophy (Egyptian + Latin)
  - Batuta code examples and guidelines
  - Updated all diagrams and examples

- **v1.0** (2025-11-02): Initial branding guide
  - Established Egyptian theme
  - Defined three pillars: 🗿☀️🪲
  - Color palette and typography
  - Usage guidelines

---

## 📄 License

This branding guide is licensed under **CC0-1.0** (Public Domain).

You are free to:
- Use DLog's branding in presentations, articles, and documentation
- Reference the Egyptian theme in your own work
- Create derivative content about DLog

We encourage:
- Proper attribution to DLog and its inspirations
- Maintaining technical accuracy
- Educational use

---

**Questions?** Open an issue or join our [Discord](https://discord.gg/dlog)

**🔺 DLog**: Built to last millennia. 🗿☀️🪲🎼

