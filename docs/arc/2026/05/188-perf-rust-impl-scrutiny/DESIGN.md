# Arc 188 — Perf + Rust impl scrutiny ("beat the shit out of the impl")

**Status:** stub opened 2026-05-13 per user direction.
**Gates on:** arc 187 (post-109 modularization) closure.

## Motivation

> *"building on MODULARIZATION-NOTES.md -> another arc where we just beat the shit out of the impl. i want the perf and rust decisions to be massively scrutinized. we wrote the best rust EDN lib in a few hours banging on the code... there's still more to do there."*

After arc 187 lands clean module boundaries, every module gets scrutinized for:

- **Allocation profile** — heap touches per op; can stack/Arc-share suffice?
- **Hot-path measurement** — profiling representative workloads (typecheck pass over a non-trivial program, freeze, eval-heavy benchmarks)
- **Rust-idiom honesty** — `Clone` where `&` would do? `String` where `&str` works? Boxed enums where `#[repr(C)]` could pack? Unbounded `Vec` growth where `SmallVec` fits the 95th percentile?
- **Zero-Mutex compliance** — confirm no Mutex/RwLock crept in; verify Arc + ThreadOwnedCell + program-with-mailbox tiers are honored
- **SIMD where it matters** — VSA primitives in holon-rs already use SIMD per CLAUDE.md; verify substrate paths that flow through similar shape exploit it
- **Cache-friendliness** — struct layouts; hot fields adjacent; avoid pointer-chase tax

The precedent: wat-edn was hand-rolled in hours and outperforms existing Rust EDN crates. The pattern transfers — when the architecture is clean (post-187) and the substrate is whole (post-109), the Rust impl can be massaged with surgical care for measured wins.

## Sketch

Per-module sweep:

1. Pick a module (post-187 boundary)
2. Profile it under representative load
3. Identify the top 3 allocations / dispatches / hot paths
4. Surface in a per-module SCORE doc; pick the work that earns its measurement-driven justification
5. Land the change with before/after benchmark numbers in the SCORE

Refuse speculative micro-optimization. Every change ships with **measured improvement** (≥5% on the targeted hot path, or it doesn't land). Substrate-as-teacher applies at the perf layer too — the profiler is the teacher.

## Cross-references

- Arc 187 (modularization) — must close first
- Arc 189 (wat-edn streaming) — possible sub-arc; surfaced as parenthetical user note
- `docs/MODULARIZATION-NOTES.md` — informs which modules get scrutiny order
- Holon-rs SIMD (CLAUDE.md "5x similarity speedup with --features simd") — precedent for measured-improvement-per-change discipline
- wat-edn (`crates/wat-edn/`) — the "wrote best Rust EDN lib in hours" reference
