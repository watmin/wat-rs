# Macro arc — live progress log

Updated as work happens. If compaction hits mid-slice, read this to
resume. Full design: `docs/wat-dispatch-macro-design-2026-04-19.md`.
Namespace principle: `docs/namespace-principle-2026-04-19.md`.
Caching architecture: `docs/caching-design-2026-04-19.md`.

## Completed — everything under the macro-arc + sweep umbrella

All tasks below shipped green with zero clippy warnings. Full
workspace test suite passing at each checkpoint.

### Typed constructors (prep)
- **#184** — typed `:wat::core::vec` / `::list` constructors.
- **#185** — typed `:wat::std::HashMap` / `::HashSet` constructors.
- **#186** — reverted in-runtime Cache scaffold.

### `:rust::` namespace + interop machinery
- **#187** — LocalCache as pure wat source over `:rust::lru::LruCache`.
  Runtime dispatch, check-pass scheme dispatch, `(:wat::core::use!)`
  form, `wat/std/LocalCache.wat`.
- **#190** — Cargo workspace conversion; `wat-macros/` sibling crate.
- **#192** — `wat-macros` bootstrap: `#[wat_dispatch]` attribute
  parser with 8 unit tests.
- **#191** — `FromWat`/`ToWat` + `Value::RustOpaque` generic opaque
  handle variant.

### The macro
- **#193** — Method-level codegen: dispatch + scheme + register.
  Covers associated fns, `&self`, `&mut self`, `self` (owned_move);
  primitives, Option, Vec, Tuple, Result, Self-opaque returns;
  `type_params` attribute for phantom generics; all three scope modes.
- **#195** — Regenerated `src/rust_deps/lru.rs` via macro. Hand-written
  `LruCacheCell` removed; `Value::RustOpaque` replaces the dedicated
  `Value::rust__lru__LruCache` variant.

### Marshaling expansions (rusqlite prep)
- **#196 (E1)** — `Vec<T>` marshaling.
- **#197 (E2)** — `Tuple<A,B,...>` marshaling, arities 1–6.
- **#198 (E3)** — `:Result<T,E>` wat type + `(Ok v)` / `(Err e)`
  constructors + match integration + marshaling.
- **#199 (E4)** — `scope = "shared"` — plain Arc payload, &self
  methods, cross-thread permitted.
- **#200 (E5)** — `scope = "owned_move"` — `OwnedMoveCell<T>` with
  AtomicBool gate for consumed-after-use handles.

### Honesty sweep
- **#201 (H)** — All Rust-sourced types surfaced under `:rust::*`
  with fully-qualified paths (`:rust::std::io::Stdin`, not
  `:rust::io::Stdin`). `:wat::` and `:rust::` coexist as siblings.

## Queue

- **#188** 🔜 — Program Cache (L2) as pure wat source. Uses
  `:wat::std::LocalCache` + queues + `spawn` + `select`. Completes
  058 Step 5.
- **#189** — Backfill 058 FOUNDATION.md with the session's arc.
  Mechanical but important.
- **Trading-lab migration** — rusqlite shim + wat driver. The
  foundation is now ready.

## Critical decisions locked

1. `:Result<T,E>` — shipped (E3).
2. Variadic SQL params as tuples — shipped (E2).
3. Closures crossing wat↔Rust — deferred; bidirectional eval is a
   future slice, rusqlite's one-shot dispatch pattern works without.
4. `(:wat::core::use!)` scope — program-global; per-file enforcement
   is a planned upgrade.
5. No implicit `self` — methods always take target as first positional.
6. Type keyword in constructors — explicit (`:T` or `:(K,V)`) for
   content-free constructors; inferred from let-annotation for
   rust-dep primitives with no natural type arg.
7. Namespace: `:wat::` for language-native + wat-stdlib; `:rust::`
   for imported Rust crates, fully qualified. Aliases are a user
   concern.

## Where the foundation stands

wat-rs is Clojure-on-JVM shaped — a hosted language over Rust. Any
consumer crate (e.g. holon-lab-trading):

```rust
// Their Cargo.toml inherits wat-rs's deps automatically.

// Their rusqlite_shim.rs:
use wat_macros::wat_dispatch;

#[wat_dispatch(path = ":rust::rusqlite::Connection", scope = "thread_owned")]
impl WatSqliteConnection {
    pub fn open(path: String) -> std::result::Result<Self, String> { ... }
    pub fn execute(&mut self, sql: String, params: (i64, String))
        -> std::result::Result<i64, String> { ... }
    pub fn query_scalar(&mut self, sql: String, params: (i64,))
        -> std::result::Result<Option<i64>, String> { ... }
    pub fn query_all(&mut self, sql: String, params: (i64,))
        -> std::result::Result<Vec<(i64, String)>, String> { ... }
}
```

~30 lines per important method. Their wat-level driver stays pure wat.
