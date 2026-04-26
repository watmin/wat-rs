# wat-rs arc 057 — Typed HolonAST leaves — BACKLOG

**Shape:** five slices. Slice 1 changes the `HolonAST` schema in
`holon-rs` (typed leaves; remove `Atom(Arc<dyn Any>)`). Slice 2
makes `:wat::holon::Atom` a polymorphic constructor. Slice 3 ships
auto-derive `ToHolon` for wat-declared user types. Slice 4 extends
`wat-lru/src/shim.rs::hashmap_key` to accept HolonAST. Slice 5 lands
INSCRIPTION + USER-GUIDE rows + a BOOK chapter draft.

Total estimate: ~2 days.

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

**Status: not started.**

`holon-rs/src/kernel/holon_ast.rs` — replace the `Atom(Arc<dyn Any + Send + Sync>)`
variant with five typed leaf variants, plus a manual `Hash` impl
covering the f64-bearing variants.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HolonAST {
    // Leaves — terminal terms.
    Symbol(Arc<str>),
    String(Arc<str>),
    I64(i64),
    F64(f64),
    Bool(bool),

    // Composites — unchanged.
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

Encoder (in `holon-rs/src/kernel/encode.rs` or wherever
`encode_holon` lives) — update the per-variant match to consume
typed leaves. Each leaf produces a deterministic vector via
SHA-256-of-canonical-bytes (mirroring today's registry path):

- `Symbol(s)` → SHA-256(b"Symbol:" + s.as_bytes()) → `d`-bit signs
- `String(s)` → SHA-256(b"String:" + s.as_bytes()) → signs
- `I64(n)` → SHA-256(b"I64:" + n.to_le_bytes()) → signs
- `F64(x)` → SHA-256(b"F64:" + x.to_bits().to_le_bytes()) → signs
- `Bool(b)` → SHA-256(b"Bool:" + [b as u8]) → signs

Composite walks unchanged (Bind / Bundle / Permute / Thermometer /
Blend recurse into children).

`AtomTypeRegistry` shrinks dramatically: `register_builtins` no
longer registers the 12 primitive types (they're typed leaves now);
the registry survives only for `HolonAST` itself (programs-as-atoms
recursion still exists for ToHolon impls of user types — see
slice 3).

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

**Status: not started.** Depends on slice 1.

`wat-rs/src/runtime.rs` — the `:wat::holon::Atom` evaluator becomes
a dispatch on the argument's runtime `Value` variant:

```rust
fn eval_holon_atom(args: &[WatAST], env: &Environment, sym: &SymbolTable) -> Result<Value, RuntimeError> {
    if args.len() != 1 { /* arity error */ }
    let v = eval(&args[0], env, sym)?;
    let h = match v {
        Value::String(s)              => HolonAST::String(Arc::from(s.as_str())),
        Value::wat__core__keyword(k)  => HolonAST::Symbol(Arc::from(k.as_str())),
        Value::i64(n)                 => HolonAST::I64(n),
        Value::f64(x)                 => HolonAST::F64(x),
        Value::bool(b)                => HolonAST::Bool(b),
        Value::holon__HolonAST(h)     => (*h).clone(),
        // Struct / Enum / etc.: try ToHolon (slice 3); if not available, error.
        Value::Struct(_) | Value::Enum(_) => {
            return try_user_to_holon(&v, sym).ok_or_else(|| RuntimeError::TypeMismatch {
                op: ":wat::holon::Atom".into(),
                expected: "primitive, HolonAST, or user type with ToHolon",
                got: v.type_name(),
            });
        }
        other => return Err(RuntimeError::TypeMismatch {
            op: ":wat::holon::Atom".into(),
            expected: "primitive (i64/f64/bool/String/keyword), HolonAST, or user type with ToHolon",
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

## Slice 3 — Auto-derive `ToHolon` for wat-declared user types

**Status: not started.** Depends on slices 1 + 2.

Today's `:wat::core::enum` and `:wat::core::struct` declarations
generate accessors and constructors. This slice adds an
auto-generated `ToHolon` wat function per declared type:

```text
For an enum :T with variants V1, V2(field-types...):
   :T::to-holon : T -> :wat::holon::HolonAST
   ;; V1 (unit)        → (Symbol ":T::V1")
   ;; (V2 a b)         → (Bundle [(Symbol ":T::V2") <a-as-holon> <b-as-holon>])

For a struct :S with fields f1, f2, ...:
   :S::to-holon : S -> :wat::holon::HolonAST
   ;; (S/new a b ...) → (Bundle [(Symbol ":S") <a-as-holon> <b-as-holon> ...])
```

The substrate registers these `T::to-holon` functions automatically
when the enum / struct is declared (similar to how accessors get
auto-generated). `:wat::holon::Atom` consults a runtime registry of
ToHolon functions when its argument is a `Value::Struct` or
`Value::Enum`.

For non-wat-declared user types (raw Rust types via `#[wat_dispatch]`), a manual `ToHolon` impl is required. Document the
trait shape; out of scope for v1 to ship a derive-macro for Rust
(future arc).

### Tests

`wat-rs/wat-tests/holon/atom-user-types.wat` — tests using the
`:my::test::*` namespace's enum + struct declarations:

1. `test-enum-unit-variant-to-holon` — declared enum's unit variant produces `Symbol(":Enum::Variant")`.
2. `test-enum-tagged-variant-to-holon` — declared enum's tagged variant produces `Bundle([Symbol(":Enum::Variant"), <field-asts>])`.
3. `test-struct-to-holon` — declared struct produces `Bundle([Symbol(":Type"), <field-asts>])`.
4. `test-atom-of-enum` — `(:wat::holon::Atom my-direction-up)` works through ToHolon dispatch.

### Verification

```bash
cd /home/watmin/work/holon/wat-rs
cargo test --release wat_suite
# +4 tests; existing tests pass.

cd /home/watmin/work/holon/holon-lab-trading
cargo test --release wat_suite
# 336 lab tests still pass — nothing in the lab broke. The lab's
# enum / struct atoms now route through auto-derived ToHolon.
```

**LOC budget:**
- `wat-rs/src/runtime.rs` + `check.rs`: auto-derive emission (~150 LOC).
- ToHolon registry / lookup (~50 LOC).
- `wat-rs/wat-tests/holon/atom-user-types.wat`: 4 tests (~80 LOC).

**Estimated cost:** ~280 LOC. **~3 hours.**

---

## Slice 4 — `hashmap_key` accepts HolonAST

**Status: not started.** Depends on slice 1 (Hash derive).

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

## Slice 5 — INSCRIPTION + USER-GUIDE + BOOK chapter

**Status: not started.** Depends on slices 1-4.

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

- Slice 1: 5 hours (HolonAST schema + encoder migration + tests)
- Slice 2: 2 hours (polymorphic Atom constructor + tests)
- Slice 3: 3 hours (auto-derive ToHolon + tests)
- Slice 4: 1.5 hours (hashmap_key extension + tests)
- Slice 5: 2 hours (INSCRIPTION + docs)

**~13.5 hours = ~2 days.** Heavier than arc 056 (`time::Instant`, ~half day) because of the schema change. Same shape as arc 053 (Phase 4 substrate, ~5 days); much smaller in scope.

---

## Out of scope

- **`U8` / wider integers / `Bytes` / `Char` leaves.** Ship per consumer demand.
- **Serde `Serialize` / `Deserialize` for HolonAST.** Hash + Eq derive enables the work; serde impl ships separately.
- **Cross-process AST handoff.** Same — enabled, not shipped.
- **`Atom?` predicate.** Trivial follow-up; ships when a consumer wants it.
- **A Rust derive macro for `ToHolon`.** Wat-declared types get auto-derive; raw Rust types need manual impl. Macro is a future ergonomic uplift.
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
