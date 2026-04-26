# wat-rs arc 057 — Typed HolonAST leaves (closing the algebra)

**Status:** opened 2026-04-25.
**Predecessor work:** arc 051 (SimHash), arc 052 (Vector first-class), arc 053 (Phase 4 substrate / Reckoner / OnlineSubspace), arc 056 (`:wat::time::Instant`).
**Downstream consumer:** lab arc 030 slice 2 (encoding cache) is BLOCKED on this; pending wat-rs task `#57 Layer 4 — Cache key (ast-hash, d) + test sweep` rides on top.

Builder direction (2026-04-25, after a long Socratic exchange about the
`Arc<dyn Any>` escape hatch in `HolonAST::Atom`):

> "Atoms should only be able to hold HolonAST - we should make that a
> firm requirement"

> "in holon algebra - the atom is a holder of a concrete thing - that
> concrete thing can be an AST"

> "i feel like a massive refactor is more correct than avoiding it"

> "are these primitives just a most basic form on an AST? the number
> 42 is an AST?"

The recognition: **a primitive IS an AST**. The number 42 is the
simplest possible HolonAST — a leaf with no sub-terms. The boolean
`true` is an AST. The string `"foo"` is an AST. The keyword `:outcome`
is an AST. They have well-defined canonical encodings to vectors;
they are terms in the algebra; they are HolonAST.

Today's `HolonAST::Atom(Arc<dyn Any + Send + Sync>)` inverted that.
`Atom` was the only leaf, parametric over arbitrary Rust types via
`dyn Any`. That escape hatch is what:

- Made `HolonAST` un-`Hash`-able (can't derive Hash on `dyn Any`).
- Forced `AtomTypeRegistry` to exist (per-type canonicalization
  injected at runtime).
- Blocked `:wat::lru::LocalCache<HolonAST, V>` from working
  (panics on non-primitive keys).
- Blocked engram libraries / persistence / cross-process AST
  handoff from getting clean canonical bytes.
- Forced wat-lru's `hashmap_key` to reject non-primitive Values.
- Inverted Lisp's algebra (where `atom?` is a predicate and `42`
  IS the atom; not a wrapper around 42).

This arc closes the algebra: every term in HolonAST is itself
HolonAST. Primitives become first-class leaf variants. The dyn Any
storage layer dies. Hash + Eq + PartialEq derive directly. Cache
keys, engram libraries, and downstream serialization all unblock as
side effects.

Cross-references:
- `holon-rs/src/kernel/holon_ast.rs:55` — current `HolonAST` enum (six forms; `Atom(Arc<dyn Any>)` is the one we replace).
- `holon-rs/src/kernel/atom_registry.rs:124` — `canonical_bytes` (the registry indirection that becomes vestigial for primitives, reduces to user-type-as-holon for the rest).
- `wat-rs/src/runtime.rs:4535` — `hashmap_key` (extends to accept HolonAST after derive).
- `wat-rs/crates/wat-lru/src/shim.rs:72-80` — the panic that goes away.
- BOOK chapter forthcoming — "The Sealed Holon" (working title; the principle: the algebra is closed; primitives are ASTs).

---

## Why this arc, why now

The lab is stacking up consumers that all need canonical AST identity:

- arc 030 slice 2 (encoding cache `HolonAST → Vector` memoization)
- proof-perf-001 (consumes the cache)
- engram libraries (Phase 4 work; needs cache-friendly identity per AST)
- cross-process LogEntry serialization (when proofs ship across machines)
- the Reckoner's labeled training data (`(surface_ast, label_ast)` pairs need stable identity)

Every one of those today either reaches for SimHash (locality-preserving, wrong for memoization) or works around `Arc<dyn Any>` with synthetic per-type machinery.

The longer the lab grows on the dyn Any escape hatch, the more migration the eventual cleanup costs. Today the lab has ~98 wat call sites using `:wat::holon::Atom`, ~70 `AtomTypeRegistry::register::<T>` calls. **Almost none of those need to change** if the wat surface stays — `:wat::holon::Atom` becomes a polymorphic constructor that dispatches on its argument's runtime type to the right typed leaf. The lab keeps reading the same way; the substrate gets the algebra it should have had.

The convergence: cache work, engram work, persistence work, and Reckoner-as-substrate-bridge work all sit downstream of HolonAST being a real algebra type with derive-Hash. Ship this once, unblock all of them.

---

## What ships

A schema change to `HolonAST` + a polymorphic wat-surface constructor + AtomTypeRegistry shrinkage + Hash derive + tests.

### The new HolonAST

```rust
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum HolonAST {
    // Leaves — terminal terms, no sub-AST. These are atoms in the
    // Lisp sense: `(atom? x)` returns true for these.
    Symbol(Arc<str>),       // quoted keywords; today's (Atom (quote :foo))
    String(Arc<str>),       // string literals; today's (Atom "foo")
    I64(i64),               // integer atoms
    F64(f64),               // float atoms (Hash via to_bits)
    Bool(bool),             // boolean atoms

    // Composites — operate on other HolonAST terms (unchanged).
    Bind(Arc<HolonAST>, Arc<HolonAST>),
    Bundle(Arc<Vec<HolonAST>>),
    Permute(Arc<HolonAST>, i32),
    Thermometer { value: f64, min: f64, max: f64 },
    Blend(Arc<HolonAST>, Arc<HolonAST>, f64, f64),
}
```

`HolonAST::Atom(Arc<dyn Any>)` is **removed** from the enum. The
`Atom` concept survives as:
- A wat-surface constructor (`:wat::holon::Atom`) that dispatches to typed leaves.
- An optional predicate (`:wat::holon::atom?`) — true for the leaf variants. Lisp parallel.

`Hash` derives via `#[derive]`. The `f64` fields (Thermometer's value/min/max, Blend's w1/w2, the F64 leaf) need a manual `Hash` impl using `to_bits()` (Rust's `f64` doesn't impl Hash because of NaN). Same trick the archive used (`thought_encoder.rs:45-46`).

### Polymorphic `:wat::holon::Atom` evaluator

```text
(:wat::holon::Atom <expr>)
```

dispatches on `<expr>`'s runtime `Value` variant:

| Argument variant | Produces |
|------------------|----------|
| `Value::String(s)` | `HolonAST::String(Arc::from(s))` |
| `Value::wat__core__keyword(k)` | `HolonAST::Symbol(Arc::from(k))` |
| `Value::i64(n)` | `HolonAST::I64(n)` |
| `Value::f64(x)` | `HolonAST::F64(x)` |
| `Value::bool(b)` | `HolonAST::Bool(b)` |
| `Value::holon__HolonAST(h)` | `(*h).clone()` — passthrough (Lisp's quote-of-quoted is no-op) |
| anything else (struct, enum, vector, etc.) | error: "Atom argument must be a primitive or HolonAST; user types must provide ToHolon" |

Every existing `(:wat::holon::Atom "foo")` and `(:wat::holon::Atom (:wat::core::quote :outcome))` call site continues to work unchanged. The wat surface is preserved.

### AtomTypeRegistry shrinks

Today the registry registers ~12 built-in primitive canonicalizers (i8/i16/.../String/&str/bool/char/HolonAST). Those become **vestigial** — primitives no longer need registry-injected canonicalization because they're typed leaves. The registry survives only for user-type-to-holon migration:

- A user wants `(:wat::holon::Atom my-phase-label)` to work where `my-phase-label` is a `:trading::types::PhaseLabel`. They provide a wat-side or Rust-side `ToHolon` impl. `:wat::holon::Atom`'s evaluator either calls that or rejects with "user types need ToHolon."

Two paths for user types:
1. **Manual `ToHolon`** — Rust trait `pub trait ToHolon { fn to_holon(&self) -> HolonAST; }`. Lab implements per-type once.
2. **Auto-derive** for wat-declared `:wat::core::enum` / `:wat::core::struct` types — substrate maps enums to `Bundle(Symbol(:VariantName), <fields>)` and structs to `Bundle(Symbol(:TypeName), <fields>)`. Ships in this arc; lab user types work without manual impl.

### `wat-lru/src/shim.rs::hashmap_key` accepts HolonAST

After the Hash derive lands, `hashmap_key` extends to:

```rust
Value::holon__HolonAST(h) => Ok(format!("H:{:x}", structural_hash(&h))),
```

where `structural_hash` is `std::hash::Hash` materialized to u64. `LocalCache<HolonAST, V>` works directly. No registry indirection needed.

### Downstream cleanups (this arc)

- `:wat::holon::simhash` — the encoder still walks the AST; primitive leaves get a canonical content vector (via the existing per-type encoding). Behavior unchanged externally.
- `wat-rs/src/runtime.rs` `eval_*_atom*` paths — small rewrites to consume typed leaves.
- USER-GUIDE — atom section reframes ("primitives are atoms; `Atom` is the constructor smart-dispatch").

---

## Decisions resolved

### Q1 — Does `Atom` survive as a HolonAST variant?

**No.** It's removed from the Rust enum.

The wat-surface `:wat::holon::Atom` constructor survives (preserves
the 98 lab call sites and the Lisp-style quote ergonomics). The
Rust enum has typed leaves directly. This matches Lisp: `42` is the
atom; `(atom 42)` is just `42`. Wrapper redundancy goes away.

If a future use case for an "opaque-tag-wrapper" surfaces (e.g.,
"treat this subtree as an indivisible identity for binding"), it
ships as its own variant then. Speculative wrapping isn't worth a
variant today.

### Q2 — Which primitive leaves?

**Five: `Symbol(Arc<str>)`, `String(Arc<str>)`, `I64(i64)`, `F64(f64)`, `Bool(bool)`.**

Justification by current use:
- `Symbol` — every `(:wat::holon::Atom (:wat::core::quote :foo))`.
- `String` — every `(:wat::holon::Atom "literal")`.
- `I64` — sometimes used for atom-of-int (rare today; valuable for future).
- `F64` — same for floats. Note the distinction from `Thermometer`: F64 is content identity, Thermometer is gradient encoding. Both stay.
- `Bool` — for atom-of-bool. Rare today; clean for future.

**Out for v1:** `U8`/wider integers, `Bytes`/`Vec<u8>`, `Char`. Not used today; ship per consumer demand. Same "supported on demand" rhythm as past arcs.

`Symbol` vs `String` distinction kept (they mean different things — keyword identity vs raw data; the type checker treats them differently today). Unifying into a single `Text` would be ergonomically lossy.

### Q3 — How do user types (lab enums, structs) become atoms?

**Auto-derive for wat-declared types** ships in this arc. Wat's
`:wat::core::enum` and `:wat::core::struct` declarations already
generate accessors and constructors; this arc adds an auto-generated
`ToHolon` per declared type:

- An enum variant `:trading::sim::Direction::Up` → `HolonAST::Symbol(":Up")` for unit; `HolonAST::Bundle([Symbol(":Up"), <field-asts>])` for tagged.
- A struct value → `HolonAST::Bundle([Symbol(":TypeName"), <field-asts>])`.

`:wat::holon::Atom` accepts struct/enum values via this path. The
70 existing `register::<T>` registrations for user types collapse to
the auto-derive; no per-type Rust code needed.

For non-wat-declared user types (raw Rust types brought in via
`#[wat_dispatch]`), a manual `impl ToHolon for T` is required. Same
pattern as `Display`/`Debug` derives — ergonomic for declared types,
explicit for hand-rolled ones.

### Q4 — What happens to existing `AtomTypeRegistry` registrations?

The 12 built-in primitive registrations (`i8`, `i16`, ..., `String`,
`&str`, `bool`, `char`, `HolonAST`) become **vestigial** — primitives
are typed leaves now; the registry's per-type canonicalization is no
longer the path.

The registry itself survives in shrunk form for user types that
register their own `ToHolon` (programmatic registration; the
auto-derive is the common path).

The 70 `register::<T>` calls in lab + wat-rs: most are the built-in
primitives (substrate's own `with_builtins`); those get removed from
the substrate. Lab user-type registrations (~5–10 calls based on
sampling) get replaced by the auto-derive for wat-declared types or
manual `ToHolon` impls.

### Q5 — Encoding to vectors — does this change?

**No external change.** Today's `:wat::holon::encode holon → Vector`
walks the AST and produces a vector. The walk now matches on typed
leaves (Symbol → content-addressed identity vector via SHA-256 of
the symbol's string; I64 → identity vector via SHA-256 of the bytes;
etc.). Same vectors as today (assuming the canonical-bytes derivation
matches).

A subtle correctness check: today's `Atom(quote :foo)` produces some
vector V. Tomorrow's `Symbol(":foo")` produces some vector V'. They
must be equal (or downstream cosines shift, breaking proof 002's
shipped numbers).

The arc's verification gate: **proof 002 produces the same
per-thinker numbers** post-migration. If anything moves, the
encoding change leaked; back out and re-investigate.

### Q6 — `Hash` for f64 fields

`f64` doesn't impl `Hash` because of NaN. Manual impl uses `to_bits()`:

```rust
impl Hash for HolonAST {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            HolonAST::F64(x) => x.to_bits().hash(state),
            HolonAST::Thermometer { value, min, max } => {
                value.to_bits().hash(state);
                min.to_bits().hash(state);
                max.to_bits().hash(state);
            }
            HolonAST::Blend(a, b, w1, w2) => {
                a.hash(state); b.hash(state);
                w1.to_bits().hash(state); w2.to_bits().hash(state);
            }
            // Other variants derive cleanly.
            ...
        }
    }
}
```

Same trick the archive used (`thought_encoder.rs:45-46`). NaN-vs-NaN
comparison is meaningless either way; identical NaN bit patterns
hash the same; differing bit patterns don't collide.

### Q7 — Why polymorphic `:wat::holon::Atom` instead of typed constructors?

We could ship `:wat::holon::Symbol`, `:wat::holon::String`,
`:wat::holon::I64`, etc. as separate constructors. But that breaks
the 98 lab call sites — they all use `:wat::holon::Atom <expr>`.

The polymorphic constructor preserves the surface; new typed
constructors can ship later as ergonomic shortcuts (`(:wat::holon::Symbol :foo)` is one character shorter than `(:wat::holon::Atom (:wat::core::quote :foo))`). Both forms coexist; lab callers migrate at their pace if they want.

### Q8 — `simhash` and the encoder

`:wat::holon::simhash holon → :i64` is unchanged externally. Internally, the encoder walks typed leaves directly:

- `Symbol(s)` → SHA-256 of `("Symbol:" + s)` → vector of d signs
- `String(s)` → SHA-256 of `("String:" + s)` → vector
- `I64(n)` → SHA-256 of `("I64:" + n.to_le_bytes())` → vector
- `F64(x)` → SHA-256 of `("F64:" + x.to_bits().to_le_bytes())` → vector
- `Bool(b)` → SHA-256 of `("Bool:" + b)` → vector

The "same input → same vector" guarantee is preserved; the encoding mechanics shift from registry-mediated to direct.

The archive used `to_bits()` for f64 hashing for exactly this reason
— quantized values produce different hashes; cache hits stay
correct.

### Q9 — Migration verification

The arc ships a verification harness:

1. `cargo test --release` in wat-rs — every existing wat test passes
   (60+ tests; covers all the primitive-atom and user-enum paths).
2. `cargo test --release wat_suite` in holon-lab-trading — all 336
   lab tests pass with no per-test changes.
3. `cargo test --release --features proof-002 --test proof_002` —
   proof 002 produces the same per-thinker numbers
   (`always-up | 34 | 0 | 34`, `sma-cross | 34 | 5 | 29`).
4. The new `LocalCache<HolonAST, Vector>` works (smoke test in
   wat-tests/ at the substrate level).

If (3) shifts, the encoding-mechanics change isn't byte-equivalent;
back out and reconcile.

---

## Implementation sketch

Five slices, tracked in [`BACKLOG.md`](BACKLOG.md):

- **Slice 1** — `HolonAST` schema change in `holon-rs`. Add typed leaf variants; remove `Atom(Arc<dyn Any>)`. Manual Hash impl for f64-bearing variants. Update encoder walks. Tests in `holon-rs`.
- **Slice 2** — `:wat::holon::Atom` polymorphic constructor in `wat-rs/src/runtime.rs`. Tests covering each primitive dispatch case.
- **Slice 3** — Auto-derive `ToHolon` for wat-declared enums + structs in `wat-rs/src/check.rs` + `wat-rs/src/runtime.rs`. Tests that lab-shape enums round-trip.
- **Slice 4** — `wat-lru/src/shim.rs::hashmap_key` extends to `Value::holon__HolonAST` via the derived Hash. Drop "primitives only" panic message. Smoke test: `LocalCache<HolonAST, Vector>` works end-to-end.
- **Slice 5** — INSCRIPTION + USER-GUIDE rows + BOOK chapter draft ("The Sealed Holon" or as named).

Total estimate: ~2 days of focused work. Heavier than arc 056 (`time::Instant`, half a day) because of the schema change + encoder migration. Lighter than arc 053 (Phase 4 substrate, ~5 days) because no new runtime types beyond enum variants.

---

## What this arc does NOT add

- **`U8` / wider integers / `Bytes` / `Char` leaves.** Not used today; ship per consumer demand.
- **Registry-aware `AtomTypeRegistry` consolidation.** The shrunken registry stays in place for now; a follow-up arc may rip it entirely once auto-derive covers every consumer.
- **Persistence / serialization of HolonAST.** The Hash + Eq derive enables it but the actual serde impl ships separately when a consumer surfaces.
- **Cross-process AST handoff.** Same — enabled, not shipped.
- **`(ast-hash, d)` cache key for the dim-router** (lab task #57). That's a different Layer 4 concern; this arc's HolonAST::Hash unblocks it but the dim-router work is its own arc.
- **A `Char` or `Bytes` leaf.** No current consumer; defer.
- **Performance optimization.** The primitive-leaf walk is straightforward; if measurement shows a hot path, optimize then.

---

## Non-goals

- **Backwards compatibility with raw-Rust-type atoms.** `(:wat::holon::Atom <some-Rust-struct-not-implementing-ToHolon>)` errors. That's intentional — the algebra is closed; user types must opt in via ToHolon.
- **Lossless preservation of every current encoding behavior.** If quantization-related encoding changes shift any vector by 1 bit, that's acceptable provided the verification harness (proof 002 numbers) doesn't shift.
- **A general-purpose `Any → HolonAST` reflection mechanism.** Out of scope; explicit `ToHolon` is the contract.

---

## What this unblocks

- **lab arc 030 slice 2** (encoding cache) — `LocalCache<HolonAST, Vector>` works directly. The half-day cache work resumes immediately.
- **lab task #57** (`(ast-hash, d)` cache key for dim-router) — `HolonAST::Hash` is now derivable; the substrate's internal cache layer gets the right key shape.
- **Engram libraries** (Phase 4 work) — per-AST canonical identity makes engram lookup well-defined.
- **LogEntry serialization across processes** — HolonAST serde impls become straightforward after the schema change.
- **Reckoner training data with AST labels** — `(surface_ast, label_ast)` pairs get stable identity for memoization.
- **The whole BOOK Chapter 51 / 54 framing** ("programs as coordinates") — when the program IS a HolonAST and HolonAST has typed leaves, the coordinate machinery operates on a clean algebra.

PERSEVERARE.
