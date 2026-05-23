# BRIEF — Arc 233 Stone 233.2.a — mint Provenance enum + Value::Tracked variant + transparency contracts

## What we're doing

Scaffolding sub-stone for Value-level provenance. Add:

1. **Provenance enum** with 4 variants — `Unknown` (default) / `Literal { span }` / `SymbolBound { binding_span, head_span }` / `RuntimeBuilt { producer: &'static str, call_span: Span }`
2. **Value::Tracked variant** — `Tracked { inner: Box<Value>, provenance: Provenance }`
3. **Transparency contracts** — Eq/Hash/PartialEq/Display/Debug/Clone all unwrap Tracked; Value↔HolonAST serialization paths unwrap Tracked
4. **Helpers** — `Value::inner() -> &Value` (recursive unwrap if Tracked-wrapping-Tracked) + `Value::provenance() -> Provenance` (Unknown if not Tracked; Clone-returns)
5. **ValueSnapshot::of update** — extract Provenance from Tracked at construction (currently always Unknown post-233.1)

**No behavioral change visible at wat level.** No producers tag yet (that's 233.2.b). Existing 827 tests must all pass — Tracked introduction is transparent.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.md`** — the sub-DESIGN that locks Shape C (Value::Tracked wrapper variant). Implementation shape is NOT REOPENABLE in this stone. If you hit a wall, STOP + surface in SCORE.

2. **`src/runtime.rs:372`** — `pub enum Value` definition. Sweep target: ADD `Tracked { inner: Box<Value>, provenance: Provenance }` variant.

3. **`src/runtime.rs`** (around line 1700 — ValueSnapshot/Provenance live near RuntimeError) — Provenance enum lives here (the v1 `Unknown`-only version is from 233.1). Extend with the 3 new variants (Literal / SymbolBound / RuntimeBuilt).

4. **`src/comms/mod.rs:90`** — `pub trait HolonRepresentable`. NOT a Value impl; this serializes Rust types. Tracked doesn't directly affect HolonRepresentable. But Value↔HolonAST CONVERSION paths (to_holon, from_holon, etc.) DO need transparency.

5. **`src/runtime.rs:17382`** — `fn render_value(v: &Value, depth: usize) -> String`. Already used by ValueSnapshot::of. Sonnet adds Tracked arm that delegates to inner (transparency).

## The type design (locked by sub-DESIGN)

```rust
#[derive(Debug, Clone)]
pub enum Provenance {
    Unknown,
    Literal { span: Span },
    SymbolBound { binding_span: Span, head_span: Span },
    RuntimeBuilt { producer: &'static str, call_span: Span },
}

pub enum Value {
    // ── existing variants unchanged ──
    Unit,
    bool(bool),
    i64(i64),
    // ... all 30+ variants unchanged

    // ── NEW (233.2.a) ──
    Tracked {
        inner: Box<Value>,
        provenance: Provenance,
    },
}

impl Value {
    /// Returns &Value with Tracked unwrapped recursively. If self is
    /// Tracked, returns inner.inner() (handles Tracked-of-Tracked).
    /// Otherwise returns self.
    pub fn inner(&self) -> &Value {
        match self {
            Value::Tracked { inner, .. } => inner.inner(),
            other => other,
        }
    }

    /// Returns the Provenance attached to this Value. If self is Tracked,
    /// returns the wrapper's provenance. Otherwise returns Provenance::Unknown.
    /// Note: doesn't recurse — only the outermost Tracked's provenance is returned.
    pub fn provenance(&self) -> Provenance {
        match self {
            Value::Tracked { provenance, .. } => provenance.clone(),
            _ => Provenance::Unknown,
        }
    }
}
```

`ValueSnapshot::of` extends:

```rust
pub fn of(v: &Value) -> Self {
    ValueSnapshot {
        type_name: v.inner().type_name(),
        rendered: render_value(v.inner(), 0),
        provenance: v.provenance(),
    }
}
```

(`v.inner().type_name()` unwraps Tracked; `v.provenance()` extracts the wrapper's provenance.)

## Transparency contracts (each must be implemented + tested)

For each contract, sonnet adds a unit test verifying transparency.

### Contract 1 — Display unwraps

```rust
let bare = Value::i64(42);
let tracked = Value::Tracked {
    inner: Box::new(Value::i64(42)),
    provenance: Provenance::Unknown,
};
assert_eq!(format!("{}", bare), format!("{}", tracked));
// or via render_value:
assert_eq!(render_value(&bare, 0), render_value(&tracked, 0));
```

### Contract 2 — Eq compares inner

```rust
let bare = Value::i64(42);
let tracked = Value::Tracked {
    inner: Box::new(Value::i64(42)),
    provenance: Provenance::Unknown,
};
assert_eq!(bare, tracked);
assert_eq!(tracked, bare);

// Tracked-wrapping-Tracked equals bare too
let double = Value::Tracked {
    inner: Box::new(tracked.clone()),
    provenance: Provenance::RuntimeBuilt {
        producer: "test",
        call_span: Span::unknown(),
    },
};
assert_eq!(double, bare);
```

### Contract 3 — Hash unwraps (load-bearing for HashMap/HashSet correctness)

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_of(v: &Value) -> u64 {
    let mut h = DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

let bare = Value::i64(42);
let tracked = Value::Tracked {
    inner: Box::new(Value::i64(42)),
    provenance: Provenance::Unknown,
};
assert_eq!(hash_of(&bare), hash_of(&tracked));

// Verify HashMap behavior: insert with bare key, lookup with tracked key
let mut map: std::collections::HashMap<Value, &str> = std::collections::HashMap::new();
map.insert(bare.clone(), "hello");
assert_eq!(map.get(&tracked), Some(&"hello"));
```

### Contract 4 — Clone preserves Tracked-ness

```rust
let original = Value::Tracked {
    inner: Box::new(Value::i64(42)),
    provenance: Provenance::RuntimeBuilt {
        producer: "test",
        call_span: Span::unknown(),
    },
};
let cloned = original.clone();
assert_eq!(original, cloned);
match cloned {
    Value::Tracked { ref provenance, .. } => match provenance {
        Provenance::RuntimeBuilt { producer, .. } => assert_eq!(*producer, "test"),
        _ => panic!("provenance lost"),
    },
    _ => panic!("Tracked lost"),
}
```

### Contract 5 — value.inner() recurses

```rust
let bare = Value::i64(42);
let single = Value::Tracked {
    inner: Box::new(bare.clone()),
    provenance: Provenance::Unknown,
};
let double = Value::Tracked {
    inner: Box::new(single.clone()),
    provenance: Provenance::Unknown,
};
assert_eq!(single.inner(), &bare);
assert_eq!(double.inner(), &bare);  // double-wrap unwraps to original
```

### Contract 6 — value.provenance() returns outermost

```rust
let inner = Value::Tracked {
    inner: Box::new(Value::i64(42)),
    provenance: Provenance::Literal { span: Span::unknown() },
};
let outer = Value::Tracked {
    inner: Box::new(inner.clone()),
    provenance: Provenance::RuntimeBuilt {
        producer: "outer",
        call_span: Span::unknown(),
    },
};
match outer.provenance() {
    Provenance::RuntimeBuilt { producer, .. } => assert_eq!(producer, "outer"),
    _ => panic!("expected outermost RuntimeBuilt"),
}
```

### Contract 7 — ValueSnapshot::of extracts Provenance from Tracked

```rust
let tracked = Value::Tracked {
    inner: Box::new(Value::wat__core__keyword(Arc::new(":foo".to_string()))),
    provenance: Provenance::RuntimeBuilt {
        producer: "test-producer",
        call_span: Span::unknown(),
    },
};
let snap = ValueSnapshot::of(&tracked);
assert_eq!(snap.type_name, "wat::core::keyword");
assert!(snap.rendered.contains(":foo"));
match snap.provenance {
    Provenance::RuntimeBuilt { producer, .. } => assert_eq!(producer, "test-producer"),
    _ => panic!("ValueSnapshot didn't extract provenance"),
}
```

### Contract 8 — Bare Value's ValueSnapshot has Unknown provenance

```rust
let bare = Value::wat__core__keyword(Arc::new(":foo".to_string()));
let snap = ValueSnapshot::of(&bare);
assert_eq!(snap.type_name, "wat::core::keyword");
match snap.provenance {
    Provenance::Unknown => {}, // expected
    other => panic!("bare Value should have Unknown provenance; got {:?}", other),
}
```

## Implementation surface

### Step 1 — Extend Provenance enum (src/runtime.rs)

Locate the existing Provenance enum (added in Stone 233.1; currently only has Unknown). Add 3 new variants:

```rust
#[derive(Debug, Clone)]
pub enum Provenance {
    Unknown,
    Literal { span: Span },
    SymbolBound { binding_span: Span, head_span: Span },
    RuntimeBuilt { producer: &'static str, call_span: Span },
}
```

### Step 2 — Add Value::Tracked variant (src/runtime.rs:372)

Add the variant. Be careful: Value derives or impls Eq, Hash, PartialEq, Clone, Debug — all need transparency aware handling.

### Step 3 — Implement transparency in Eq/PartialEq

Custom impl Value's PartialEq (or derive + manual override). The match arm for Tracked should compare its inner against the other side (after unwrapping if the other side is also Tracked).

```rust
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        // Tracked transparency: always compare inner values
        match (self.inner(), other.inner()) {
            // ... existing variant-pair comparisons against inner ...
        }
    }
}
```

Sonnet picks the cleanest impl shape.

### Step 4 — Implement transparency in Hash

```rust
impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Transparency: hash inner; ignore Tracked wrapper
        self.inner().hash_inner_only(state);
    }
}
```

(Sonnet may need to split the existing Hash impl into a helper that operates on bare variants.)

### Step 5 — Implement transparency in Display / Debug / render_value

```rust
fn render_value(v: &Value, depth: usize) -> String {
    if depth > SHOW_MAX_DEPTH {
        return "…".to_string();
    }
    match v.inner() {  // ← unwrap Tracked first
        // ... existing variant arms unchanged ...
    }
}
```

### Step 6 — Add helpers Value::inner() + Value::provenance()

Per the type design above.

### Step 7 — Update ValueSnapshot::of

Per the type design above.

### Step 8 — Test file: tests/probe_value_tracked_transparency.rs

Create a test file with the 8 transparency contracts as test functions. All must pass.

### Step 9 — Sweep Value-construction sites for compile errors

Adding Value::Tracked variant + transparency-aware Eq/Hash may break match-exhaustiveness elsewhere. Find + fix:

```bash
cargo build --release -p wat 2>&1 | grep "non-exhaustive\|missing.*Tracked"
```

For each non-exhaustive match: add `Value::Tracked { inner, .. }` arm that delegates to `inner.something()` (transparency). Sonnet adds + audits.

## Out of scope (affirmative scope-bounding)

- Tagging Values with non-Unknown Provenance — 233.2.b (the first producer) + 233.2.c (sweep) + 233.2.d (AST-derived)
- Provenance rendering in error messages — 233.2.b will extend ValueSnapshot::Display when actual provenance starts appearing
- HolonRepresentable trait changes — out of scope (Tracked doesn't affect Rust-type serialization)
- Cross-boundary provenance transport (process/thread boundaries) — Tracked stays local; future arc if needed
- Performance tuning (Arc<Provenance> interning, etc.) — v2 territory
- holon-rs — NOT touched
- wat-edn — NOT touched

## Verification flow

```
cargo build --release -p wat                          # 0 errors (Tracked added cleanly)
cargo test --release --lib -p wat --no-fail-fast      # baseline maintained: ≥ 827 passed
cargo test --release --test probe_value_tracked_transparency
                                                       # 8/8 transparency tests PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors
                                                       # 5/5 still PASS (no regression from 233.1)
cargo clippy --release --lib -p wat -- -D warnings    # 52 warns (baseline match)
git -C /home/watmin/work/holon/holon-rs/ status --short # empty
```

## STOP triggers (REJECTION criteria — never permission-to-defer)

- **STOP-1:** unexpected compile errors that aren't variant-exhaustiveness fixes
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** 180 min elapsed (upper-bound; Value-enum-wide change)
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** new clippy warning beyond pre-existing 52
- **STOP-6:** scope creep — tagging actual producers, rendering Provenance in errors, HolonRepresentable changes, etc.
- **STOP-7:** any transparency contract fails — Eq/Hash/Display MUST unwrap Tracked
- **STOP-8:** Value-construction sites stop working (TypeMismatch / NotCallable construction broken)
- **STOP-9:** Implementation shape deviation — sub-DESIGN locks Shape C; don't reopen wrap-vs-field-vs-variant

If any STOP fires: ship NOTHING beyond the clean-stoppable state; surface as honest delta in SCORE.

## Trap-door audit

Per arc 232.0's lessons + the sub-DESIGN's audit section:

- **NO invented syntax** — Provenance enum is plain Rust; helpers are plain methods
- **NO made-up types** — Span exists (`src/span.rs:48`); Value exists; ValueSnapshot from 233.1
- **NO phantom transparency claims** — every contract has a test (the 8 above)
- **Implementation shape locked** — Shape C from sub-DESIGN. No wrap-in-TrackedValue-struct. No per-variant fields.
- **HashMap correctness is correctness-critical** — Contract 3 verifies bare/tracked Values hash + lookup identically. Don't ship without this passing.

### Specific traps flagged from pre-spawn audit (2026-05-23 night)

**Trap 1 — Hash impl recursion + discriminant tagging discipline.**

`impl Hash for Value` at `src/runtime.rs:794` uses `std::mem::discriminant` tagging FIRST, then per-variant payload hashing. If your transparency impl writes:

```rust
fn hash<H: Hasher>(&self, state: &mut H) {
    std::mem::discriminant(self).hash(state);  // ← WRONG: includes Tracked's discriminant
    match self {
        Value::Tracked { inner, .. } => inner.hash(state),
        // ... per-variant
    }
}
```

Contract 3 breaks: `bare.hash() != tracked.hash()` because Tracked has its own discriminant.

**Correct pattern:** apply `self.inner()` BEFORE the discriminant tag, AND in the match:

```rust
fn hash<H: Hasher>(&self, state: &mut H) {
    let unwrapped = self.inner();  // recursively unwraps Tracked
    std::mem::discriminant(unwrapped).hash(state);
    match unwrapped {
        // ... per-variant; NO Tracked arm needed (inner() never returns Tracked)
    }
}
```

`Value::inner()` recurses through Tracked-of-Tracked layers, so the match never sees a Tracked variant. The match's exhaustiveness check is satisfied without a Tracked arm. Discriminant tagging operates on the unwrapped variant.

Apply the same `self.inner()` + `other.inner()` discipline throughout `impl PartialEq for Value` at line 705.

**Trap 2 — unreachable!() arms for non-hashable variants.**

`src/runtime.rs:10404+` documents `unreachable!()` Hash arms for opaque variants (Sender / Receiver / fn / RustOpaque / etc.) — these are NOT atomizable per `is_atomizable` (src/check.rs:3623). The static guarantee is that only atomizable Values reach Hash contexts.

If `inner()` returns one of these variants (because someone wrapped an opaque Value in Tracked), the `unreachable!()` will fire. That's CORRECT behavior — opaque Values shouldn't be Hash keys regardless of Tracked wrapping. The Tracked unwrap doesn't change the atomizability contract.

Verification: Contract 3's tests use atomizable variants only (i64, keyword). Don't test Tracked-wrapping-an-opaque-Value — that's not a supported case.

**Trap 3 — Existing tests with `_` catch-all match arms.**

Most tests construct Values directly + don't match. But some may use `match value { Value::X(...) => ..., _ => panic!("unexpected") }` patterns. After Tracked addition, a `_` arm would silently accept Tracked instead of panicking. This is BEHAVIORAL not COMPILE-TIME — cargo build won't catch it.

Audit: grep `tests/` and `src/` for `match` arms over `Value` that end in `_ =>`. For each, decide: does the test EXPECT Tracked to be accepted (transparently) or REJECTED (panic)? In v1, no producers tag, so no test should encounter Tracked unexpectedly. If sonnet adds new tests that construct Tracked, those need explicit assertions.

Low risk in 233.2.a because no producers emit Tracked yet. Flag for 233.2.b+.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly)
- HARD CUT — no aliases. No "if you can't unwrap, use the wrapped form" fallback
- Per `feedback_inscription_immutable`: SCORE is a new file
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.md` — sub-DESIGN; Shape C locked
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.1.md` — ValueSnapshot scaffolding from 233.1
- `src/runtime.rs:372` — Value enum (sweep target)
- `src/runtime.rs` near ValueSnapshot — Provenance enum (extend)
- `src/runtime.rs:17382` — render_value (transparency in Display path)
- `src/span.rs:48` — Span type (carries file/line/col)
- arc 216 Stone 216.5a — `impl Hash + PartialEq + Eq for Value` (the precedent; Tracked transparency follows this pattern)
- `feedback_sonnet_writes_substrate` — protocol; sonnet writes substrate
- `feedback_wat_colon_quote` — no inner colons inside `<>` (probably not relevant here but standard audit)
