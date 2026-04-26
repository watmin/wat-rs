# wat-rs arc 057 — Typed HolonAST leaves (closing the algebra)

**Status:** shipped 2026-04-25. See `INSCRIPTION.md` for the
canonical post-ship record. This DESIGN stays as the
pre-implementation reasoning artifact; one significant pivot is
flagged inline (Q3 — SHIPPED REVISION).
**Predecessor work:** arc 051 (SimHash), arc 052 (Vector first-class), arc 053 (Phase 4 substrate / Reckoner / OnlineSubspace), arc 056 (`:wat::time::Instant`).
**Downstream consumer:** lab arc 030 slice 2 (encoding cache) is now substrate-unblocked; pending wat-rs task `#57 Layer 4 — Cache key (ast-hash, d) + test sweep` rides on top.

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HolonAST {
    // Leaves — terminal vocabulary atoms (BOOK Chapter 45 framing —
    // irreducible semantic units the algebra projects onto the
    // hypersphere). These are also atoms in the Lisp `atom?`
    // predicate sense.
    Symbol(Arc<str>),       // keywords; today's (Atom (quote :foo))
    String(Arc<str>),       // string literals; today's (Atom "foo")
    I64(i64),               // integer atoms
    F64(f64),               // float atoms (manual Hash via to_bits)
    Bool(bool),             // boolean atoms

    // Atom — opaque-identity wrapper around any HolonAST. Per BOOK
    // Chapter 54: the substrate has TWO distinct strategies for
    // programs-as-data — opaque identity (Atom-wrap, single SHA-256
    // seed of canonical bytes) and similarity-preserving (recursive
    // encoding through composites). Atom NARROWED to Arc<HolonAST>
    // contents only — the dyn Any escape hatch dies; the
    // semantically-distinct opaque-identity strategy survives.
    Atom(Arc<HolonAST>),

    // Composites — similarity-preserving recursive encoding.
    Bind(Arc<HolonAST>, Arc<HolonAST>),
    Bundle(Arc<Vec<HolonAST>>),
    Permute(Arc<HolonAST>, i32),
    Thermometer { value: f64, min: f64, max: f64 },
    Blend(Arc<HolonAST>, Arc<HolonAST>, f64, f64),
}
```

11 variants. `HolonAST::Atom(Arc<dyn Any>)` is replaced by
`HolonAST::Atom(Arc<HolonAST>)` — opaque-identity wrapping for
programs only; primitives are first-class typed leaves.

`Hash + Eq` impls (manual, not derive — f64 fields use `.to_bits()`
because Rust's `f64` doesn't impl Hash; mem::discriminant for
variant tagging). Same trick the archive used (`thought_encoder.rs:45-46`).

### Polymorphic `:wat::holon::Atom` evaluator

`(:wat::holon::Atom <expr>)` evaluates `<expr>` (eval semantics —
preserves variable / function-result Atom args, e.g. the lab's
`(:wat::holon::Atom name)` where `name` is a `:String` parameter).
Dispatches on the resulting `Value`:

| Evaluated argument | Produces |
|--------------------|----------|
| `Value::String(s)` | `HolonAST::String(Arc::from(s))` |
| `Value::wat__core__keyword(k)` | `HolonAST::Symbol(Arc::from(k))` |
| `Value::i64(n)` | `HolonAST::I64(n)` |
| `Value::f64(x)` | `HolonAST::F64(x)` |
| `Value::bool(b)` | `HolonAST::Bool(b)` |
| `Value::holon__HolonAST(h)` | `HolonAST::Atom(h)` — opaque-identity wrap (preserves the BOOK Ch.54 atom-vs-recursive-encoding distinction) |
| `Value::Enum {...}` | **ERROR**: "wat enum values aren't directly representable in the algebra; lower explicitly via a wat-side function returning the holon shape you want, then wrap in Atom for opaque identity if needed" |
| `Value::Struct {...}` | Same error |
| Lambda / Vector / ProgramHandle / etc. | Same error ("not lowerable to holon") |

Every existing primitive-argument call site (`(:wat::holon::Atom "foo")`, `(:wat::holon::Atom (:wat::core::quote :outcome))`, `(:wat::holon::Atom name)` where `name` is a `:String` variable) continues to work unchanged — they all evaluate to primitives that map to typed leaves.

The `Value::Enum`/`Value::Struct` error case never fires in the lab today because the lab's surface ASTs use string literals, quoted keywords, and variable-bound primitives — no enum-as-atom pattern. New consumers who want enum-as-atom write a tiny wat-side lowering helper:

```scheme
;; The consumer's lowering — choose whatever holon shape suits
;; their cosines. Substrate doesn't opine.
(:wat::core::define
  (:trading::sim::Direction/to-symbol
    (d :trading::sim::Direction)
    -> :String)
  (:wat::core::match d -> :String
    (:trading::sim::Direction::Up   ":Direction::Up")
    (:trading::sim::Direction::Down ":Direction::Down")))

;; Then:
(:wat::holon::Atom (:trading::sim::Direction/to-symbol some-direction))
;; → evaluates to a String → maps to HolonAST::Symbol leaf
```

This matches BOOK Ch.48's first-class-enum stance (substrate doesn't mechanically lower; consumer chooses) and the user's principle ("the consumer always knows better than the substrate").

### AtomTypeRegistry shrinks (probably retires)

Today the registry registers ~12 built-in primitive canonicalizers (i8/i16/.../String/&str/bool/char/HolonAST). Those become **vestigial** — primitives are typed leaves now; HolonAST doesn't need registry-injected canonicalization because it has structural Hash + Eq.

The registry was the dyn Any escape hatch's enabler. With dyn Any gone, the registry has nothing to canonicalize. It probably retires entirely; if any caller surfaces a real need for runtime-extensible canonicalization (unlikely — consumers do explicit lowering at the call site instead), it ships as its own arc.

**No `ToHolon` trait, no auto-derive in this arc.** Per BOOK Ch.48 (first-class enums, substrate doesn't mechanically lower) and the user's principle ("the consumer always knows better"), wat enum / struct values aren't directly Atom-able. Consumers write explicit lowering helpers in their own wat code. The substrate stays out of opinionation.

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

**Yes — narrowed to `Arc<HolonAST>` contents only.** The dyn Any
escape hatch dies; the opaque-identity wrapping semantic survives
because BOOK Chapter 54 explicitly distinguishes two strategies for
programs-as-data:

- `Atom(Arc<HolonAST>)` → opaque identity (single SHA-256 of canonical bytes; no similarity preservation)
- Recursive encoding through composites → similarity-preserving

These are SEMANTICALLY DISTINCT. A consumer choosing "treat this
program as one atomic identity for cosine purposes" needs the
Atom-wrap; a consumer choosing "let the substrate find similarity
via shared structure" doesn't wrap. Collapsing them would lose a
real algebraic operation.

For primitives (Symbol, String, I64, F64, Bool), no wrapper is
needed — primitives ARE their own atoms in the Lisp `atom?`
predicate sense AND in BOOK Ch.45's "vocabulary atoms" sense. The
wat-surface `(:wat::holon::Atom 42)` produces `I64(42)` directly,
not `Atom(I64(42))`.

The wrapper wrapping itself is also useful: `(:wat::holon::Atom (:wat::holon::Atom <holon>))` produces `Atom(Arc::new(Atom(...)))`, a different vector than the inner alone — Lisp's `'(quote x)` ≠ `'x`.

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

**SHIPPED REVISION (2026-04-25, mid-arc pivot).** During slice 2
implementation the builder challenged the strict-error stance for
quoted forms specifically:

> "(Atom (lambda (x) (* x x))) — this is a valid expr - right?"

> "the atom is meant 'to hold' forms - not eval them - someone else
> can eval them"

> "we can just (quote :the-next-form) all the way down"

> "we tell both stories?... the users can choose 'do i want next
> form?' or 'do i want the value?'"

The shipped semantics for `Value::wat__WatAST` arguments to
`:wat::holon::Atom` is **structural lowering**, not error:

- `WatAST::IntLit(n)` → `HolonAST::I64(n)`
- `WatAST::FloatLit(x)` → `HolonAST::F64(x)`
- `WatAST::BoolLit(b)` → `HolonAST::Bool(b)`
- `WatAST::StringLit(s)` → `HolonAST::String(s)`
- `WatAST::Keyword(k)` → `HolonAST::Symbol(k)`
- `WatAST::Symbol(ident)` → `HolonAST::Symbol(ident.name)` (scope dropped)
- `WatAST::List(items)` → `HolonAST::Bundle([lower each])`

The form's identity participates in the algebra (cosine, Bind,
presence, structural cache keys). A quoted lambda becomes a Bundle
whose shape encodes the program. Substrate holds coordinates, not
values.

The Story-2 recovery primitive `:wat::holon::to-watast` (added
slice 2) is the structural inverse: lifts a HolonAST back to a
runnable WatAST so consumers can `(:wat::eval-ast! reveal)` when
they want the value, not the path. Identifier scope is the only
lossy piece (recovered as bare-name on lift via the leading-`:`
keyword convention).

For `Value::Enum` / `Value::Struct` (true wat-declared user
values, not quoted forms), the pre-pivot strict-error stance
**still holds** as shipped — the substrate doesn't mechanically
lower these; the consumer writes an explicit lowering helper as
the original Q3 text describes (the example below remains valid).
BOOK Ch.48 (first-class enums; no mechanical lowering) and the
user's "consumer always knows better" principle apply unchanged.

The lab today doesn't use enum-as-atom anywhere — surface ASTs are
all string literals, quoted keywords, and variable-bound primitives
(per a `grep -rn` survey of `:wat::holon::Atom` argument shapes).

When a future consumer surfaces a real need ("I want my PhaseLabel
to be a holon"), they write a one-line lowering helper:

```scheme
(:wat::core::define
  (:trading::types::PhaseLabel/to-symbol
    (l :trading::types::PhaseLabel) -> :String)
  (:wat::core::match l -> :String
    (:trading::types::PhaseLabel::Peak    ":Peak")
    (:trading::types::PhaseLabel::Valley  ":Valley")
    (:trading::types::PhaseLabel::Transition ":Transition")))

;; Use site:
(:wat::holon::Atom (:trading::types::PhaseLabel/to-symbol some-phase))
;; → evaluates to a String → maps to HolonAST::Symbol leaf
```

The consumer chose the lowering. Substrate stays minimal.

See INSCRIPTION's "two stories" section + BOOK Chapter 59 for the
shipped framing.

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

Four slices, tracked in [`BACKLOG.md`](BACKLOG.md):

- **Slice 1** — `HolonAST` schema in `holon-rs`. Add 5 typed primitive leaf variants. Narrow `Atom` from `Arc<dyn Any>` to `Arc<HolonAST>`. Manual `Hash + Eq` impls (f64 fields use `to_bits`). Update encoder walks (byte-equivalent canonical bytes for primitives — preserves proof 002 numbers). Shrink/retire `AtomTypeRegistry::with_builtins`. Tests.
- **Slice 2** — `:wat::holon::Atom` polymorphic eval-and-dispatch in `wat-rs/src/runtime.rs`. Primitives → typed leaves; HolonAST → opaque-Atom-wrap; Enum/Struct → ERROR with helpful message. Tests covering each dispatch case + a test asserting Enum/Struct error.
- **Slice 3** — `wat-lru/src/shim.rs::hashmap_key` extends to `Value::holon__HolonAST` via the derived Hash. Drop "primitives only" panic message. Smoke test: `LocalCache<HolonAST, Vector>` works end-to-end.
- **Slice 4** — INSCRIPTION + USER-GUIDE rows + BOOK chapter draft (working title: "The Sealed Holon" — captures the "atom is opaque-identity wrap; primitives are vocabulary atoms; algebra is closed" trio).

Total estimate: ~1.5 days of focused work (slice 3 of the original five — the auto-derive ToHolon — got cut per BOOK Ch.48 alignment + Q3 above; saved ~3 hours and removed real risk).

The proof 002 verification gate ships at slice 4: same per-thinker numbers (`always-up | 34 | 0 | 34`, `sma-cross | 34 | 5 | 29`). If they shift, encoding mechanics changed; back out.

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
