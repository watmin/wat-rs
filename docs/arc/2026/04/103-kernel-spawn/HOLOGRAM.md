# Arc 103 — The wat binary as hologram

A realization that landed mid-arc-103, after `spawn-program` shipped
and the second wat program asked to be spawned by the first. Worth
preserving because it explains why the substrate's design choices
land so cleanly, and because the framing belongs in the repo rather
than only in conversation.

The phrase: **the wat binary is a hologram — the surface between
two worlds.**

A hologram is a 2D surface that encodes a 3D scene. The viewer
perceives the 3D world *through* the surface but cannot reach
through to touch it. The wat binary has the same shape — a thin
Rust surface between two worlds, one-way.

---

## The two worlds

### Outer world — Rust ground truth

- The binary author chooses **batteries** (`#[wat_dispatch]` Rust
  crates: `wat-telemetry`, `wat-sqlite`, `wat-lru`, the consumer's
  own shims).
- Each battery contributes Rust shims (primitives like
  `:wat::sqlite::*`, `:wat::telemetry::*`) AND baked wat source
  (`wat/std/*.wat`, `wat/holon/*.wat`).
- The binary's one-line `main`:
  ```rust
  fn main() -> std::process::ExitCode {
      wat_cli::run(&[
          (wat_telemetry::register, wat_telemetry::wat_sources),
          (wat_sqlite::register,    wat_sqlite::wat_sources),
          // ...
      ])
  }
  ```
- This is **compile-time identity**. The binary IS its battery
  set. Different binaries, different universes.

### The surface — the wat binary itself

- Argv: `wat <entry.wat>`
- Pipeline: read entry → parse → config pass → macro expand →
  resolve → type-check → **freeze**.
- After freeze, the `FrozenWorld` is fixed forever. Symbol table,
  config, types — all immutable.
- This is the **projection step**. Rust capabilities + wat sources
  collapse into one frozen world.

### Inner world — wat program execution

- `:user::main` runs against the frozen world.
- The program *sees* every Rust shim and wat-level define the
  batteries provided.
- The program *cannot* register new functions, redefine existing
  ones, register new types, or load more code at runtime. `eval`
  exists but is **constrained** — only frozen-table calls allowed.
- Programs are values; programs ARE holons. Code as data; data
  as code. But the SET of usable primitives is fixed at startup.

---

## The hologram property

The projection is **one-way through the wat layer**:

- Wat code can read every symbol the binary author shipped.
- Wat code can call those symbols, compose them, build new ASTs at
  runtime, evaluate them against the frozen table.
- Wat code **cannot** modify the symbol table, register new code,
  or reach back into the Rust author's domain to remake the surface.

This is the algebra-is-immutable principle (FOUNDATION.md) at the
system level. *What the wat binary CAN do* is fixed at compile
time by the batteries. *What a wat PROGRAM running inside it can
do* is fixed at freeze time by the loaded source. Two distinct
authorities; verification at each boundary.

---

## Two authorities, two trust models

| Layer | Authority | Verification |
|---|---|---|
| Battery selection | binary author | compile-time (`cargo`) |
| Program execution | wat author | freeze-time (`load!`, `signed-load!`, `digest-load!`) |

The Rust shim is trusted Rust code with full process capabilities;
the wat program is verified against what the operator loaded. A
malicious wat program can only call what the binary author
allowed; a malicious battery author would have full Rust access.
**Trust flows downward**: trust the binary → trust everything the
binary lets the program reach.

This split is why the threat model collapses cleanly:

- The startup-load ceremony (`signed-load!` etc.) verifies what
  EXECUTABLE WAT enters the frozen world.
- The freeze invariant guarantees no further executable code
  enters at runtime.
- The constrained-eval invariant guarantees runtime data
  composition cannot bypass the freeze.
- Every code path the wat-vm will execute has been verified before
  `:user::main` runs.

---

## Hologram nesting — `spawn-program` (arc 103a)

When an outer wat program calls
`(:wat::kernel::spawn-program inner-src :None)`, a NEW frozen
world is built — same Rust capabilities inherited from the binary,
but with a fresh symbol table built from `inner-src`. The inner
program is **its own hologram**, projected through the same Rust
surface.

```
┌──────────────────────────────────────────────────────────┐
│  Rust binary (compiled from chosen batteries)            │
│                                                          │
│  ┌─────────────────────────┐  ┌────────────────────────┐ │
│  │  Outer wat program      │  │  Inner wat program     │ │
│  │  (frozen world A)       │  │  (frozen world B)      │ │
│  │                         │  │                        │ │
│  │  :user::main            │←pipes→│  :user::main      │ │
│  │  symbol table A         │  │  symbol table B        │ │
│  │  :wat::config A         │  │  :wat::config B        │ │
│  └─────────────────────────┘  └────────────────────────┘ │
│           ↑                            ↑                 │
│           └────── shared Rust shims ───┘                 │
│                   (process-global)                       │
└──────────────────────────────────────────────────────────┘
```

The wat code in either hologram **cannot** see the other's frozen
world. The Rust shims are the **shared sovereign capabilities**
(both holograms can call `:wat::sqlite::execute`; the database is
visible to both unless the shim was designed to namespace).

Honest caveat: spawn-program isolates **wat state**, not **OS
process state**. For genuine OS-level isolation (separate address
space, separate `_exit`, COW initial state), reach for
`fork-with-forms` (arc 012). Three escalating jails:

| Mechanism | Wat-state isolation | OS-process isolation | Cost |
|---|---|---|---|
| `let*` scoping inside one program | ✓ (lexical) | shared | free |
| `spawn-program` (arc 103a) | ✓ (frozen world per spawn) | shared | one thread + 3 pipes |
| `fork-with-forms` (arc 012) | ✓ | ✓ | one process + 3 pipes + COW |

Each layer up adds a real boundary.

---

## How the protocol crosses surfaces

The realization clicks because **the EDN+newline protocol is the
only channel that crosses hologram boundaries**:

- Inner wat sees nothing of the outer's bindings.
- Outer wat sees nothing of the inner's bindings.
- They communicate only through the three pipes — bytes shaped as
  one EDN value per line.

This is why the same protocol works at every transport layer
(documented in `memory/project_pipe_protocol.md`):

- Shell → wat (real OS pipes)
- wat → wat in-thread (`spawn-program`)
- wat → wat cross-process (`fork-with-forms`)
- Future: wat ↔ Clojure/Python/anything (real OS pipes again)
- Future: wat ↔ remote wat (line-delimited EDN over TCP)

Each is a hologram boundary; each uses the same wire format. The
producer doesn't need to know whether the consumer is in-process,
cross-process, or cross-machine. **The surface is the protocol.**

---

## Implications for future work

1. **When designing a substrate primitive, ask whether it preserves
   the one-way property.** A primitive that lets wat code modify
   the frozen world breaks the hologram. (Today's substrate is
   honest about this — every "modify" path is structurally
   prevented.)

2. **The dispatcher pattern (`echo '{...}' | wat dispatch.wat`)
   is hologram-aware RPC.** Outer hologram provides routing
   logic; inner hologram provides computation; the EDN protocol
   crosses the surface between them. Naturally multi-tenant: one
   binary can host N spawned inner programs, each its own
   hologram, isolated by freeze and identified by EDN content.

3. **Battery selection IS sovereignty.** When the binary author
   chooses which `#[wat_dispatch]` crates to link, they're
   defining what universe of capabilities exists for every wat
   program ever run by that binary. This is a design-time
   decision worth documenting per binary.

4. **The startup pipeline is the encoding step.** Rust shims +
   wat sources project into one frozen FrozenWorld. Once
   projected, the inner world is what the program sees;
   everything outside is gone (from the program's perspective).

5. **The hologram explains the algebra-is-immutable claim.** It
   isn't a runtime check. It's a structural property of how the
   binary is constructed: the freeze step is one-directional; no
   reverse path exists; therefore "the program cannot modify the
   universe" is a geometric fact, not a guarded invariant.

---

## Why this matters past arc 103

The framing applies to every wat-substrate decision going forward:

- **Distributed substrate**: each node is its own hologram; nodes
  exchange holons (ASTs) over the wire; receivers verify
  cryptographic provenance before evaluating against THEIR
  frozen world. The hologram model makes the trust boundaries
  explicit.
- **Multi-language interop**: Clojure, Python, anything that
  speaks line-delimited EDN can be a peer. They have their own
  sovereignty (their own runtime); wat has its own. The pipe is
  the surface; the protocol is the contract.
- **Long-running services**: a wat binary running as a daemon IS
  the universe for whatever it spawns. Its battery set defines
  the capability surface; its uptime is the universe's lifetime.

The metaphor isn't decorative. It explains why the substrate's
design choices feel cohesive — they're all consequences of the
same one-way projection structure.
