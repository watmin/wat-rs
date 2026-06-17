# Arc 276 — `deporder` as a consumable vendor tool (portable load-order verification)

> **STATUS: STUB — queued, non-blocking.** Surfaced 2026-06-17 by the builder while arc 275 (the
> `deporder` analyzer) was in flight. Not started. Blocked behind 275.1 (the engine) + 275.2 (our own
> enforcement + reorder). This file is the context breadcrumb so the arc can be picked up cold.

## The trigger

Watching 275.1 build `:wat::deporder::verify` as a **pure function of `Vector<SourceFile>`**, the builder
saw the dividend:

> *"dude… it's portable too… those who vend out libs can run this on their sources… we need to get this
> tool consumable in crates that vend wat."*

The analyzer doesn't care *whose* sources it verifies — our stdlib, a vending crate's `&[WatSource]`, or
the combined installed set, all run the identical algorithm. The only stdlib-specific atom is the
`:wat::stdlib::sources` accessor; `deporder` itself is universal. This arc is the **exposure/packaging**
that lets external crates consume it — the wat engine is reused as-is, nothing in the algorithm changes.

## Grounded facts (the vending mechanism — crawled 2026-06-17)

- **`WatSource` is public** (`src/source.rs:40`): `{ pub path, pub source }`, both `&'static str` so
  external crates construct them via `include_str!`.
- **Crates install their sources** via the dep-source mechanism: `install_dep_sources` /
  `installed_dep_sources` + the `DEP_SOURCES` `OnceLock` (`src/source.rs`). After install, every freeze
  sees them as part of `stdlib_forms`. The harness `tests/wat_harness_deps.rs` (DEP_A/DEP_B) is the
  worked test of this path.
- **The 275.1 engine** `:wat::deporder::verify (files: Vector<SourceFile>) → Vector<Violation>` is the
  reusable core (pure, no I/O). The 275.1 `:wat::stdlib::sources` intrinsic exposes only OUR baked
  `stdlib_files()`.

## Design sketch (to four-question when the arc opens)

1. **Rust drop-in for vendors.** A convenience
   `pub fn verify_load_order(sources: &[WatSource]) -> Result<(), Vec<Violation>>` that runs the wat
   `deporder` engine over a given source set. A crate that vends wat adds ONE build test over its own
   `&[WatSource]` array → its load order is checked at `cargo test`, the same red-build guarantee we get
   for the stdlib. (Decide the `Violation` Rust shape / how it crosses the wat→Rust boundary — likely
   reuse the EDN bridge the engine already returns through.)
2. **Optional: a dep-aware accessor** (baked + installed sources, in load order) so a running program
   can verify the *whole* installed order — catching **cross-crate** ordering bugs (a dep source that
   eval-depends on something loaded later). This generalizes `:wat::stdlib::sources`; weigh whether it's
   a second accessor or a flag.

## Scope / discipline

- **Non-blocking, behind 275.** Do not start until 275.1 (engine) + 275.2 (our enforcement + reorder)
  land — the engine + its proof must exist first.
- **Reuse the engine as-is.** This arc adds NO algorithm; it is exposure only. If it tempts an engine
  change, that change belongs in 275, not here.
- Open it on the arc rhythm: ground the dep-source API, four-question the drop-in shape, a disconfirming
  probe (a fixture crate's `&[WatSource]` with a deliberate violation → `verify_load_order` returns it),
  build via shadowdancer + weigh.
