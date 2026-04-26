# wat-rs arc 057 — Typed HolonAST leaves — BACKLOG

**Status: shipped 2026-04-25.** All four slices in one session.
`INSCRIPTION.md` is the canonical post-ship record; this BACKLOG
stays as the historical planning artifact.

**One mid-arc pivot worth flagging up front:** slice 2 originally
called for path-3 strict ("WatAST → ERROR; consumer writes a
lowering helper"). After builder pushback ("the atom is meant to
hold forms — not eval them — someone else can eval them" / "we can
just (quote :the-next-form) all the way down"), slice 2 pivoted to
path 2 (structural lowering: List → Bundle, Keyword → Symbol,
literals → matching primitive leaves) AND a new Story-2 recovery
primitive `:wat::holon::to-watast` was added so the eval-recovery
round-trip works through both surfaces. See INSCRIPTION's "two
stories" framing and DESIGN's "Q3 — SHIPPED REVISION" note.

A second mid-arc carry-along: slice 3 surfaced a workspace-default
visibility gap (the wat-lru sub-crate's deftest suite wasn't run
by `cargo test` from the wat-rs root, which had let an arc 047
signature change silently rot a test). `Cargo.toml::default-members`
fix shipped concurrently; `cargo test` is now workspace-wide by
default. INSCRIPTION's "visibility-gap correction" section captures
the rationale.

A third post-ship deliverable: `:wat::test::assert-coincident` (a
purpose-built holon-equality test helper). Three lab tests went red
under the new encoding because they'd been reaching for `assert-eq
cosine 1.0` (a Story-2 mechanism asking a Story-1 question) where
the substrate already had the right predicate (`coincident?`,
arc 023) but no test wrapper for it. The helper lives in
`wat/std/test.wat`; lab adoption shipped concurrently.

**Shape:** four slices. Slice 1 changes the `HolonAST` schema in
`holon-rs` (5 typed leaves added; `Atom` narrowed from
`Arc<dyn Any>` to `Arc<HolonAST>`). Slice 2 makes `:wat::holon::Atom`
a polymorphic eval-and-dispatch (primitives → typed leaves;
HolonAST → opaque-Atom-wrap; WatAST → structural lowering [shipped
shape] OR Enum/Struct → loud error [original draft]). Slice 3
extends `wat-lru/src/shim.rs::hashmap_key` to accept HolonAST.
Slice 4 lands INSCRIPTION + USER-GUIDE rows + a BOOK chapter draft.

Total estimate: ~1.5 days. Actual: ~1 day including the path-2
pivot, workspace visibility fix, and `assert-coincident`.

**Cut from earlier draft:** auto-derive ToHolon for wat-declared
user types. Per BOOK Ch.48 (first-class enums; substrate doesn't
mechanically lower) and the user's principle ("the consumer always
knows better"), enum/struct values aren't directly Atom-able;
consumers write explicit one-line lowering helpers when they need
it. The lab today doesn't use enum-as-atom anywhere — the cut
removes ~3 hours of work and real risk while changing nothing the
lab depends on.

This arc is the substrate-side unblock for lab arc 030 slice 2
(encoding cache). The lab is paused on this; once shipped, the
cache work picks up and proof-perf-001 follows.

Builder direction (2026-04-25):

> "Atoms should only be able to hold HolonAST"

> "the number 42 is an AST?"

> "i feel like a massive refactor is more correct than avoiding it"

The DESIGN.md has the full Q1-Q9 set; this BACKLOG is
implementation-side per slice.

---

## Slice 1 — `HolonAST` schema change in `holon-rs`

**Status: shipped 2026-04-25** (`holon-rs` commit `c450da3`).
Implementation matches the planning sketch below; `AtomTypeRegistry`
retired entirely (rather than just shrunk) since typed leaves
left it with nothing to dispatch on. Per-variant accessors
(`as_i64` / `as_string` / `as_symbol` / `as_bool` / `atom_inner`)
replaced the old `atom_value<T>` downcast surface. Empty `Bundle`
materializes as a zero ternary vector (algebra's identity element)
to keep structurally-lowered `()` forms encodable.

`holon-rs/src/kernel/holon_ast.rs` — add 5 typed primitive leaf
variants; narrow the existing `Atom` variant from
`Arc<dyn Any + Send + Sync>` to `Arc<HolonAST>` (preserves the
opaque-identity wrapper semantic per BOOK Ch.54; kills the dyn Any
escape hatch). Manual `Hash + Eq` impls (f64 fields use `to_bits`).

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HolonAST {
    // Primitive leaves — vocabulary atoms (BOOK Ch.45).
    Symbol(Arc<str>),
    String(Arc<str>),
    I64(i64),
    F64(f64),
    Bool(bool),

    // Opaque-identity wrapper — strictly typed; no more dyn Any
    // (BOOK Ch.54 atom-vs-recursive-encoding distinction).
    Atom(Arc<HolonAST>),

    // Composites — similarity-preserving recursive encoding.
    Bind(Arc<HolonAST>, Arc<HolonAST>),
    Bundle(Arc<Vec<HolonAST>>),
    Permute(Arc<HolonAST>, i32),
    Thermometer { value: f64, min: f64, max: f64 },
    Blend(Arc<HolonAST>, Arc<HolonAST>, f64, f64),
}

impl Hash for HolonAST {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            HolonAST::Symbol(s) => s.hash(state),
            HolonAST::String(s) => s.hash(state),
            HolonAST::I64(n) => n.hash(state),
            HolonAST::F64(x) => x.to_bits().hash(state),
            HolonAST::Bool(b) => b.hash(state),
            HolonAST::Atom(h) => h.hash(state),
            HolonAST::Bind(a, b) => { a.hash(state); b.hash(state); }
            HolonAST::Bundle(xs) => xs.hash(state),
            HolonAST::Permute(a, k) => { a.hash(state); k.hash(state); }
            HolonAST::Thermometer { value, min, max } => {
                value.to_bits().hash(state);
                min.to_bits().hash(state);
                max.to_bits().hash(state);
            }
            HolonAST::Blend(a, b, w1, w2) => {
                a.hash(state); b.hash(state);
                w1.to_bits().hash(state); w2.to_bits().hash(state);
            }
        }
    }
}

impl PartialEq for HolonAST {
    fn eq(&self, other: &Self) -> bool {
        // Standard structural eq except f64 fields use bit-equality
        // (for Hash/Eq contract — same reasoning as Hash).
        ...
    }
}
```

Encoder (in `holon-rs/src/kernel/holon_ast.rs` `canonical_edn_holon`
+ `atom_seed`) — **byte-equivalent encoding for primitives** so
proof 002 numbers stay green. The new typed leaves emit the same
canonical bytes today's `Atom(payload)` does for the corresponding
type; only the Rust storage layer changes:

- `Symbol(s)` → `[TAG_ATOM, len("String"), "String", len(s), s.as_bytes()]` (matches today's `Atom(quote :foo)` byte-for-byte; keywords are stored as Strings starting with `:`)
- `String(s)` → `[TAG_ATOM, len("String"), "String", len(s), s.as_bytes()]` (matches today's `Atom("foo")`)
- `I64(n)` → `[TAG_ATOM, len("i64"), "i64", 8, n.to_le_bytes()]`
- `F64(x)` → `[TAG_ATOM, len("f64"), "f64", 8, x.to_le_bytes()]`
- `Bool(b)` → `[TAG_ATOM, len("bool"), "bool", 1, [b as u8]]`
- `Atom(h)` → `[TAG_ATOM, len("wat/algebra/Holon"), "wat/algebra/Holon", len(canonical_edn(h)), canonical_edn(h)]` (preserves today's "programs as atoms" path)

Composite walks (`Bind` / `Bundle` / `Permute` / `Thermometer` /
`Blend`) unchanged — they recurse through `canonical_edn_holon`
which now dispatches on the new variants.

`AtomTypeRegistry` retires (or shrinks heavily): `register_builtins`
no longer registers the 12 primitive types — primitives are typed
leaves now; the registry's per-type canonicalization is no longer
the path. The registry struct survives if `holon-rs` callers
outside this codebase still use it; otherwise it can be deleted in
a follow-up cleanup pass.

### Tests

`holon-rs/src/kernel/holon_ast.rs` `mod tests`:

1. `derive_hash_eq` — round-trip a HolonAST through a HashMap key.
2. `f64_to_bits_hash` — Thermometer{value: 0.1, ...} hashes consistently across instances.
3. `f64_nan_hash` — NaN bit patterns hash consistently (same NaN bits → same hash).
4. `encoder_symbol_round_trip` — `(:wat::holon::encode (Symbol :foo))` produces deterministic vector.
5. `encoder_int_vs_float` — `I64(42)` and `F64(42.0)` produce distinct vectors (no accidental cross-type collision).
6. `encoder_legacy_atom_compat` — encoding a `Symbol(":foo")` produces the SAME vector as today's `Atom(quote :foo)` (verified against a recorded reference vector if needed).

### Verification

```bash
cd /home/watmin/work/holon/holon-rs
cargo test --release
# All existing holon-rs tests must pass; new tests cover the schema change.
```

If existing tests break: investigate per-failure. Most should pass without modification because the public API (`encode`, `cosine`, etc.) doesn't change shape.

**LOC budget:**
- `holon-rs/src/kernel/holon_ast.rs`: enum redefinition (~80 LOC), Hash + PartialEq impls (~60 LOC), Debug update (~20 LOC).
- `holon-rs/src/kernel/encode.rs` (or wherever): per-leaf encoding (~50 LOC delta).
- `holon-rs/src/kernel/atom_registry.rs`: shrink `register_builtins` (~50 LOC removed).
- New tests: ~100 LOC.

**Estimated cost:** ~360 LOC. **~5 hours** (substantial schema + careful encoder migration).

---

## Slice 2 — `:wat::holon::Atom` polymorphic constructor

**Status: shipped 2026-04-25** (`wat-rs` commit `355721c`),
**under the path-2 pivot**: WatAST args lower structurally instead
of erroring. The structural lowering (`watast_to_holon`) plus the
new Story-2 recovery primitive `:wat::holon::to-watast`
(`holon_to_watast`) together compose to a clean form-as-coordinate
+ form-as-runnable-program pair. The 8 deftests sketched below
became the existing wat_cli demos un-ignored under the new shape
(`programs_are_atoms_hello_world`, `presence_proof_hello_world`).

`wat-rs/src/runtime.rs` — the `:wat::holon::Atom` evaluator becomes
a dispatch on the argument's runtime `Value` variant:

```rust
fn eval_holon_atom(args: &[WatAST], env: &Environment, sym: &SymbolTable) -> Result<Value, RuntimeError> {
    if args.len() != 1 { /* arity error */ }
    let v = eval(&args[0], env, sym)?;
    let h = match v {
        // Primitives → typed leaves (vocabulary atoms).
        Value::String(s)              => HolonAST::String(Arc::from(s.as_str())),
        Value::wat__core__keyword(k)  => HolonAST::Symbol(Arc::from(k.as_str())),
        Value::i64(n)                 => HolonAST::I64(n),
        Value::f64(x)                 => HolonAST::F64(x),
        Value::bool(b)                => HolonAST::Bool(b),
        // HolonAST → opaque-identity wrap (BOOK Ch.54).
        Value::holon__HolonAST(h)     => HolonAST::Atom(h),
        // Wat enum / struct: error loudly. Per BOOK Ch.48 the
        // substrate doesn't mechanically lower; consumer writes
        // explicit lowering helper.
        Value::Enum { .. } | Value::Struct(_) => {
            return Err(RuntimeError::TypeMismatch {
                op: ":wat::holon::Atom".into(),
                expected: "primitive (i64/f64/bool/String/keyword) or HolonAST; \
                           wat enum/struct values aren't directly representable in the \
                           algebra — write a wat-side lowering function returning the \
                           holon shape you want, then wrap in Atom for opaque identity",
                got: v.type_name(),
            });
        }
        // Lambda / Vector / ProgramHandle / etc.: same error.
        other => return Err(RuntimeError::TypeMismatch {
            op: ":wat::holon::Atom".into(),
            expected: "primitive or HolonAST",
            got: other.type_name(),
        }),
    };
    Ok(Value::holon__HolonAST(Arc::new(h)))
}
```

Update `check.rs` for the new return-type / accepted-argument shape if needed. The current type scheme should remain — return type is `:wat::holon::HolonAST`; argument is polymorphic / `:Any` style.

### Tests

`wat-rs/wat-tests/holon/atom-polymorphic.wat` — one deftest per dispatch case:

1. `test-atom-string` — `(:wat::holon::Atom "foo")` produces a HolonAST that cosine-self-equals 1.0.
2. `test-atom-keyword` — `(:wat::holon::Atom (:wat::core::quote :foo))` same.
3. `test-atom-i64` — `(:wat::holon::Atom 42)` same.
4. `test-atom-f64` — `(:wat::holon::Atom 3.14)` same.
5. `test-atom-bool` — `(:wat::holon::Atom true)` same.
6. `test-atom-passthrough` — `(:wat::holon::Atom (:wat::holon::Atom "foo"))` — Atom-of-already-HolonAST produces the inner unchanged (cosine to inner = 1.0).
7. `test-atom-cross-type-distinct` — `(:wat::holon::Atom 42)` and `(:wat::holon::Atom 42.0)` produce distinct vectors (cosine < 1.0).
8. `test-atom-symbol-vs-string` — `(:wat::holon::Atom :foo)` and `(:wat::holon::Atom "foo")` produce distinct vectors (different leaf variants with the "Symbol:" / "String:" canonical-bytes prefix).

### Verification

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release wat_suite
# All existing wat tests pass + 8 new atom-polymorphic tests.
```

**LOC budget:**
- `wat-rs/src/runtime.rs`: `eval_holon_atom` rewrite (~30 LOC delta).
- `wat-rs/wat-tests/holon/atom-polymorphic.wat`: 8 tests (~120 LOC).

**Estimated cost:** ~150 LOC. **~2 hours.**

---

## Slice 3 — `hashmap_key` accepts HolonAST

**Status: shipped 2026-04-25** (`wat-rs` commit `0bf01bc`).
Three deftests (`HolonKey.wat`: round-trip, distinguishes,
structural-equal) all green. Carry-along: `Cargo.toml::default-members`
makes `cargo test` workspace-wide by default; CacheService.wat's
post-arc-047 `Option<String>` rot fix shipped concurrently (was
the visibility-gap test that surfaced the workspace-default need).

`wat-rs/src/runtime.rs` — extend `hashmap_key` (line 4535):

```rust
pub fn hashmap_key(op: &str, v: &Value) -> Result<String, RuntimeError> {
    match v {
        Value::String(s)              => Ok(format!("S:{}", s)),
        Value::i64(n)                 => Ok(format!("I:{}", n)),
        Value::f64(x)                 => Ok(format!("F:{}", x.to_bits())),
        Value::bool(b)                => Ok(format!("B:{}", b)),
        Value::wat__core__keyword(k)  => Ok(format!("K:{}", k)),
        Value::holon__HolonAST(h) => {
            // Now derivable since slice 1.
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            h.hash(&mut hasher);
            Ok(format!("H:{:x}", hasher.finish()))
        }
        Value::Vector(v) => {
            // Bit-exact i8 hash.
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            v.as_slice().hash(&mut hasher);
            Ok(format!("V:{:x}", hasher.finish()))
        }
        other => Err(RuntimeError::TypeMismatch {
            op: op.into(),
            expected: "hashable value (primitive, HolonAST, Vector — or extend hashmap_key)",
            got: other.type_name(),
        }),
    }
}
```

`wat-rs/crates/wat-lru/src/shim.rs` — drop the "primitives only" panic message; defer to `hashmap_key` which now accepts richer types. Update doc comments. Existing rejection of Lambda / Function / etc. preserved (they're not hashable; not consumer-driven yet).

### Tests

`wat-rs/wat-tests/lru/holon-key.wat` — three deftests:

1. `test-localcache-holon-key-roundtrip` — `LocalCache<HolonAST, i64>` put/get works for a simple HolonAST.
2. `test-localcache-holon-key-distinguishes` — two distinct HolonASTs produce distinct cache slots (no false hits).
3. `test-localcache-holon-key-structural-equal` — two structurally-equal HolonASTs (built from independent constructors) produce the SAME cache slot (cache hit on the second build).

### Verification

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release wat_suite
# +3 tests; existing tests pass.
```

**LOC budget:**
- `wat-rs/src/runtime.rs::hashmap_key`: +20 LOC for HolonAST + Vector arms.
- `wat-rs/crates/wat-lru/src/shim.rs`: -10 LOC of "primitives only" diagnostic, +5 LOC clearer "not hashable" diagnostic.
- `wat-rs/wat-tests/lru/holon-key.wat`: ~80 LOC for 3 tests.

**Estimated cost:** ~100 LOC. **~1.5 hours.**

---

## Slice 4 — INSCRIPTION + USER-GUIDE + BOOK chapter

**Status: shipped 2026-04-25** (`wat-rs` commits `253c433` for
INSCRIPTION + USER-GUIDE + ZERO-MUTEX update; `holon-lab-trading`
commit `0f9de83` for BOOK Ch 59 *42 IS an AST*; commit `141a7a8`
for BOOK Ch 60 *Assert What You Mean* — the consumer-side
recognition that followed the substrate fix). Two BOOK chapters
landed instead of the planned one because the substrate close
spawned a parallel test-surface recognition worth its own chapter.

`wat-rs/docs/arc/2026/04/057-typed-holon-leaves/INSCRIPTION.md` — same shape as arc 056's INSCRIPTION:

- "What shipped" table per slice.
- LOC delta + test count delta.
- Architecture notes:
  - `Atom(Arc<dyn Any>)` removal — what broke (or didn't) in dependents.
  - ToHolon auto-derive details for enums/structs.
  - The proof 002 verification — same numbers? if not, what shifted?
  - AtomTypeRegistry shrinkage final state.
- "What this unblocks" — lab arc 030 slice 2; engram libraries; cross-process; Reckoner training data.
- "What this arc deliberately did NOT add" — `U8` / `Bytes` / `Char` leaves; serde impls; cross-process handoff; persistence.
- "The thread" — date timeline.

`wat-rs/docs/USER-GUIDE.md` — atom section reframes:
- "primitives are atoms" framing.
- `:wat::holon::Atom` as polymorphic constructor.
- Typed leaf variants documented.
- ToHolon trait + auto-derive section.
- AtomTypeRegistry section shrinks (or moves to historical-context).

`holon-lab-trading/BOOK.md` — new chapter (working title: **"The Sealed Holon"** or **"42 IS an AST"**). The principle: the algebra is closed; primitives ARE its atoms in the Lisp sense; `Atom` as a wrapper-around-anything was an inversion that we corrected. References Lisp's atom-as-predicate. Sets the stage for engram libraries / persistence / cross-process work that depends on a closed algebra.

Drafting the BOOK chapter is creative work — the user typically writes the prose; this slice provides the technical scaffolding (the principle + the implementation that proves it) and the user lands the chapter when ready.

### Verification

End-to-end:

```bash
# wat-rs full pass
cd /home/watmin/work/holon/wat-rs
cargo test --release

# lab full pass
cd /home/watmin/work/holon/holon-lab-trading
cargo test --release wat_suite

# proof 002 — same numbers?
cargo test --release --features proof-002 --test proof_002 -- --nocapture
ls -t runs/proof-002-*.db | head -1 | xargs -I{} sqlite3 {} \
  "SELECT thinker, COUNT(*), SUM(state='Grace'), SUM(state='Violence') \
   FROM paper_resolutions GROUP BY thinker ORDER BY thinker;"
# Expected: always-up | 34 | 0 | 34, sma-cross | 34 | 5 | 29
```

If proof 002 numbers shift, encoding-mechanics changed in a way
that altered cosines. Investigate before declaring slice 5 done.

**Estimated cost:** ~2 hours. Doc + verification.

---

## Verification end-to-end

After all five slices land:

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release

cd /home/watmin/work/holon/holon-lab-trading
cargo build --release
cargo test --release wat_suite
cargo test --release --features proof-002 --test proof_002

# wat-rs wat_suite count climbs by ~15 (atom-polymorphic + atom-user-types + holon-key tests).
# lab wat_suite stays at 336 (no test additions; same numbers from proof 002).
```

---

## Total estimate

- Slice 1: 5 hours (HolonAST schema + byte-equivalent encoder migration + tests)
- Slice 2: 2 hours (polymorphic Atom constructor + Enum/Struct error case + tests)
- Slice 3: 1.5 hours (hashmap_key extension + tests)
- Slice 4: 2 hours (INSCRIPTION + USER-GUIDE + BOOK chapter draft)

**~13.5 hours = ~2 days.** Heavier than arc 056 (`time::Instant`, ~half day) because of the schema change. Same shape as arc 053 (Phase 4 substrate, ~5 days); much smaller in scope.

---

## Out of scope

- **`U8` / wider integers / `Bytes` / `Char` leaves.** Ship per consumer demand.
- **Serde `Serialize` / `Deserialize` for HolonAST.** Hash + Eq derive enables the work; serde impl ships separately.
- **Cross-process AST handoff.** Same — enabled, not shipped.
- **`Atom?` predicate.** Trivial follow-up; ships when a consumer wants it.
- **Auto-derive `ToHolon` for wat-declared user types.** Per BOOK Ch.48 + Q3 — substrate doesn't mechanically lower; consumers write explicit one-line lowering helpers.
- **Removing `AtomTypeRegistry` entirely.** Shrinks dramatically in slice 1 but stays in place for the auto-derive registration path. Future arc may rip if measurement shows it's unused.
- **Performance optimization of the leaf walks.** Straightforward; if measurement shows a hot path, optimize then.

---

## Risks

**proof 002 numbers shift.** The encoding-mechanics migration
(registry-mediated → typed-leaf-direct) might produce different
canonical bytes for a given AST, shifting the resulting vector by
some amount. Mitigation: byte-equivalent canonical-bytes derivation
per leaf type (`SHA-256(b"Symbol:" + s)` should match today's
canonical_edn_holon for a Symbol-payload Atom). Verification gate:
proof 002 numbers in slice 1 + slice 5.

**ToHolon auto-derive correctness.** Wat enum/struct variants have
specific identities; the auto-derive maps variant names to Symbols.
A mistyped variant name in the generator produces wrong Symbols →
wrong vectors → wrong cache lookups. Mitigation: slice 3 tests
exercise this path; slice 5's lab pass catches downstream effects.

**`Atom` constructor breakage in edge cases.** `Atom`-of-tuple,
`Atom`-of-vec, `Atom`-of-ProgramHandle — these aren't currently
common but might exist. Polymorphic dispatch errors loudly;
diagnostic message names what's missing. Consumer surfaces
unsupported types if any.

**`AtomTypeRegistry::with_builtins` breakage in dependents.** Some
external code may depend on the registry containing primitive
canonicalizers. Mitigation: keep `with_builtins` API intact; have
it register the (now-vestigial) primitive entries as no-op
delegations to the new typed leaf paths, OR error loudly with
"primitives are typed leaves now; remove this registration."

**The migration's blast radius surprises.** I sampled
`grep -rn HolonAST::Atom` and got 16 Rust call sites + 98 wat call
sites. The wat surface is preserved; the Rust call sites need
mechanical rewrite. If a deeper search surfaces more dependencies
(e.g., `dyn Any` extracted from `Atom` somewhere unexpected), the
migration grows. Mitigation: incremental compile catches breakages;
each slice's `cargo test` is the gate.

**Lab `:trading::types::PhaseLabel` etc. as atoms.** These are wat-
declared enums; auto-derive ToHolon should cover them. Verification:
lab's test suite + proof 002 gate.

---

## What this unblocks

- **lab arc 030 slice 2** — encoding cache resumes immediately.
- **lab task #57** — `(ast-hash, d)` cache key for the dim-router has the right substrate underneath.
- **proof-perf-001** — cache hit-rate + speedup measurement becomes a real proof.
- **engram libraries** — Phase 4 work; per-AST identity makes engram match well-defined.
- **LogEntry serde** — when consumers cross processes (multi-machine proof scaling), serde derives.
- **Reckoner labeled-training-data** — `(surface, label)` pairs get stable identity for dedup / memoization.
- **The BOOK chapter** — names a principle the substrate has been groping toward; future work cites it.

PERSEVERARE.
