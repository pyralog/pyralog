# 🔺 Pyralog Branding Guide

> **Built to Last Millennia**

**Theme**: Ancient Egyptian Architecture & Symbolism  
**Core Values**: Permanence · Power · Precision · Monumentality

---

## 📖 Table of Contents

### Core Identity
- [Brand Philosophy](#-brand-philosophy) - Why Egyptian theme
- [The Platform Icon](#-pyralog-platform-icon) - 🔺 Pyramid
- [The Four Pillars](#️-the-four-pillars) - 🗿 ☀️ 🪲 🎼
- [Sulise Foundation](#-sulise-evergreen) - 🌲 Language development toolkit
- [Brand Architecture](#-brand-architecture) - How they work together

### Visual Design
- [Visual Identity](#-visual-identity) - Colors, typography, logos
- [Logo Variations](#logo--iconography) - Usage examples

### Usage Guidelines
- [Writing Style](#-writing-style) - Tone and voice
- [Terminology](#️-terminology-guidelines) - Naming conventions
- [Use Cases](#-use-cases-for-branding) - Documentation, blog, social media

### Reference
- [Attribution](#-attribution--credits) - Inspirations and credits
- [Checklist](#-brand-checklist) - Quality control
- [What to Avoid](#-what-to-avoid) - Common mistakes

---

## 🎨 Brand Philosophy

Pyralog's branding draws from **ancient Egyptian civilization**—a culture that perfected:

- **Engineering Excellence**: Pyramids lasting 4,500+ years
- **Mathematical Precision**: Advanced geometry and astronomy
- **Distributed Coordination**: Managing vast territories without modern technology
- **Permanence**: Stone architecture, immutable records, eternal legacy

**Why Egyptian?** These values directly mirror Pyralog's architecture:

| Egyptian Engineering | Pyralog Technology |
|---------------------|-------------------|
| Stone monuments (permanent) | Crash-safe primitives |
| Pharaohs (distributed authority) | Decentralized coordination |
| Scarab seals (unique identity) | Globally unique IDs |
| Hieroglyphics (immutable records) | Append-only logs |

**Why Batuta (Latin)?** The 🎼 Batuta language represents the **human interface** layer—orchestration and expression. While the infrastructure is Egyptian (monumental, permanent), the interface is musical (expressive, artistic).

---

## 🔺 Pyralog Platform Icon

**Symbol**: Pyramid  
**Represents**: The complete Pyralog platform

**Why Pyramid?**
- Most iconic Egyptian symbol
- **Layered architecture** - Foundation → Coordination → Interface
- **Timeless monument** - Built to last millennia
- **Solid foundation** - Stable, crash-safe infrastructure
- **Points upward** - Infinite scalability
- **Hierarchical structure** - Clear system organization

**Platform Hierarchy**:
```
      🔺 Pyramid
     Platform Layer
    ──────────────
     🎼 Interface
    (Batuta Language)
    ──────────────
   🗿  ☀️  🪲
  Infrastructure
   (Egyptian)
```

---

## 🏛️ The Four Pillars

Pyralog's architecture consists of three Egyptian infrastructure primitives and one Latin interface language:

| Component | Symbol | What It Is | Key Features |
|-----------|--------|------------|--------------|
| **🗿 Obelisk Sequencer** | Monument | Crash-safe atomic counter | • Sparse files<br>• ~1-2μs latency<br>• Instant recovery |
| **☀️ Pharaoh Network** | Sun/Ruler | Distributed coordination | • 1024 nodes<br>• No bottlenecks<br>• Linear scaling |
| **🪲 Scarab IDs** | Sacred Beetle | Unique identifiers | • 64-bit IDs<br>• Time-ordered<br>• Zero coordination |
| **🎼 Batuta** | Conductor's Baton | Query language | • Clojure + Elixir<br>• Actor-first<br>• Compiles to Rust |

**Plus the grammar foundation**: 🌲 [Sulise Evergreen](#-sulise-evergreen) (modular grammar toolkit)

---

### 🗿 Obelisk Sequencer

**Egyptian Symbol**: Obelisk/Monument  
**Technical Role**: Foundation primitive for crash-safe atomic operations

**What It Does**:
- Persistent atomic counters using sparse files
- File size = counter value (genius simplicity)
- Survives crashes with instant recovery
- Powers Scarab IDs and transaction sequencing

**Why Obelisk?**
- Single piece of stone = atomic operation
- Tall and permanent = durable state
- Marks important locations = observable checkpoints
- Lasted millennia = crash-safe guarantee

**Usage Example**:
```rust
let counter = ObeliskSequencer::open("txn.seq")?;
let id = counter.increment()?;  // Crash-safe!
```

---

### ☀️ Pharaoh Network

**Egyptian Symbol**: Sun God Ra / Pharaoh  
**Technical Role**: Distributed coordination without centralized bottlenecks

**What It Does**:
- 1024 independent coordinator nodes
- Inspired by Twitter's Snowflake architecture
- Each node autonomous, no single point of failure
- Linear horizontal scalability

**Why Pharaoh/Sun?**
- Pharaohs = distributed authority
- Sun rays = reaching all points simultaneously
- Ra = divine power without physical presence
- Royal seals = coordination tokens

**Usage Example**:
```rust
let coordinator = pharaoh_network.route(key);  // No bottleneck!
let id = coordinator.assign_scarab_id()?;
```

---

### 🪲 Scarab IDs

**Egyptian Symbol**: Sacred Scarab Beetle  
**Technical Role**: Globally unique, time-ordered identifiers

**What It Does**:
- 64-bit distributed IDs (inspired by Snowflake)
- Timestamp + Coordinator ID + Sequence
- No coordination needed between nodes
- Monotonically increasing within coordinator

**Why Scarab?**
- Sacred seals for identity and authentication
- Each seal unique = globally unique IDs
- Pressed into clay/wax = immutable records
- Symbol of transformation = data lifecycle

**Usage Example**:
```rust
let generator = ScarabGenerator::new(worker_id, sequencer);
let id = generator.next()?;  // Globally unique!
```

---

### 🎼 Batuta Language

**Latin Symbol**: Conductor's Baton  
**Technical Role**: Human interface for orchestrating distributed operations

**What It Does**:
- Lisp-based language (Clojure + Elixir fusion)
- Actor-first concurrency model
- Compiles to Rust and WebAssembly
- Native Pyralog integration
- Zig-style error handling, Pony-style capabilities

**Why Batuta (not Egyptian)?**
- **Infrastructure is Egyptian** (permanent, monumental)
- **Interface is Latin** (expressive, artistic)
- Conductor orchestrates the orchestra
- Bridges power (🗿☀️🪲) with expression (🎼)
- Both cultures valued precision and beauty

**Usage Example**:
```clojure
(defquery active-users [db time-range]
  (->> (pyralog/query db
         {:find [?user ?count]
          :where [[?e :event/user ?user]
                  [?e :event/timestamp ?t]
                  [(>= ?t ~(:start time-range))]]})
       (group-by first)
       (map (fn [[user events]]
              {:user user
               :count (count events)
               :scarab (pyralog/scarab-id)}))))
```

---

### 🌲 Sulise Evergreen

**Symbol**: Evergreen Tree  
**Technical Role**: Complete programming language development toolkit

**What It Does**:
- **Grammar & Parsing**: Modular S-expression and surface syntaxes (EBNF, profiles, desugaring)
- **Type Systems**: Type theory, inference, polymorphism
- **Semantics**: Operational, denotational, axiomatic semantics
- **Compilation**: AST transformations, optimization passes, code generation
- **Language Design**: Primitives, abstractions, composition rules
- **Theoretical Foundations**: Category theory, lambda calculus, formal methods

**Why Evergreen?**
- **Complete tree of knowledge** = all language development aspects
- **Evergreen** = permanent, stable, comprehensive foundation
- **Growth** = extensible, composable language primitives
- **Deep roots** = theoretical foundations (category theory, type theory)
- **Strong trunk** = core language primitives (S-expressions, evaluation)
- **Many branches** = diverse language features (syntax, types, semantics)
- **Always green** = timeless principles that never go out of date

**Visual Metaphor**:
- **Roots** = Theoretical foundations (lambda calculus, category theory)
- **Trunk** = Core primitives (S-expressions, evaluation model)
- **Branches** = Language features (syntax, types, semantics, compilation)
- **Leaves** = Concrete implementations (Batuta, other languages)
- **Evergreen** = Permanent foundation for all language development

**Relationship to Batuta**:
```
🌲 Sulise (Language Dev Toolkit)
    ↓ provides foundation for
🎼 Batuta (Concrete Language)
    ↓ orchestrates
🗿☀️🪲 Pyralog (Infrastructure)
```

**Complete Scope**:
```
Grammar ────┐
Types ──────┤
Semantics ──┤─→ 🌲 Sulise ─→ 🎼 Batuta
Compiler ───┤
Theory ─────┘
```

**Usage Example**:
```
;; Sulise provides complete language toolkit
(defrule expression syntax/grammar)
(deftype Value type-system/inference)
(defsem eval semantics/operational)
(defpass optimize compiler/transform)

;; Batuta is built using Sulise primitives
(deflang batuta
  :grammar sulise/profile-a
  :types sulise/gradual
  :semantics sulise/actor-model)
```

**Why Not Egyptian?**
- Sulise is **academic/research** (complete language science)
- Batuta is **artistic/human** (Latin, musical, expressive)
- Pyralog is **infrastructure** (Egyptian, monumental, permanent)
- Each layer has its own cultural metaphor that fits its purpose

---

## 🎭 Brand Architecture

**How the Complete Stack Works Together**:

```
┌───────────────────────────────────────────────┐
│        🎼 Batuta (Interface Layer)            │
│   Orchestrates queries and data operations    │
└───────────────────────────────────────────────┘
                 ↓ syntax provided by
          ┌────────────────────┐
          │  🌲 Sulise         │
          │  (Grammar Layer)   │
          └────────────────────┘
                 ↓ uses infrastructure
┌───────────────────────────────────────────────┐
│      🗿 Obelisk → 🪲 Scarab → ☀️ Pharaoh      │
│         Primitive    IDs      Network         │
│                                                │
│  Foundation  →  Identity  → Coordination      │
└───────────────────────────────────────────────┘
                     ↓ achieves
           28 Billion Operations/Second
```

**Dependency Flow**:

1. **🗿 Obelisk** = Foundation primitive (crash-safe counter)
2. **🪲 Scarab** = Uses Obelisk for sequence generation
3. **☀️ Pharaoh** = Uses Scarab for distributed coordination
4. **🌲 Sulise** = Grammar foundation (EBNF, profiles, desugaring)
5. **🎼 Batuta** = Uses Sulise grammar to orchestrate infrastructure

**Code Example Showing Integration**:

```rust
// 1. Foundation: Obelisk provides crash-safety
let counter = ObeliskSequencer::open("counter.seq")?;

// 2. Identity: Scarab uses Obelisk for uniqueness
let scarab = ScarabGenerator::new(worker_id, counter);
let unique_id = scarab.next()?;

// 3. Coordination: Pharaoh routes via Scarab
let coordinator = pharaoh_network.route_by_id(unique_id);

// 4. Interface: Batuta orchestrates everything
```

```clojure
;; Batuta query using all three primitives
(defquery user-transactions [db user-id]
  (pyralog/query db
    {:find [?txn-id ?amount ?timestamp]
     :where [[?t :txn/user user-id]
             [?t :txn/id ?txn-id]      ; 🪲 Scarab ID
             [?t :txn/amount ?amount]
             [?t :txn/timestamp ?timestamp]]}))
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

- **Do**: "Pyralog achieves 28 billion operations per second"
- **Don't**: "Pyralog might be able to handle billions of operations"

**Technical but Accessible**

- **Do**: "The Obelisk Sequencer uses sparse files for crash-safe counters"
- **Don't**: "It's kinda like a counter but with files or something"

**Inspired by Ancient Engineering**

- **Do**: "Built to last millennia, like the pyramids"
- **Don't**: "It's pretty durable"

### Voice

**Pyralog is**:
- ✅ Ambitious (28 billion ops/sec)
- ✅ Precise (exact numbers, benchmarks)
- ✅ Confident (definitive statements)
- ✅ Educational (explains complex concepts)
- ✅ Monumental (big ideas, lasting impact)

**Pyralog is NOT**:
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
- **Pyralog** (not "pyralog" or "PYRALOG" or "D-Log")

### When to Use Icons

**Always use icons** in:
- Headings introducing the concept
- Feature lists
- Architecture diagrams
- Quick references

**Example**:
```markdown
## 🗿 Obelisk Sequencer: Crash-Safe Counters

Pyralog's Obelisk Sequencer provides:
- Atomic increments
- Crash safety
- Instant recovery

## 🎼 Batuta: Orchestrating Data Operations

Query Pyralog using Batuta's expressive syntax:
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

### Logo Variations

**1. Full Stack** (Complete Branding):
```
       🔺
      Pyralog
    ────────────
   🗿 ☀️ 🪲 🎼
  Obelisk·Pharaoh·Scarab·Batuta
    (🌲 Sulise grammar)
```

**2. Simple** (Clean & Minimal):
```
🔺 Pyralog
🗿☀️🪲🎼
```

**3. Horizontal** (Headers & Badges):
```
🔺 Pyralog  |  🗿 Obelisk · ☀️ Pharaoh · 🪲 Scarab · 🎼 Batuta · 🌲 Sulise
```

**4. Layered** (Shows Full Architecture):
```
      🔺 Pyralog
     ─────────
    🎼 Batuta
   ───────────
    🌲 Sulise
   ───────────
  🗿  ☀️  🪲
 Infrastructure
```

### Icon Reference

**Core Components**:

| Icon | Represents | Use When |
|------|------------|----------|
| 🔺 | Pyralog Platform | Mentioning entire system |
| 🗿 | Obelisk Sequencer | Discussing crash-safety |
| ☀️ | Pharaoh Network | Discussing coordination |
| 🪲 | Scarab IDs | Discussing unique identifiers |
| 🎼 | Batuta Language | Discussing queries/interface |
| 🌲 | Sulise Evergreen | Discussing language development/theory |

**Feature Icons**:

| Icon | Represents | Metaphor | Use When |
|------|------------|----------|----------|
| 🎭 | Actor Model | Theater performers | Discussing actors, concurrency, message-passing |
| 🕸️ | Distributed Systems | Decentralized mesh | Discussing distribution, clustering, decentralization |
| ⚡ | Parallel Execution | Lightning speed | Discussing parallelism, performance, throughput |
| 🔒 | Cryptographic Security | Lock/vault | Discussing verification, encryption, zero-trust |
| 🗄️ | Multi-Model Database | Filing cabinet | Discussing data models, queries, storage |
| 🧮 | Tensor Operations | Computation | Discussing ML, AI, scientific computing |

### ASCII Art Style

Use clean, professional ASCII art for diagrams:

```
┌─────────────────────────────┐
│  🎼 Batuta (Interface)      │
│ ┌─────────────────────────┐ │
│ │  ☀️ Pharaoh (Coord)     │ │
│ │ ┌─────────────────────┐ │ │
│ │ │ 🪲 Scarab + 🗿 Base │ │ │
│ │ └─────────────────────┘ │ │
│ └─────────────────────────┘ │
└─────────────────────────────┘
```

---

## 🌍 Attribution & Credits

**Always credit these inspirations**:

| What | Who/What | How We Use It |
|------|----------|---------------|
| **Scarab IDs** | Twitter's Snowflake (2010) | Distributed ID generation |
| **Batuta Syntax** | Clojure (Rich Hickey) | Lisp syntax, macros, persistent data |
| **Batuta Actors** | Elixir/Erlang (José Valim, Joe Armstrong) | Actor model, supervision trees |
| **Error Handling** | Zig (Andrew Kelley) | `Result!Type` error unions |
| **Reference Caps** | Pony (Sylvan Clebsch) | Compile-time data race prevention |
| **Actor-Reactor** | Stella Language | Topology-level reactivity |
| **Networking** | WireGuard | Quantum-resistant protocol |
| **Transactions** | TiKV's Percolator | Distributed transaction protocol |
| **Lang Dev** | Sulise | Complete PL development toolkit |

**Thematic Influences**:
- **Egyptian** (🗿☀️🪲): Ancient Egyptian engineering excellence (4,500+ years)
- **Latin** (🎼): Musical/orchestral tradition (precision + expression)
- **Natural** (🌲): Evergreen trees (permanence, growth, complete knowledge tree)

**Credit Format**:
```markdown
🪲 Scarab IDs are inspired by Twitter's Snowflake algorithm (2010)
🎼 Batuta draws from Clojure, Elixir, Zig, and Pony
🌲 Sulise: Complete programming language development toolkit (grammar, types, semantics, compilation)
```

---

## 📊 Use Cases for Branding

### Documentation

```markdown
# 🔺 Pyralog
> Built to Last Millennia

## 🗿 Obelisk Sequencer
The Obelisk Sequencer is a crash-safe...

## ☀️ Pharaoh Network
The Pharaoh Network eliminates...
```

### Blog Posts & Talks

**Title Format**:
- "The Obelisk Sequencer: A Novel Persistent Atomic Primitive"
- "🔺 Pyralog: Built to Last Millennia"
- "Pharaoh Network: Coordination Without Consensus"

**Slide Structure**:
1. Title: 🔺 + tagline
2. Problem statement
3. Four pillars: 🗿☀️🪲🎼
4. Architecture diagram
5. Benchmarks

### Social Media

**Twitter/X Template**:
```
🔺 Pyralog achieves 28B ops/sec with:
🗿 Crash-safe counters
☀️ Distributed coordination
🪲 Unique identifiers
🎼 Expressive queries

Built to last millennia. Zero bottlenecks.
```

**Badges**:
```markdown
![Pyralog](https://img.shields.io/badge/🔺-Pyralog-C2B280)
![Components](https://img.shields.io/badge/🗿☀️🪲🎼-Components-15317E)
```

### Code Documentation

```rust
/// The 🗿 Obelisk Sequencer provides crash-safe
/// persistent atomic counters.
pub struct ObeliskSequencer { /* ... */ }
```

---

## 🎯 Brand Positioning

### Taglines

**Primary**:
> **🔺 Pyralog: Built to Last Millennia**

**Alternatives**:
- "🔺 Pyralog: Monumental Performance, Eternal Data"
- "🔺 Pyralog: The Distributed Log for the Ages"
- "🔺 Pyralog: Solid Foundation, Infinite Scale"
- "🔺 Pyralog: 28 Billion Operations Per Second. Zero Compromises."

### Elevator Pitch

**30 seconds**:
```
🔺 Pyralog is a unified distributed data platform inspired by ancient 
Egyptian engineering. Like the pyramids, it's built to last—with 
crash-safe primitives (🗿 Obelisk Sequencer), distributed 
coordination (☀️ Pharaoh Network), unique identifiers (🪲 Scarab IDs),
and an expressive language (🎼 Batuta). It achieves 28 billion 
operations per second without centralized bottlenecks.
```

**5 seconds**:
```
🔺 Pyralog: Distributed data infrastructure with zero bottlenecks.
28 billion ops/sec. Built to last millennia.
```

---

## ✅ Brand Checklist

When creating Pyralog content, ensure:

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

- **v2.2** (2025-11-02): Added feature-level icons
  - Added 🎭 Theater Masks for Actor Model (theater performers = actors)
  - Added 🕸️ Spider Web for Distributed Systems (decentralized mesh)
  - Added feature icons: ⚡ Parallel, 🔒 Security, 🗄️ Database, 🧮 Tensors
  - Split icon reference into "Core Components" and "Feature Icons"
  - Added metaphor column to explain icon choices

- **v2.1** (2025-11-02): Added Sulise Evergreen language development toolkit
  - Added 🌲 Sulise as complete PL development foundation
  - Covers: grammar, types, semantics, compilation, theory
  - Evergreen tree = complete tree of language knowledge
  - Positioned beneath Batuta (provides all language primitives)
  - Updated all architecture diagrams to show 5 components
  - Added thematic influences (Egyptian, Latin, Natural)
  - Natural metaphor for academic/research/language science layer

- **v2.0** (2025-11-02): Major refactoring for clarity
  - Added comprehensive table of contents
  - Consolidated "Three Pillars" → "Four Pillars" (fixed inconsistency)
  - Streamlined component descriptions with comparison table
  - Simplified logo variations (4 clear options)
  - Condensed Attribution & Credits into table format
  - Reduced redundancy across sections
  - Improved scannability and navigation
  - Total changes: 580 lines modified (284 insertions, 296 deletions)

- **v1.2** (2025-11-02): Added Pyralog platform icon
  - Selected 🔺 Pyramid as primary platform icon
  - Represents layered architecture and lasting monument

- **v1.1** (2025-11-02): Added Batuta language
  - Added 🎼 Batuta as fourth component
  - Multi-cultural design philosophy (Egyptian + Latin)

- **v1.0** (2025-11-02): Initial branding guide
  - Established Egyptian theme
  - Defined infrastructure components: 🗿☀️🪲

---

## 📄 License

This branding guide is licensed under **CC0-1.0** (Public Domain).

You are free to:
- Use Pyralog's branding in presentations, articles, and documentation
- Reference the Egyptian theme in your own work
- Create derivative content about Pyralog

We encourage:
- Proper attribution to Pyralog and its inspirations
- Maintaining technical accuracy
- Educational use

---

**Questions?** Open an issue or join our [Discord](https://discord.gg/pyralog)

**🔺 Pyralog**: Built to last millennia. 🗿☀️🪲🎼

