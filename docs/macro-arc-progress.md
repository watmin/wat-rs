# Macro arc — live progress log

Updated as work happens. If compaction hits mid-slice, read this to resume.
Full design: `docs/wat-dispatch-macro-design-2026-04-19.md`.

## Completed

- **Task #184** ✓ — typed `:wat::core::vec` / `::list` constructors (c7f27d3)
- **Task #185** ✓ — typed `:wat::std::HashMap` / `::HashSet` (9d760aa)
- **Task #186** ✓ — reverted in-runtime Cache scaffold (c7f27d3)
- **Task #187** ✓ — LocalCache as pure wat source over `:rust::lru::LruCache`
  (6a38366, 45ecf08, cac8f75, 4bb719f — four sub-commits)
- **Task #190** ✓ — workspace conversion. Root Cargo.toml is now
  `[workspace] + [package]`; `wat-macros/` sibling crate added.
- **Task #192** ✓ — wat-macros bootstrap: `syn`/`quote`/`proc-macro2`
  deps, `WatDispatchAttr` parser with 8 unit tests covering path/scope
  parsing, defaults, ordering, and all error cases.
- **Task #191** ✓ — `FromWat`/`ToWat` traits in `src/rust_deps/marshal.rs`.
  Primitives (i64, f64, bool, String, (), Value pass-through), Option
  round-trip, and the generic `Value::RustOpaque` variant with
  `make_rust_opaque` / `rust_opaque_arc` / `downcast_ref_opaque`
  helpers. 12 round-trip + error-case tests.

## Foundation the macro will use

Everything below exists today in `main` as of commit 4bb719f:

- `:rust::` reserved prefix in `resolve.rs`.
- `src/rust_deps/mod.rs` — `RustDepsBuilder`, `RustDepsRegistry`,
  `SchemeCtx` trait, `UseDeclarations`.
- `src/rust_deps/lru.rs` — hand-written shim with `LruCacheCell`
  (thread-id guard), `dispatch_new/put/get`, `scheme_new/put/get`,
  `register()` fn. This is the macro's byte-for-byte codegen target.
- Runtime dispatch for `:rust::*` in `runtime.rs` via registry lookup.
- Checker dispatch for `:rust::*` in `check.rs` via `CheckSchemeCtx`.
- `(:wat::core::use!)` form validated at resolve pass, no-op at
  runtime and check.
- `wat/std/LocalCache.wat` — three thin defines over `:rust::lru::LruCache`.
- End-to-end test: `freeze::tests::invoke_main_uses_std_local_cache_via_rust_lru_shim`.

## In progress

_(nothing — ready for next task)_

## Queue

- **Task #193** 🔜 — Method-level codegen (dispatch/scheme/register).
  Target is `src/rust_deps/lru.rs`'s exact structure.
- **Task #194** — Scope handling: `shared` / `thread_owned` / `owned_move`.
- **Task #195** — Regenerate `src/rust_deps/lru.rs` via macro. Diff
  against hand-written is the correctness proof.

## Stalled / deferred

- **Task #188** — Program Cache. Lands after L1 / macro arc.
- **Task #189** — Backfill FOUNDATION.md. After macro arc lands.

## Critical decisions locked

1. `:Result<T,E>` wat-level type — deferred until rusqlite forces it.
   Added to marshaling traits when the first caller demands it.
2. Variadic SQL params as `:Tuple<...>` — same deferral.
3. Closures crossing wat↔Rust — bidirectional eval, deferred.
4. `use!` scope — program-global now, per-file when multiple files
   with distinct deps exist (wat-rs stdlib or holon-lab-trading).
5. No implicit `self` — methods always take target as first positional.
6. Type keyword in constructors — explicit (`:T` or `:(K,V)`) for
   content-free constructors; inferred from let-annotation for rust-dep
   primitives that have no natural type arg (lru's `::new` takes just `cap`).

## Session stats

Six commits this session. 465 lib + 17 integration + 1 doc test green.
Zero clippy warnings. Pushed.
