# BRIEF — 118.1: the lazy-seq foundation (`Value::Seq` + 6 primitives)

**The work, in one paragraph.** Mint the lazy-seq runtime (arc 118 Option C, clojure-faithful Surface C): a new
`Value::wat__core__Seq(Arc<Seq>)` value, the `Seq`/`LazyCell` types in a new `src/seq/` home, a `realize` that forces
deferred thunks to weak-head-normal-form (memoized, force-once), and the **six irreducible primitives**: `seq-empty`,
`cons`, `lazy-seq`, `first`, `rest`, `empty?`. Everything richer (`map`/`filter`/`take`/`iterate`/`mapv`/`for-each`/…)
is wat over these six in a LATER strike — **do not build the HOF family here.** This strike is the value type + the
six primitives + wiring the variant through the exhaustive `Value` matches (the cascade).

## THE ONE CONTRACT DECISION (pinned — `118/DESIGN.md`, four-questioned)
```rust
// src/seq/  (new home)
pub enum Seq {
    Empty,                                   // terminator (no Option wrapper — the seq IS the data)
    Cons { head: Value, tail: Arc<Seq> },    // strict head; tail is a Seq (possibly a Thunk)
    Thunk(LazyCell),                          // UNREALIZED — realize() forces it to Empty | Cons
}
pub struct LazyCell {
    thunk:  Arc<Function>,                    // a 0-arg wat closure () -> Seq  (Value::wat__core__fn)
    forced: OnceLock<Arc<Seq>>,               // memoize: run the thunk ONCE, cache — the ChildHandle.cached_exit pattern
}
// new variant:  Value::wat__core__Seq(Arc<Seq>)
```
- **`realize(&Seq) -> Arc<Seq>`** (to WHNF): if `Thunk`, `forced.get_or_init(|| apply the thunk closure)` and recurse
  until `Empty | Cons`. The thunk is a captured wat closure carrying its own env — force = `apply_function(closure)`.
  Memoized via `OnceLock` (thread-safe, force-once — pattern off `value.rs:182` `ChildHandle.cached_exit`).
- **`lazy-seq` is a SPECIAL FORM, not a normal fn** — `(lazy-seq <body>)` must NOT eval `<body>` eagerly; it captures
  `<body>` as a 0-arg closure over the current env and returns `Value::Seq(Arc::new(Seq::Thunk(LazyCell{…})))`. (Same
  capture-don't-eval shape as `quote` / the 294.b `:wat::holon::literal` — mirror that dispatch.)
- **`first`/`rest`/`empty?` are CLOJURE-FAITHFUL polymorphic** — EXTEND the existing eager `:wat::core::first`/`rest`
  (they exist) to add a `Value::Seq` arm that `realize`s first; `Cons`→head / tail, `Empty`→`nil`(first)/`Empty`(rest).

## Read in order (the rooms — grounded this session)
1. **`src/value/value.rs:39`** — the `Value` enum (Arc-based; `wat__core__fn(Arc<Function>)` is the closure variant;
   `ChildHandle` at `:182,:184` is the `OnceLock` memoize precedent). Add `wat__core__Seq(Arc<Seq>)`. Then **the
   cascade**: every exhaustive `match` on `Value` (Eq, Hash, Display/type-name, edn/holon, clone helpers, etc.) grows
   a `Seq` arm — let the compiler waterfall you site to site (the fail-count is the progress meter). Eq/Hash on a
   lazy seq: realize-and-compare element-wise is the honest choice, but **STOP-1** if that risks divergence on an
   infinite seq — surface for a decision (Clojure compares realized prefixes / errors; pick the honest bound).
2. **`src/seq/` (NEW)** — `mod.rs`: the `Seq`/`LazyCell` types + `realize` (the `OnceLock` force-once + `apply_function`
   on the thunk). Home the type here; register `src/seq` in `src/lib.rs`.
3. **`src/collection/seq_container.rs`** — read it (a seq abstraction may already exist); reuse/extend if it fits,
   else home in `src/seq/`. Ground before adding.
4. **The intrinsic/runtime dispatch** (`src/intrinsic/` + `src/runtime.rs` head-match) — the six handlers. `cons`/
   `seq-empty` are NEW intrinsics; `lazy-seq` is a NEW special-form arm (capture-don't-eval — mirror `quote` at
   `runtime.rs:4007` + `check.rs:4389`); `first`/`rest`/`empty?` EXTEND the existing handlers (grep
   `":wat::core::first"`/`"rest"`).
5. **`src/check.rs`** — type the primitives over a `Seq<T>` type: `seq-empty :: Seq<T>` · `cons :: (T, Seq<T>) ->
   Seq<T>` · `lazy-seq :: <body:Seq<T>> -> Seq<T>` (special-form, body typed as `Seq<T>`) · `first :: Seq<T> ->
   :Option<T>` (clojure-faithful: empty→nil) · `rest :: Seq<T> -> Seq<T>` · `empty? :: Seq<T> -> bool`. Mirror how
   `quote`/the holon-literal type without recursing where the body is captured.

## STOP triggers (halt + surface; do NOT improvise)
- **STOP-1 (Eq/Hash on lazy seqs):** if making `Value::Seq` `Eq`+`Hash` forces realization that could diverge on an
  infinite seq — STOP and surface; do not silently realize-the-whole-thing. (A bounded/realized-prefix policy or an
  error is the honest call — the orchestrator decides.)
- **STOP-2 (force needs eval context):** if `realize` (running the thunk closure) cannot reach an `Environment`/
  `SymbolTable` from where `rest`/`first` are dispatched — STOP and report the seam. (The thunk is a captured
  closure; `apply_function` should suffice — but surface if not.)
- **STOP-3 (`lazy-seq` can't be a clean special-form arm):** if capturing the body unevaluated needs more than a
  `quote`-style dispatch arm (e.g. a new AST node) — STOP. (294.b proved `quote`-mirroring needs no new variant.)

## EXPECTATIONS (scorecard — fixed before the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | the RED probe flips GREEN | `cargo nextest run --release -p wat -E 'test(lazy_seq_cons_first_rest_traverses)'` (un-ignore) | PASS |
| 2 | the lazy seq runs + prints | `cargo run -q --bin wat -- <a 1-2 lazy-seq program>` | `1` then `2` |
| 3 | the tail is LAZY (not eager) | a `lazy-seq` whose body would error if eval'd, never `rest`-ed | does NOT error (deferred) |
| 4 | nothing else breaks | `cargo nextest run --release` (whole workspace) | floor 0; SET-diff ∅ vs HEAD |

**Runtime prediction:** 40–70 min (a new `Value` variant = a real cascade). **You are a LEAF. Do NOT spawn
subagents.** Build incrementally (`cargo build --release -p wat` after each room; let the exhaustive-match errors
waterfall). Read every diff end-to-end. If a STOP fires or the work exceeds this brief, halt and report.
