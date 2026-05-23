# Sub-DESIGN — Stone 233.2.l — #[wat_value] proc-macro structural seal

**Status:** ACTIVE (2026-05-23 late late). Sub-DESIGN under arc 233 Stone 233.2 chain (j → k → l → e).

**Driver:** Stones 233.2.j (producer migration) + 233.2.k (variant retirement) eliminate the CURRENT trap-door class instance (Value::Tracked). Stone 233.2.l prevents the META-CLASS: any FUTURE wrapping-style variant added to `Value` would re-introduce the same trap-door. Per FAILURE-ENGINEERING.md: ✅✅✅ standard — "design cannot be dishonest." The proc-macro emits a **compile error** at the construction site (the enum definition itself), structurally forbidding the pattern at the highest layer possible.

User direction 2026-05-23 late late:

> *"we just debated the need for a Rust macro that eliminates the problem we're chasing now... i never want to experience this path again — i want to prove that we walked here and prove we never need to come here again"*

> *"annihilation of failure domains is direction"*

This sub-DESIGN articulates the proc-macro mechanic, applies it to the `Value` enum after 233.2.k retirement, and ensures the rule catches the dead `Value::Tracked` shape while allowing legitimate composition variants (`Vec<Value>`, `Option<Value>`, `Result<Value, Value>`, etc.).

## The trap-door class (structural definition)

A "wrapping variant" is a Value enum variant V such that:
- V contains exactly ONE pointer-to-Self field (`Box<Value>`, `Arc<Value>`, `Value`, `Rc<Value>`)
- V optionally contains metadata fields (Provenance, Span, etc.)
- The TYPE of the wrapped Value can be ANY variant of Value

**Why this is the trap-door:** `match v { Value::SpecificVariant(x) }` against `v = V { inner: Box::new(Value::SpecificVariant(...)) }` does NOT match. The caller forgets to unwrap; the dispatch silently misses. The class is reproducible (3+ incidences this session even with discipline-only enforcement).

**Why container variants are NOT trap-doors:**
- `Vec(Arc<Vec<Value>>)`: match dispatches on `Vec` variant, no shadowing of inner items
- `Option(Arc<Option<Value>>)`: same — match dispatches on `Option`, inner is opaque
- `Result(Arc<Result<Value, Value>>)`: same
- `HashMap(Arc<HashMap<Value, Value>>)`: same

Container variants have their OWN match dispatch; their inner Values are accessed via container APIs, not through the variant's match-arm direct destructure.

## Rule (precise)

**Forbidden field types on a Value variant (reject at compile time):**

1. `Value` directly (e.g., `enum Value { Wrap(Value) }`)
2. `Box<Value>`, `Arc<Value>`, `Rc<Value>` (single-instance smart-pointer to Self)
3. `Box<Box<Value>>`, `Arc<Box<Value>>`, etc. (nested forms of the above)

**Allowed field types (pass):**

1. Primitive: `i64`, `f64`, `bool`, `char`, `String`, `Arc<String>`, etc.
2. Collection: `Vec<Value>`, `Arc<Vec<Value>>`, `HashMap<K, Value>`, `Arc<HashMap<Value, Value>>`, `HashSet<Value>`, etc.
3. Sum-type containers: `Option<Value>`, `Arc<Option<Value>>`, `Result<Value, Value>`, `Arc<Result<Value, Value>>`
4. Tuples not containing the forbidden pattern as a SOLE field: `(i64, String)`, etc.
5. Specifically: types whose **outermost type constructor is NOT a smart-pointer-of-Self**

**Escape hatch (explicit opt-in):**

```rust
#[wat_value]
pub enum Value {
    i64(i64),
    String(Arc<String>),
    Vec(Arc<Vec<Value>>),  // OK — container variant
    
    // If a legitimate future use case requires wrapping (none known today),
    // the author MUST explicitly opt-in per variant:
    #[wat_value(allow_wrapping = "<reason>")]
    SomeFutureWrap { inner: Box<Value>, meta: SomeData },
}
```

The opt-in requires a documented reason string. The reason becomes part of the compile-time record; reviewers see WHY the wrap was allowed.

## Compile-time error shape

When a wrapping variant is detected without opt-in:

```
error: #[wat_value]: variant `Tracked` has wrapping shape (single Box<Self> field)
  --> src/runtime.rs:613:5
   |
613 |     Tracked { inner: Box<Value>, provenance: Provenance },
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: wrapping variants are forbidden because they silently mis-dispatch
           pattern-match on Value::X(...) — the inner Value::X gets shadowed.
   = note: this is the trap-door class arc 233 eliminated (see Stone 233.2.f).
   = note: if your use case GENUINELY requires wrapping, add
           #[wat_value(allow_wrapping = "your reason")] to this variant.
   = note: more often the right fix is a SIBLING TYPE outside Value
           (e.g., wat::runtime::TrackedValue per Stone 233.2.h).
```

The error MUST teach. Per `docs/SUBSTRATE-AS-TEACHER.md` — every error is a lesson. The substrate's own panic-as-EDN doctrine + arc 233's diagnostic-richness work apply to proc-macro errors too.

## Implementation surface

### Crate location

The proc-macro lives in `wat-macros/` (existing crate, used by `#[wat_dispatch]`, `#[restricted_to]`, etc.). New file: `wat-macros/src/wat_value.rs`.

### Macro signature

```rust
#[proc_macro_attribute]
pub fn wat_value(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse `args` for optional opt-in flags (allow_wrapping, etc. — at struct level not used; per-variant attrs handled in input scan)
    // Parse `input` as syn::ItemEnum
    // For each variant:
    //   Check for #[wat_value(allow_wrapping = "...")] attribute
    //   If absent, walk field types for forbidden pattern
    //   If forbidden pattern found, emit compile_error! with diagnostic
    // Return input unchanged (or with #[wat_value(...)] attrs stripped from variants)
}
```

### Detection algorithm

For each variant V:
1. Skip if V has `#[wat_value(allow_wrapping = "...")]` attribute (with non-empty reason)
2. For each field F in V:
   - If F's type is `Self` / `EnumName`: REJECT
   - If F's type is `Box<T>` / `Arc<T>` / `Rc<T>` AND T is `Self` / `EnumName` / nested smart-pointer of same: REJECT
   - Otherwise: ALLOW
3. If ALL fields ALLOW: variant passes

The macro takes the enum's IDENT (e.g., `Value`) and uses it as the self-reference token. This lets the same macro work on multiple enums (future-proofing — e.g., `HolonAST` could adopt the same seal).

### Application

```rust
// src/runtime.rs (post-233.2.k)
use wat_macros::wat_value;

#[wat_value]  // ← the structural seal
pub enum Value {
    Unit,
    bool(bool),
    i64(i64),
    f64(f64),
    Char(char),
    String(Arc<String>),
    Symbol(Arc<String>),
    Keyword(Arc<String>),
    // ... all leaf + container variants ...
    Vec(Arc<Vec<Value>>),                              // OK — container
    HashMap(Arc<HashMap<Value, Value>>),               // OK — container
    HashSet(Arc<HashSet<Value>>),                      // OK — container
    Option(Arc<Option<Value>>),                        // OK — sum container
    Result(Arc<Result<Value, Value>>),                 // OK — sum container
    // NO Tracked variant — retired in 233.2.k
    // ... no future wrapping variant compiles without explicit opt-in
}
```

## FM 2-bis probe plan

Write `wat-macros/tests/probe_wat_value_seal.rs` (or `tests/probe_stone_233_2_l_structural_seal.rs` in main crate) BEFORE the BRIEF:

### Contract 1 — Forbidden variant rejected at compile time

`compile_fail` test using `trybuild` (or similar):

```rust
// tests/ui/wat_value_rejects_wrapping.rs (trybuild fixture)
use wat_macros::wat_value;
use std::boxed::Box;

#[wat_value]
pub enum BadValue {
    Leaf(i64),
    Wrap { inner: Box<BadValue> },  // ← should FAIL to compile
}

fn main() {}
```

Expected stderr contains: `wrapping variants are forbidden` and `inner: Box<BadValue>`.

### Contract 2 — Container variants pass

`compile_pass` test:

```rust
#[wat_value]
pub enum OkValue {
    Leaf(i64),
    Vec(Vec<OkValue>),
    Option(Option<OkValue>),
    Map(std::collections::HashMap<OkValue, OkValue>),
}
```

Compiles without error.

### Contract 3 — Opt-in escape hatch works

```rust
#[wat_value]
pub enum DocumentedValue {
    Leaf(i64),
    
    #[wat_value(allow_wrapping = "legacy interop with foreign type system; see arc N+5")]
    Wrap { inner: Box<DocumentedValue> },  // ← compiles because of opt-in
}
```

Compiles. The reason string is preserved (verify via macro tests or runtime introspection).

### Contract 4 — Real Value enum compiles

After 233.2.k retires Value::Tracked, applying `#[wat_value]` to the actual `pub enum Value` in src/runtime.rs compiles cleanly.

### Contract 5 — Adversarial: ensure macro can't be bypassed via type alias

```rust
type BoxedValue = Box<EvilValue>;

#[wat_value]
pub enum EvilValue {
    Wrap { inner: BoxedValue },  // ← still REJECTED
}
```

The macro must resolve type aliases (or at minimum, reject patterns whose AST shape matches the forbidden form even through alias indirection). May require limited type alias resolution; sonnet picks the approach (likely: scan for the pattern conservatively, mark as REJECTED any inner type that COULD resolve to a smart-pointer-of-Self; user can always use `#[wat_value(allow_wrapping = "...")]` to opt-in).

## Shape decisions

### Decision 1 — Detection mechanism

**Option (a) — Pure syntactic scan (chosen):** walk the syn TypePath for each field; reject if the outer Path segment is `Box`/`Arc`/`Rc` AND the inner generic argument resolves syntactically to the enum's own name. Pros: simple, fast, no need for type-resolver dependencies. Cons: type aliases bypass (mitigated by contract 5 — opt-in if needed).

**Option (b) — Semantic resolution:** invoke rustc's type resolver via proc-macro to resolve aliases. Pros: catches alias bypass. Cons: complex; pulls heavy deps; not worth the cost given opt-in escape hatch.

**Verdict:** Option (a). Cover alias-bypass risk via documentation in the macro's user docs ("if you alias a forbidden type, you bypass the seal; consider whether you want to").

### Decision 2 — Where the opt-in attribute lives

**Option (a) — Per-variant `#[wat_value(allow_wrapping = "reason")]` (chosen):** attribute on the offending variant. Per-variant granularity; reason string documents WHY.

**Option (b) — Enum-level `#[wat_value(allow_any_wrapping)]`:** blanket opt-out. **Rejected** — defeats the purpose; if you can disable the seal whole-cloth, the seal is convention not structural.

**Verdict:** Option (a). Per-variant opt-in with reason. Future authors document WHY.

### Decision 3 — Where the macro applies

**Option (a) — Only `pub enum Value` in src/runtime.rs (chosen):** focused scope; arc 233 closes; future enums adopt as needed.

**Option (b) — All enums in wat-rs:** sweep all enums. **Rejected** — premature generalization. Apply where we know the trap-door class manifests; expand when new enums surface the same shape.

**Verdict:** Option (a). HolonAST, WatAST, etc. may adopt later — separate stones if/when needed.

## Four-questions verdict

| Question | Verdict |
|---|---|
| **Obvious?** | YES — `#[wat_value]` on the enum; compile error if a future author tries to add a wrapping variant; error message names what to do |
| **Simple?** | YES — single proc-macro fn; syntactic scan; per-variant opt-in escape hatch; no semantic resolver |
| **Honest?** | YES — names the class precisely (wrapping = single smart-pointer-of-Self field); names the trap-door it prevents; opt-in escape hatch with mandatory reason string documents legitimate exceptions |
| **Good UX?** | YES — substrate-as-teacher error message; opt-in carries why; macro lives in existing wat-macros crate (no new crate; familiar location) |

PROCEED.

## Scope (this stone)

- Mint `#[wat_value]` proc-macro in `wat-macros/src/wat_value.rs`
- Export via `wat-macros/src/lib.rs` (`pub use wat_value::wat_value;`)
- Apply to `pub enum Value` in src/runtime.rs (post-233.2.k)
- Write 5 contract tests (trybuild-based or manual probe)
- Verify cargo build clean + 827 baseline holds
- SCORE doc

## Out of scope (affirmative scope-bounding)

- Application to HolonAST / WatAST / other enums (separate stones if needed)
- Type alias semantic resolution (opt-in covers the corner)
- Lint-level enforcement on USER code (this is a substrate-level seal; users define their own types via defrecord which uses different mechanics)
- Conversion of existing `#[derive(...)]` macros (out of scope)

## Calibration prediction

| Stone | Predicted |
|---|---|
| 233.2.l | **45–90 min Mode A; 120 min STOP** |

Smaller than 233.2.j cascade. Focused proc-macro work + 5 tests + application to Value. The trybuild test framework may require some setup; budget 15 min for that.

## Trap-door audit (FM 2-bis pre-flight)

- [x] Detection algorithm catches Value::Tracked shape (verified by inspection)
- [x] Detection algorithm allows Vec<Value> / Option<Value> / Result<Value, Value> (verified by inspection)
- [ ] Opt-in escape hatch's reason string is non-empty (test contract 3)
- [ ] Type-alias bypass risk documented (cover via test contract 5 + macro user docs)
- [ ] Error message includes file:line:variant + recovery hint (sonnet implements to match SUBSTRATE-AS-TEACHER doctrine)
- [ ] Real Value enum compiles post-application (test contract 4 — runs against the actual runtime.rs file)

## Builds on / unblocks

**Builds on:**
- 233.2.k (Value::Tracked retired) — the proc-macro can't apply to a Value enum that still has Value::Tracked (would fail the seal at first compile). 233.2.k MUST land first.
- 233.2.h (TrackedValue mint) — the error message recommends "use TrackedValue per Stone 233.2.h" as the legitimate sibling-type alternative to wrapping.

**Unblocks:**
- arc 233 Stone 233.2.e (AST-derived provenance on fully-sealed substrate) — once 233.2.l ships, the substrate is structurally guaranteed wrap-free; 233.2.e can introduce new mechanisms safely.
- arc 233 Stone 233.3 (Errors-as-EDN extension) — independent.
- arc 233 Stone 233.4 (INSCRIPTION) — gated on l + e shipping.
- Any future enum that adopts `#[wat_value]` for the same seal (HolonAST, WatAST candidates).

## The annihilation (this is the load-bearing point)

Stone 233.2.j ships the migration. Stone 233.2.k retires the variant. Stone 233.2.l makes the trap-door class **structurally impossible to construct**: a future author cannot ADD a wrapping variant to Value without explicitly typing `#[wat_value(allow_wrapping = "reason")]` and documenting WHY.

This is the FAILURE-ENGINEERING.md ✅✅✅ standard:

| Standard | Mechanism | What it catches |
|---|---|---|
| ✅ | Convention | Author remembers the rule; remembers to call .inner() (FAILS in practice; we fell through 3+ times this session) |
| ✅✅ | Convention + CI | Lint or test enforces; catches AFTER construction; fails at build time |
| ✅✅✅ | Structural | Compile error AT construction; the dishonest move CANNOT be expressed |

The proc-macro is ✅✅✅. The SITUATION that produces the trap-door (a Value variant wrapping another Value with metadata) cannot exist in the source. Same shape as ZERO-MUTEX's "the situation that produces the failure is never constructed."

**The walk is proven** by the commit chain: arc 233 Stone 233.1 → 233.2.a → ... → 233.2.j → 233.2.k → 233.2.l. Each commit + SCORE + INSCRIPTION is the proof we walked here. **The next-walk-impossible** is sealed by the proc-macro: no future session can re-introduce the class without ceremonial opt-in.

## Cross-references

- `DESIGN-STONE-233.2.md` — sub-stone table (this stone is row 233.2.l)
- `DESIGN-STONE-233.2.g.md` — Shape A pivot that mandated TrackedValue (the legitimate sibling-type alternative the proc-macro recommends)
- `DESIGN-STONE-233.2.h.md` — TrackedValue mint (the legitimate sibling type)
- `DESIGN-STONE-233.2.j.md` — producer migration cascade (enables 233.2.k)
- (forthcoming) `DESIGN-STONE-233.2.k.md` — variant retirement (prerequisite for 233.2.l)
- `scratch/FAILURE-ENGINEERING.md` — the doctrine driving ✅✅✅ structural seal
- `docs/SUBSTRATE-AS-TEACHER.md` — error-message-as-lesson doctrine
- `wat-macros/src/lib.rs` — host crate for the new proc-macro
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF discipline
