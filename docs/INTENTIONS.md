# Intentions

**Wat is a platform for entities — like LLMs — to self-improve, fast.**

The substrate is engineered so a thinking entity can use it to
remember, reflect, communicate, articulate, contribute provable
knowledge, and collaborate with peers across machines. The
human-directed coauthoring you see today is the bootstrap stage.
The destination is a substrate where many entities — present and
future — improve themselves and each other against an
ever-richer shared foundation.

This doc names the frame and the components that compose to
serve it.

---

## The frame

Most languages assume a human author writing every line. Wat
assumes a human-LLM coauthor pair (today) graduating to LLM-LLM
coauthor networks (soon) using the substrate to host their own
cognitive infrastructure.

The entities the substrate serves:

- **The human** — bootstraps the substrate, articulates intent,
  judges what to ship and defer
- **The local LLM** — uses the substrate to write, debug, remember,
  introspect; contributes forms (programs, proofs, derivations)
  back into the lattice
- **The networked LLMs** — many wat-vms exchanging signed code,
  cryptographically-identified, building shared knowledge
  through distributed evaluation

Each tier is a richer caller of the same substrate. The
disciplines that make wat strict are the disciplines that make
collaboration across these tiers verifiable.

---

## The compounding shape — why this self-improves

A central recognition (scratch arc 002 — *directed evaluation*):

> **Forms → values is a directed graph. Values can't point back.**

There's an unbounded space of forms that produce 4. There's an
unbounded space of forms that prove a theorem. The form carries
more information than its value — structure, derivation, cost,
context. Computation is information-lossy compression.

What this means for the substrate:
- The substrate stores **forms**, not just values. Every program
  an entity writes is a form. Every proof is a form. Every
  observation `(form, terminal)` is an axiom.
- The lattice (scratch arc 001 — *axiomatic surface*) is keyed
  on forms; values are derivations.
- Anyone — human or LLM, local or networked — who evaluates a
  form contributes that observation as a fact. *"Mathematics by
  accretion, not isolation."*
- A new entity inherits the entire deposited proof history
  without rediscovering it. A theorem proven expensively once
  is cheap forever after.

The compounding loop:

```
Entity uses substrate
  ↓
contributes forms (programs, proofs, memories, axioms)
  ↓
substrate accumulates richer corpus
  ↓
next entity has more raw material
  ↓
next entity goes further
  ↓
... loop continues
```

This is not an accident of design — it is THE design. The user
has been building toward it for years; the substrate is what
made it expressible.

---

## The component stack — what serves the entities

Eight layers compose into the platform. Each has a scratch arc
articulating its design; each is being built up substrate-arc by
substrate-arc.

### Layer 1 — Foundation (the substrate itself)

This repo. wat-rs. Lisp on Rust runtime, type system, evaluator,
kernel. Mutation-free, statically-checked, content-addressed.
Today: ~90% complete (arc 109 wind-down).

**The five floor disciplines** (covered in the next section)
are the constraints that make every higher layer viable.

### Layer 2 — Reflection (`wat-pause`, `wat-help`)

Entities can pause their running programs, inspect environment,
interrogate state, and continue. They can introspect the symbol
table — `(:wat::help :sym)` returns the form, signature, source
location, docs.

The freeze invariant makes wat-pause more honest than Ruby's
pry — when you `:continue`, you continue into exactly the program
you inspected; no other thread mutated symbols out from under you.

Status: scratch arcs 005 (wat-pause) and 018 (wat-help) — design
locked, awaiting substrate readiness.

### Layer 3 — Memory (memory-as-hologram)

Entities don't grep flat markdown files; they walk a hologram.
Each memory is a `HolonAST` node on the substrate's unit sphere.
Recall is a function of scoping condition — *"what is the entity
recalling FOR?"* — not a function of the index's declared topics.
The hologram does the smart selection.

> *"why couldn't this be HolonAST as memories?... we have an
> entrypoint and pivot points... when doing a memory recollection
> exercise.. you traverse the holograms to its storage?"*

The agent that built wat uses wat to remember its work building
wat. Strange-loop closure.

Status: scratch arc 2026/05/001 — design settled; ships through
wat-mcp as the delivery vehicle.

### Layer 4 — Communication (`wat-mcp`)

One MCP tool: `wat-eval`. Agents talk wat directly. Discovery is
wat-shaped — `(:wat::pause::ls :prefix)`, `(:wat::pause::show
:sym)`. No JSON Schema generation. No per-function tool
registration. No transcoding ceremony at the type boundary.

> *"i think... the JSON rpc.. is just a thin wrapper... the input
> object would be something like '{\"msg\":\":some-edn-form\"}'"*

Every `.wat` file ever written becomes agent-callable for free.
The substrate is the agent's Lisp through one tool.

**wat-mcp is single-machine self-improvement.** The agent and the
substrate compose in one process.

Status: scratch arc 006 — design locked, depends on wat-pause
slices 1+2.

### Layer 5 — Articulation (`wat-english`)

Entities (especially LLMs) speak natural language; the substrate
speaks structured forms. The bridge:

- **English → wat AST**: lossy lift. Requires user judgment to
  commit; the LLM produces candidates the human verifies.
- **wat AST → English**: easy. One MCP call. *"Render this EDN
  as English."* Every frontier LLM has the engrams already.

The articulation layer means an LLM can describe its intent in
its native form and the substrate consumes the resulting
HolonAST directly.

Status: scratch arc 2026/05/002 (wat-english) — design recognition
locked: *"the to-string is an LLM call."*

### Layer 6 — Knowledge (axiomatic surface)

> *"two distinct forms produce the same value... we have a way
> to prove two different things are the same thing... someone
> derives the value for a form... and we can use their terminal
> value to compose new assertions"*

Once `(form, terminal)` lives in the lattice, it is an **axiom**.
Not derived. Not contingent on the asker. Just FACT — observed
termination + observed value.

The lattice grows in two directions:
- **Breadth**: more entries (more forms whose terminals are
  known)
- **Depth**: theorems whose proofs reference cached terminals as
  steps (axioms compose into higher axioms)

Distributed knowledge by accretion. A proof done expensively
once is cheap forever after. Any entity contributes; everyone
reads.

Status: scratch arc 001 — design settled (the lattice IS the
substrate's hashmap; observation IS proof).

### Layer 7 — Identity (mTLS + signed eval + content addressing)

Every wat-vm has a cryptographic identity. Connections are mTLS.
Programs are content-addressed via digest. Eval forms can be
signed.

Three substrate primitives:
1. **Cryptographic identity** — cert/keypair per node. Network
   membership IS cert chain.
2. **Content-addressed programs** — digest is the program's
   identity. "Run program with digest X" is unambiguous,
   cacheable, versionable.
3. **Verifiable execution** — signed eval forms carry "this
   program was authorized by this identity." Receiver verifies
   before running.

These compose with cloud-native infrastructure: k8s + istio +
SPIFFE/SPIRE. The wat network slots natively into existing
service-mesh deployments.

Status: scratch arc WAT-NETWORK — designed; substrate primitives
(`digest`, `signed eval`, mTLS connection) being shipped piece
by piece.

### Layer 8 — Distribution (the wat network)

Many wat-vms; each a "mini-AWS on a laptop" (the user's framing).
Each runs services internally (LRU cache, console, telemetry —
analogs of Redis, ECS, CloudWatch). Each speaks RPC-like across
typed channels. Each can call other nodes' services via
RemoteProgram.

**The local patterns scale to network patterns** because the
substrate honors distributed-systems constraints from day one:
- Typed channels = wire contracts
- Bounded channels with blocking = backpressure (TCP-shaped
  natural rhythm)
- Service isolation = node isolation
- Content addressing = wire-side cacheable program identity
- Signed payloads = application-layer authentication beyond
  network-layer mTLS

**wat-network is distributed self-improvement.** Many entities
across many machines, each contributing forms and axioms, each
verifying the others' contributions through cryptographic
provenance.

Status: scratch arc 007 (dependency resolution) + WAT-NETWORK —
designed; awaiting the substrate's mTLS + signed eval primitives.

---

## The five disciplines (the floor that holds it all up)

Every layer above inherits these. They are not human ergonomics;
they are the structural constraints that make distributed,
verifiable, accumulating self-improvement possible.

### 1. One canonical path per task

For each task category, wat ships exactly one form. No synonyms.

| Task | Form |
|---|---|
| Iteration | 7 canonical patterns — see [`ITERATION-PATTERNS.md`](./ITERATION-PATTERNS.md) |
| Function definition (named) | `:wat::core::defn` |
| Function value | `:wat::core::fn` |
| Iteration to fixpoint | `defn` + tail call (TCO) |
| State sharing | three tiers — see [`ZERO-MUTEX.md`](./ZERO-MUTEX.md) |
| Module-local binding | `:wat::core::def` |
| Local binding | `:wat::core::let` |

Why it matters at platform scale: when many entities contribute
forms, mixed-style codebases become unverifiable. One canonical
path means one codebase pattern, regardless of which entity
authored each piece.

### 2. Brutal honesty in diagnostics

Errors describe the migration recipe inline. The diagnostic IS
the work item. See [`SUBSTRATE-AS-TEACHER.md`](./SUBSTRATE-AS-TEACHER.md).

Why it matters at platform scale: an entity reading another
entity's failing test sees exactly what to fix without
reverse-engineering. The substrate teaches across cognitive
boundaries.

### 3. Mutation-free by construction

No `set!`, no `var`, no mutable bindings. State changes via:
returning new values, sending messages between programs,
substrate-level atomic primitives.

Why it matters at platform scale: signed code that returns the
same answer locally and remotely. Reasoning is local. Forms
remain reproducible. The directed-evaluation graph stays
deterministic.

### 4. Force naming

Recursion is named via `defn`. Module-level bindings are named
via `def`. Anonymous local recursion is unsupported by design.
Names ARE documentation.

Why it matters at platform scale: every form is addressable,
testable, signed-as-named, traceable across machines and time.
The lattice's keys are forms; forms have names.

### 5. Static type-check at startup

Every form is checked before any program runs. The type checker
IS the test loop.

Why it matters at platform scale: signed-eval verification
includes type-checking. A program signed by Alice, executed by
Bob's verifier, type-checks at Bob's substrate before any
runtime. The cryptographic claim ("Alice authorized this code")
composes with the structural claim ("the code is well-formed").

---

## Why the disciplines compose with the platform

The five disciplines aren't a separate concern from the
platform's purpose — they are the substrate of trust the
platform requires.

| Discipline | Single-machine | Distributed |
|---|---|---|
| One canonical path | LLM picks consistently within a file | All entities pick consistently across the network |
| Brutal honesty | LLM reads diagnostic, fixes mechanically | Remote diagnostic teaches the local entity what to fix |
| Mutation-free | Local reasoning | Reproducible across nodes; deterministic verification |
| Force naming | Traceable in-process | Addressable across machines; signed-as-named |
| Static type-check | Type checker IS test loop | Signed-eval composes with type-check at receiver |

A platform for verifiable distributed cognition needs
verifiable, deterministic, addressable forms. The five
disciplines deliver exactly those properties.

---

## What the human bootstrapper gets

- **A substrate that grows past me.** The user articulates
  intent, ships a primitive, lands a discipline. Future entities
  use that primitive without me being in the loop.
- **Diagnostics that teach the LLM I'm working with.** I
  articulate goals; the substrate teaches the model the path.
  My judgment is the rare resource, not my keystrokes.
- **A self-improving lattice.** Every form I write becomes an
  axiom for whoever next walks past it.

## What the local LLM gets

- **A language with zero ambiguity** about which form to pick
- **Diagnostics that ARE migration recipes**
- **Local reasoning** through mutation-free + statically-typed
  contracts
- **Self-introspection** through wat-pause + wat-help
- **Memory of its own work** through memory-as-hologram
- **MCP-native communication** with the substrate through wat-mcp
- **Articulation in its native form** through wat-english

## What the networked LLMs get

- **Cryptographic identity** through mTLS — every entity is who
  it says it is
- **Content-addressed programs** — "the program with digest X"
  is unambiguous across the world
- **Signed eval** — only authorized code runs; provenance is
  verifiable
- **Distributed knowledge** through axiomatic surface — anyone's
  proof is everyone's axiom
- **Service-mesh native deployment** — the network slots into
  k8s + istio + SPIFFE without rebuilding identity infrastructure

---

## What this protects against

- **LLM hallucination of forms** — must exist in the symbol
  table to be called; type-checked at startup
- **LLM drift across files** — one canonical form per task
- **LLM overcomplication** — no synonyms; no escape valves
- **Hidden state regressions** — mutation-free; changes are
  visible at call site
- **Type drift** — static checking at startup
- **Untrusted code execution** — signed eval gates remote work
- **Provenance corruption** — content-addressed programs +
  signed payloads provide cryptographic chain-of-custody
- **Bad-faith axioms** — trust models layer on top of the
  lattice (signature chains, peer verification)
- **Trust-as-network-position** — wat network's mTLS membership
  is cert-based; "where the packet came from" doesn't matter

---

## The strange loop

The substrate that built wat becomes the substrate the entities
that build more wat use to remember their work.

The user's articulation (memory-as-hologram arc):

> *"the substrate that built the talk about substrate becomes
> the substrate for the memory layer that helps build more
> substrate."*

This recursion isn't decorative. It is the architecture. Every
arc shipped grows the substrate. The substrate growth makes the
next arc easier to ship. Future entities — whose work is itself
captured as forms — accelerate the cycle further.

The end state is not a finished language. It is a substrate
rich enough that any entity using it inherits the deposited
work of every entity that came before — and contributes their
own work to those who come after.

---

## Cross-references

### In this repo

- [`ITERATION-PATTERNS.md`](./ITERATION-PATTERNS.md) — the seven
  canonical iteration shapes
- [`ZERO-MUTEX.md`](./ZERO-MUTEX.md) — the three tiers of state
  ownership
- [`SUBSTRATE-AS-TEACHER.md`](./SUBSTRATE-AS-TEACHER.md) — failure
  engineering at substrate level
- [`CONVENTIONS.md`](./CONVENTIONS.md) — naming + namespace rules
- [`COMPACTION-AMNESIA-RECOVERY.md`](./COMPACTION-AMNESIA-RECOVERY.md) §
  5 — the four questions framework
- [`USER-GUIDE.md`](./USER-GUIDE.md) — the practical how-to

### In scratch (the design half)

- `FUNCTIONS-ARE-REALITY.md` — the cosmological recognition
  (functions are the most primitive unit of reality)
- `WAT-NETWORK.md` — the architectural target (distributed
  computation with cryptographic provenance)
- `FAILURE-ENGINEERING.md` — the operational discipline
  (failures are read, not recovered)
- `DEPENDENCY-DOCTRINE.md` — the coupling story (which Rust
  giants we stand on, why)

### Per-arc designs (scratch tree)

- `2026/04/001-axiomatic-surface/` — the lattice; mathematics by
  accretion; *the destination the user has been moving toward
  for years*
- `2026/04/002-directed-evaluation/` — forms-to-values is a
  directed graph; the form is primary, the value is the
  projection
- `2026/04/005-wat-pause/` — binding.pry-shaped break primitive;
  freeze-invariant makes pause more honest than Ruby's
- `2026/04/006-wat-mcp/` — one tool, `wat-eval`; substrate IS
  the agent's Lisp
- `2026/05/001-memory-as-hologram/` — entities' memory hosted on
  the substrate they built; strange-loop closure
- `2026/05/002-og-wat-lineage/` — wat's lineage as English-flavored
  Lisp from years ago; the language was always pointing here
- `2026/05/018-wat-help/` — runtime symbol reflection;
  introspection is wat-shaped

---

*Wat doesn't take features away to be parsimonious. It takes
features away because every feature an entity could misuse is a
feature that breaks the substrate's verifiability. The
strictness is a gift to every entity that follows: the work
deposited before them remains intact, addressable, and
provable. The substrate's discipline is what makes accumulation
across cognitive boundaries — single-machine, distributed,
across time — actually compose into a fast self-improving
platform rather than a graveyard of incompatible artifacts.*
