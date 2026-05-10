# Arc 170 — The hermetic concept

> Note: this doc captures a substrate-concept realization that
> surfaced during slice 1 review (2026-05-09). It articulates the
> framing that makes spawn-thread / spawn-process /
> spawn-remote-program a single coherent pattern, not three
> bespoke things. See [`REALIZATIONS-SLICE-1.md`](./REALIZATIONS-SLICE-1.md)
> for the slice-1 deficiency that opened the conversation.

## Same interface; different runtime envs

A program is a fn satisfying a contract:

```
(fn [<inputs>] -> :nil <body>)
```

The signature IS the contract. The fn IS the program. That stays
constant across thread / process / remote-program variants.

What CHANGES is the runtime environment — specifically, what the
spawned fn shares with the spawner:

| Variant | Shares | Memory | Host | Network |
|---|---|---|---|---|
| **Thread**   | memory  | ✓ same wat world | ✓ same OS process | ✓ same machine |
| **Process**  | host    | ✗ forked (COW)   | ✓ same OS host    | ✓ same machine |
| **Remote** *(future)* | network | ✗               | ✗                 | ✓ same socket-reachable |

Each variant is "less shared" than the previous. The fn doesn't
know — it sees the same input shape (channels for Thread; pipes
for Process; sockets-as-multiplexed-channels for Remote). The
substrate handles the boundary mechanics.

## Hermetic = the package that bridges memory boundaries

A **hermetic package** is the substrate-level primitive that lets
a fn cross a memory boundary. It carries:

- `prologue` — the captured environment (type defs + dep defs +
  captured value bindings)
- `entry_form` — the expression that evaluates to a fn Value
  (anonymous fn-form for inline lambdas; symbol AST for named
  functions)

The package is **self-contained**. Any context with a fresh wat
world (which carries substrate primitives by default) can:

1. Freeze the prologue → seeded type registry + symbol table
2. Eval `entry_form` in that frozen world → fn Value
3. Apply the fn Value to args → invocation

No external references survive. The seal is complete.

## When the seal matters

| Variant | Hermetic seal needed? | Why |
|---|---|---|
| Thread   | NO  | shared memory; the fn Value is used directly from parent's world |
| Process  | YES | forked process has parent's memory at fork-time but not after; the seal is the reproducible substrate state for the child to freeze |
| Remote   | YES | no shared memory at all; the seal serializes (via wat-edn) + ships over the wire |

Threads don't need the package because the fn Value lives in the
shared wat world; the parent passes the Value directly to the
worker thread. Processes and remote programs need the package
because the spawned context starts from a clean slate and must
reconstruct the fn-with-its-environment from a portable
description.

## The concept generalizes

The hermetic package is broader than "what spawn-process needs
internally." It's the fundamental substrate primitive for
**packaging a fn for transport** — independent of the transport
mechanism.

Use cases the same primitive serves:

| Caller | Transport | What it does with the package |
|---|---|---|
| `spawn-process`            | fork(2)         | Pass `prologue` to child via `fork-program-ast`-style pathway; child freezes; child evals `entry_form`; child applies |
| `spawn-remote-program` *(future)* | socket   | Serialize package via wat-edn → wire bytes; remote deserializes + freezes + evals + applies |
| Disk persistence *(future)* | filesystem    | Serialize package; save; later: load + freeze + eval + apply |
| Test fixture replay *(future)* | in-process | Capture a fn from a running test; package it; replay in a controlled fresh world |
| `wat/std/hermetic.wat` *(today)* | fork(2) | TODAY: wraps `fork-program-ast` with a string-source. ALTERNATIVE: rebuild as thin wrapper over `(spawn-process fn)` — uniform with arc 170's surface |

Today's `wat/std/hermetic.wat` is the SPECIFIC, wat-level case of
the generic substrate-level hermetic primitive. After arc 170
ships, it could be re-expressed as a thin wat-level wrapper
around `spawn-process` — one less special-case in the surface.

## Why this matters for arc 170

Arc 170 spec'd closure extraction as the substrate primitive
spawn-process needs internally. The realization is that this
primitive ALSO IS the substrate's hermetic-package mechanism.
Same code; broader frame.

For arc 170 itself, scope doesn't change:
- Closure extraction stays Rust-internal (DESIGN line 84-85)
- Wat-level surface in arc 170 is just `(spawn-process fn)` /
  `(spawn-thread fn)`
- Future arcs may surface the hermetic package as its own wat-
  level concept

What this framing buys us:
- Three spawn variants become a single coherent pattern, not
  three bespoke things
- Future remote-program arc inherits the hermetic primitive
  ready-built — it just adds transport (socket) + protocol
  (Q-channel multiplex)
- Future "package a fn for disk / replay / capability-bridge"
  use cases can build on the same primitive without re-inventing
- Existing `wat/std/hermetic.wat` becomes a candidate for
  rebuild on the uniform foundation (a future arc, not arc 170)

## Connections to existing memory

- **`project_wat_binary_hologram.md`** — "the binary IS the surface
  between Rust universe and wat universe... Holograms nest via
  spawn-program." Closure-extraction packages ARE the smaller
  holograms nested inside the binary hologram. Each spawn creates
  another nested hologram; the hermetic seal is what makes the
  nesting work cleanly.
- **`project_pipe_protocol.md`** — "line-delimited EDN + kernel
  pipes. One protocol; four transports." The hermetic package is
  the analogue at the program level: one PACKAGE shape; many
  transports (in-thread / fork / remote / disk).

## Open questions for future arcs

- Should the hermetic package itself be a wat-level value type
  (e.g., `:wat::kernel::Hermetic`)? Currently Rust-internal;
  future arc may surface it.
- Should `wat/std/hermetic.wat` get rebuilt on top of
  `spawn-process(fn)` once arc 170 ships? The wrapper is a
  string-source convenience over fork-program-ast; once
  spawn-process exists, the wrapper becomes uniform with the
  rest of the spawn surface.
- Does the hermetic package wire format (the EDN serialization
  of `prologue` + `entry_form` via wat-edn) need its own
  versioning / compatibility story? Comes up when remote-program
  arc opens.

## What this doc does NOT do

- Doesn't change arc 170 scope
- Doesn't open a new arc
- Doesn't propose substrate edits beyond what slice 1b already
  documents
- Doesn't surface a wat-level hermetic-package value type (that
  would be a future arc if useful)

This is a substrate-concept capture — names the pattern so future
arcs can reference it without re-deriving the framing.
