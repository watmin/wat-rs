# Arc 002 — Rust interop macro — INSCRIPTION

**Status:** SHIPPED — predates the formal DESIGN.md / INSCRIPTION.md
arc-folder discipline (which firmed up around arc 003+).

The work shipped under the older convention of three sibling
documents:

- [MACRO-DESIGN.md](MACRO-DESIGN.md) — the design surface for
  the `#[wat_dispatch]` proc-macro that lifts Rust crate types
  into the `:rust::*` namespace. Generated `register` +
  `wat_sources` per crate, wired call-sites through
  `RustDepsBuilder`, and made arc 013's external-crate contract
  possible.
- [NAMESPACE-PRINCIPLE.md](NAMESPACE-PRINCIPLE.md) — the rules
  for how `:rust::*` paths mirror real Rust paths. The
  load-bearing axiom that "wat_dispatch is only necessary for
  external crates; anything in wat is always native" lives here.
- [PROGRESS.md](PROGRESS.md) — the live work log; every
  checkpoint shipped green with zero clippy warnings and full
  workspace test suite passing.

Effectively closed. This INSCRIPTION exists so the seal-status
survey reflects current reality (no implementation work
outstanding) without erasing the older doc-shape archaeology —
the three sibling files stay where they are.

**PERSEVERARE.**
