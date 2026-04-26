# wat-rs arc 057 — Typed HolonAST leaves — INSCRIPTION

**Status:** shipped 2026-04-25. Three substrate slices land in one
session; the BOOK chapter is the fourth artifact and is being drafted
in conversation with the builder rather than dropped here as prose.

Builder direction (the recognition that opened the arc):

> "Atoms should only be able to hold HolonAST - we should make that a
> firm requirement"

> "in holon algebra - the atom is a holder of a concrete thing - that
> concrete thing can be an AST"

> "are these primitives just a most basic form on an AST? the number
> 42 is an AST?"

The recognition: **a primitive IS an AST.** The number `42` is the
simplest possible HolonAST — a leaf with no sub-terms. The boolean
`true` is an AST. The string `"foo"` is an AST. The keyword `:outcome`
is an AST. They have well-defined canonical encodings; they are terms
in the algebra; they are HolonAST.

The pre-arc-057 schema inverted that: `Atom(Arc<dyn Any>)` was the
only leaf, parametric over arbitrary Rust types. The dyn-Any escape
hatch made `HolonAST` un-`Hash`-able, forced `AtomTypeRegistry` to
exist, blocked structural cache keys, and inverted Lisp's algebra
where `42` IS the atom (not a wrapper around 42).

After this arc, every term in HolonAST is itself HolonAST. `Hash + Eq`
derive cleanly. Cache keys, engram libraries, and the dual-LRU
coordinate cache (lab arc 030 slice 2 onward) all unblock as side
effects.

A second recognition shaped slice 2's semantics, after the builder
asked whether `(Atom (lambda ...))` is still valid:

> "the atom is meant 'to hold' forms - not eval them - someone else
> can eval them"

> "we can just (quote :the-next-form) all the way down"

> "we tell both stories?... the users can choose 'do i want next form?'
> or 'do i want the value?'"

Two surfaces ship, not one:

1. **Story 1 — coordinate.** `:wat::holon::Atom` lowers any captured
   wat form structurally to a HolonAST tree. The form's identity
   participates in the algebra (cosine, Bind, presence, future cache).
   The substrate holds coordinates, not values.
2. **Story 2 — value.** `:wat::holon::to-watast` lifts a HolonAST back
   to a runnable WatAST. Pair with `:wat::eval-ast!` when the consumer
   wants the actual answer, not the path to it.

Neither dominates. The substrate provides both; the consumer chooses
per call site.

Cross-references:
- `holon-rs/src/kernel/holon_ast.rs` — the new 11-variant enum + manual `Hash + Eq` impls.
- `wat-rs/src/runtime.rs::value_to_atom` — slice 2's polymorphic Atom dispatcher (`watast_to_holon` lowers; the Atom output participates as a HolonAST).
- `wat-rs/src/runtime.rs::eval_holon_to_watast` — slice 2's recovery primitive (`holon_to_watast` lifts).
- `wat-rs/src/runtime.rs::hashmap_key` — slice 3's `Value::holon__HolonAST` arm (uses the now-derivable structural Hash).
- `wat-rs/crates/wat-lru/src/shim.rs` — slice 3's "primitives only" panic dropped.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `holon-rs/src/kernel/holon_ast.rs` (full rewrite) — 11-variant enum, manual `Hash`/`PartialEq`/`Eq` (f64 via `to_bits`), per-variant accessors (`as_i64`, `as_string`, `as_symbol`, `as_bool`, `atom_inner`), byte-equivalent canonical-bytes for primitives, empty-Bundle → zero vector. `holon-rs/src/kernel/atom_registry.rs` deleted. `holon-rs/src/kernel/mod.rs`, `lib.rs`, `memory/reckoner.rs` migrated. | ~510 changed / 648 removed | 27 new in holon_ast | shipped |
| 2 | `wat-rs/src/runtime.rs` — `EncodingCtx` drops `registry` field; 13 `encode()` call sites lose the registry arg; `value_to_atom` becomes a polymorphic dispatcher (primitives → typed leaves; HolonAST → opaque-Atom-wrap; Value::wat__WatAST → structural lowering via `watast_to_holon`); `eval_atom_value` dispatches on HolonAST variant directly; `:wat::holon::to-watast` primitive added (`eval_holon_to_watast` + `holon_to_watast` lifter, structural inverse). `wat-rs/src/lower.rs` switches to typed leaf constructors; `wat-rs/src/dim_router.rs` adds 5 leaf-arity arms; `wat-rs/src/check.rs` registers `to-watast` scheme. `wat-rs/src/lib.rs` and `tests/mvp_end_to_end.rs` drop the registry param. `wat-rs/tests/wat_cli.rs` programs use `to-watast` instead of `atom-value` for the eval-recovery path. | ~350 changed | rewrites in lower / runtime / wat_cli; full lab tests still pass | shipped |
| 3 | `wat-rs/src/runtime.rs::hashmap_key` — `Value::holon__HolonAST` arm using `std::hash::DefaultHasher` over the derived structural Hash. `wat-rs/crates/wat-lru/src/shim.rs` — drop the "primitives only" panic; updated diagnostics. `wat-rs/crates/wat-lru/wat-tests/lru/HolonKey.wat` — 3 deftests (round-trip, distinguishes, structural-equal). `wat-rs/Cargo.toml` — `default-members` covers every workspace crate so `cargo test` is workspace-wide by default. `wat-rs/crates/wat-lru/wat-tests/lru/CacheService.wat` — `(:wat::core::first stdout)` matched on `Some/None` (post-arc-047 shape; the test had silently rotted). | ~120 changed | 3 new HolonKey deftests; CacheService re-greens | shipped |
| 4 | This INSCRIPTION + USER-GUIDE rows. BOOK chapter draft (working title in flight; the builder is co-authoring rather than ratifying). | doc-only | — | shipped |

**Substrate test count: 612 → 612 unit / 359 integration.** Workspace-wide totals at HEAD: **983 passing, 0 failing** under the new `cargo test` default.

Build: `cargo build --release --workspace` clean. `cargo test --release` (workspace-wide by default per the new `default-members`): 983 tests, 0 failed.

---

## Architecture notes

### Closing the algebra

The 11-variant `HolonAST`:

```rust
pub enum HolonAST {
    // Vocabulary leaves (BOOK Ch.45)
    Symbol(Arc<str>),
    String(Arc<str>),
    I64(i64),
    F64(f64),
    Bool(bool),

    // Opaque-identity wrap (BOOK Ch.54 — atom-vs-recursive distinction)
    Atom(Arc<HolonAST>),

    // Composites — similarity-preserving recursive encoding
    Bind(Arc<HolonAST>, Arc<HolonAST>),
    Bundle(Arc<Vec<HolonAST>>),
    Permute(Arc<HolonAST>, i32),
    Thermometer { value: f64, min: f64, max: f64 },
    Blend(Arc<HolonAST>, Arc<HolonAST>, f64, f64),
}
```

Every variant is itself HolonAST. Hash + Eq + PartialEq derive (manual
impls because f64 fields use `to_bits` per the standard NaN-Hash
dance). Cache keys, engram identities, persistence — all unblock.

`Atom(Arc<HolonAST>)` survives — narrowed from `Arc<dyn Any>`. Per
BOOK Ch.54 it's a SEMANTICALLY DISTINCT operation from structural
encoding: `Atom(prog)` produces an opaque-identity vector (single
SHA-256 of canonical bytes), `prog` produces the structural vector
(decomposable via `unbind`). Collapsing them would lose a real
algebraic operation. The wrap is also repeatable: `Atom(Atom(x)) ≠
Atom(x) ≠ x`, mirroring Lisp's `'(quote x)` ≠ `'x`.

### Polymorphic `:wat::holon::Atom`

Single wat-surface operator, six dispatches:

| Argument shape | Produces |
|----------------|----------|
| `Value::i64(n)` | `HolonAST::I64(n)` |
| `Value::f64(x)` | `HolonAST::F64(x)` |
| `Value::bool(b)` | `HolonAST::Bool(b)` |
| `Value::String(s)` | `HolonAST::String(s)` |
| `Value::wat__core__keyword(k)` | `HolonAST::Symbol(k)` |
| `Value::holon__HolonAST(h)` | `HolonAST::Atom(h)` — opaque-wrap |
| `Value::wat__WatAST(form)` | structural lowering (Story 1) |

Every existing primitive-argument call site continues to work
unchanged. The new behavior is the WatAST case — a quoted form lowers
recursively (List → Bundle, Keyword → Symbol, Symbol → Symbol,
literals → matching primitive leaves; identifier scope dropped per
"forms are spelling, scope is resolution-time").

### `:wat::holon::to-watast` — Story 2

The structural inverse of the WatAST lowering. Lifts a HolonAST back
to a runnable WatAST so consumers can `(:wat::eval-ast! reveal)`. The
keyword-vs-identifier distinction is recovered via the leading-`:`
convention (Symbol(":foo") → Keyword; Symbol("foo") → Symbol identifier
without scope). Lossy on identifier scope; round-trips cleanly enough
for the eval-and-get-the-value workflow.

### `Atom(Arc<HolonAST>)` opaque-wrap shape

`(:wat::holon::Atom <holon>)` (where the argument is itself a
HolonAST) produces `Atom(Arc<HolonAST>)` — the opaque-identity
wrap. Its canonical bytes are `[TAG_ATOM, "wat/algebra/Holon",
canonical_edn(inner)]`, byte-equivalent with the legacy `Atom(WatAST)`
encoding when the inner is a structurally-lowered form. Future
consumers wanting "this entire subtree as one identity, no
decomposition" reach for this constructor explicitly.

### `AtomTypeRegistry` retired

The registry's job was per-type canonicalization for the dyn-Any
payload. Typed leaves replaced that job; the registry has nothing
to dispatch on. Deleted entirely. `canonical_edn_holon` and `encode`
shed the registry parameter (13 wat-rs call sites updated; the
function signature is one arg shorter).

### Empty Bundle = identity element

`HolonAST::Bundle(Arc::new(vec![]))` materializes as a zero ternary
vector instead of panicking. Required so structurally-lowered wat
forms containing `()` (the empty list — common in match arms,
pattern bindings, `:None` values) survive encoding. Mathematically:
empty bundle = sum of nothing = zero = the identity under the
algebra's superposition.

### `default-members` covers every workspace crate

`Cargo.toml` gains `default-members = [<every crate>]`. Without this,
`cargo test` from the wat-rs root only exercised the root package;
the wat-lru sub-crate's deftest suite (and any other downstream)
required an explicit `--workspace`. That visibility gap let the arc
047 signature change to `:wat::core::first` (Vec<T> → Option<T>)
silently rot CacheService.wat. Pinning here keeps every workspace
crate honest at every checkpoint. CacheService.wat fixed concurrently
to match on Some/None for the post-arc-047 shape.

---

## What this unblocks

- **Lab arc 030 slice 2 — encoding cache.** `LocalCache<HolonAST,
  Vector>` works directly via the now-derived structural Hash; the
  predictor's encode-and-cosine hot path memoizes per-AST.
- **The dual-LRU coordinate cache** (the builder's vision).
  Form → next-form (expansion) and form → value (eval). Both LRUs
  key on the structurally-lowered HolonAST; cache hits are
  algebraic, not by-reference.
- **Reckoner labels on intermediary forms.** Any sub-form on the
  expansion grid can carry a learned label; predictions become
  engrams of labeled traversals through the form-coordinate space.
- **Engram libraries.** Per-AST canonical identity makes
  engram lookup well-defined.
- **Cross-process / persistence.** HolonAST has structural identity
  now; serde and EDN-based wire formats become straightforward (a
  separate arc when a real consumer surfaces).

---

## What this arc deliberately did NOT add

- **`U8` / wider integers / `Bytes` / `Char` leaves.** Ship per
  consumer demand.
- **Auto-derive `ToHolon` for wat-declared user types.** Per
  BOOK Ch.48 (first-class enum representation; substrate doesn't
  mechanically lower) — consumers write explicit one-line lowering
  helpers when they need it.
- **The dual-LRU cache itself.** Slice 3 unblocks it; the actual
  cache is its own arc once a real consumer (proof-perf-001 / lab
  arc 030 slice 2) names the contract.
- **Serde `Serialize` / `Deserialize` for HolonAST.** Hash + Eq
  derive enables the work; serde impl ships separately.
- **Cross-process AST handoff.** Same — enabled, not shipped.
- **Performance optimization of the leaf walks.** Straightforward;
  optimize when measurement shows a hot path.

---

## Visibility-gap correction

The cargo workspace ran tests on a per-package basis by default;
`crates/wat-lru` was outside that surface. arc 047 changed
`:wat::core::first` to return `Option<T>` for Vec args, and the
CacheService.wat test that called `(first stdout)` directly into a
`:String` binding silently broke. Nobody ran `--workspace` between
arcs 047 and 057, so the rot accumulated invisibly.

The fix layer is twofold:
1. The test (`CacheService.wat`) gets the post-arc-047 shape (match
   on Some/None).
2. The visibility (`Cargo.toml::default-members`) makes
   `cargo test` workspace-wide. Future arcs that change a substrate
   surface signature surface the downstream impact at the next
   `cargo test`, not at the next session that happens to type
   `--workspace`.

Builder feedback that drove this:

> "we can make workspace the default?"

> "there are no pre-existing bugs - explain this"

The latter is the load-bearing correction — "pre-existing" was
deflection; the rot was real and the fix is the codebase's
responsibility, not a future session's.

---

## The thread

- **2026-04-25 (morning)** — DESIGN.md + BACKLOG.md drafted.
  Path 3 (strict; lists error) initially proposed.
- **2026-04-25 (mid)** — slices 1 + 2 land for primitives;
  lossless WatAST round-trip marked `#[ignore]` as casualty of
  the strict reading.
- **2026-04-25 (mid, after builder pushback)** — design pivots to
  path 2 (structural lowering); the dual-LRU cache vision named.
  Slice 2 reworks `value_to_atom` to lower WatAST recursively.
- **2026-04-25 (after builder confirmation)** — `:wat::holon::to-watast`
  added as Story-2 recovery; both wat_cli demos un-ignored and re-greened
  under the new shape.
- **2026-04-25 (slice 3)** — `hashmap_key` extended; wat-lru shim
  panic dropped; `default-members` visibility fix; CacheService.wat
  rotted-test repaired.
- **Lab side next:** arc 030 slice 2 (encoding cache) resumes from
  pause; the dual-LRU coordinate cache is its own arc once the
  consumer contract is named.

PERSEVERARE.
