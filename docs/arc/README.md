# Arc log

Durable records of the major work arcs through wat-rs. Each arc is a
directory under `YYYY/MM/NNN-slug/` holding design docs, progress
logs, and post-mortems for the work that shaped the current code.

Arcs here are complementary to the 058 language specification (which
lives in `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/`).
058 is the authoritative language semantics; arc/ here captures the
*implementation* journey — how wat-rs reached the shape it has and
why a decision was made the way it was.

## Current arcs

- **2026/04/001-caching-stack** — L1 / L2 cache design. `DESIGN.md`
  captures the architecture (LocalCache + Cache program), the zero-
  Mutex discipline, and the thread-ownership invariant.
  `DEADLOCK-POSTMORTEM.md` records the first concrete
  thread-ownership bug we hit and the principle that prevents
  recurrence.
- **2026/04/002-rust-interop-macro** — The `:rust::` namespace and
  `#[wat_dispatch]` proc-macro that together let wat programs use
  any Rust crate. `MACRO-DESIGN.md` is the authority on how the
  macro works. `NAMESPACE-PRINCIPLE.md` codifies how `:wat::` and
  `:rust::` coexist. `PROGRESS.md` is the live checklist of what's
  shipped versus what's next.

## When to add an arc

A new directory belongs here when work spans multiple commits with
a shared goal and produces design decisions or principles we want
future sessions to read cold. Single-commit slices don't need an
arc entry — the commit message is enough.
