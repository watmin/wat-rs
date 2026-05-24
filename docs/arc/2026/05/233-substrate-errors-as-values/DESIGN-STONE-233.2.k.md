# Sub-DESIGN — Stone 233.2.k — Value::Tracked variant retirement + Environment stores TrackedValue

**Status:** ACTIVE (2026-05-23 late late). Sub-DESIGN under arc 233 Stone 233.2 chain (j ✓ → **k** → l → e).

**Driver:** Stone 233.2.j shipped the eval_inner cascade + 5 producer migrations but left ONE documented exemption: `bind_let_binding` re-wraps as `Value::Tracked` to preserve producer provenance through let-bindings (Environment stores bare Value). The exemption expires at this stone. The ONLY honest path that eliminates the exemption permanently is **Option A — Environment stores TrackedValue**. After this stone, `Value::Tracked` ceases to exist entirely; Stone 233.2.l seals the meta-class via proc-macro.

User direction 2026-05-23 late late:

> *"i never want to experience this path again — i want to prove that we walked here and prove we never need to come here again"*

This sub-DESIGN articulates the variant retirement scope, the Environment storage flip that dissolves the Phase 5 exemption, and the four-questions verdict.

## The exemption that must dissolve

From Stone 233.2.j's SCORE (the unplanned Phase 5 fix):

```rust
// src/runtime.rs:6207-6212 (CURRENT — to be removed in 233.2.k)
let value = match provenance {
    Provenance::Unknown => tv.value_owned(),
    _ => Value::Tracked { // #[probe-3-exempt: let-binding provenance preservation — expires at Stone 233.2.k]
        inner: Box::new(tv.value_owned()),
        provenance,
    },
};
Ok(scope.child().bind(name, value).build())
```

This re-wrap exists because:
- `eval_inner` returns `TrackedValue` (post-233.2.i)
- `Environment.bindings` stores bare `Value` (HashMap<String, Value>)
- Provenance would be LOST at the boundary; the re-wrap preserves it via `Value::Tracked`

For Stone 233.2.k to retire `Value::Tracked`, the re-wrap site must go. Three options were considered:

| Option | Mechanic | Verdict |
|---|---|---|
| **A** | Environment stores `TrackedValue` directly (HashMap<String, TrackedValue>) | **CHOSEN** — permanent structural fix |
| B | Accept provenance loss until 233.2.e; mark probes 6/7/8 `#[ignore]` with recovery note | REJECTED — `#[ignore]` is dishonest deferral (`feedback_no_known_defect_left_unfixed`) |
| C | Side-channel: parallel `HashMap<Symbol, Provenance>` next to bindings | REJECTED — same family as Value::Tracked variant (carrier-side-by-side); doesn't structurally close the class |

## Option A scope (the cascade)

### Phase 1 — Environment storage type flip

```rust
// src/runtime.rs:1267 + 1304 (Environment + EnvironmentBuilder)
// Before
bindings: HashMap<String, Value>,
// After
bindings: HashMap<String, TrackedValue>,
```

### Phase 2 — Environment API signature flips

- `EnvironmentBuilder::bind(name, value: Value)` → `bind(name, tv: TrackedValue)`
- `Environment::lookup(name) -> Option<&Value>` → `Option<&TrackedValue>` (or `Option<TrackedValue>` if owned semantics needed)
- `bindings.insert` at line 1310 receives `TrackedValue` directly
- 6 known lookup callers update (mechanical; most likely already had `.into_tracked()` post-lookup which now becomes redundant)

### Phase 3 — bind_let_binding simplification

```rust
// After 233.2.k
LetBinding::Single { name, rhs } => {
    let tv = eval_inner(rhs, scope, sym)?;
    Ok(scope.child().bind(name, tv).build())  // direct; no re-wrap
}
LetBinding::Destructure { names, rhs } => {
    let value = eval_inner(rhs, scope, sym)?.value_owned();
    let elements = destructure_tuple(&value, names.len(), ":wat::core::let")?;
    let mut builder = scope.child();
    for (name, elem) in names.iter().zip(elements) {
        // Destructure slots get Unknown provenance — each slot has its own
        // origin which we'd need separate tracking for (out of scope; arc 233.2.e
        // revisits if/when destructure provenance becomes load-bearing).
        builder = builder.bind(name, TrackedValue::from(elem));
    }
    // ...
}
```

Destructure preserves current behavior (Unknown for destructured slots; not a regression).

### Phase 4 — Variant + helper retirement

DELETE:

1. `Value::Tracked { inner: Box<Value>, provenance: Provenance }` variant — `src/runtime.rs:~613`
2. `Value::inner()` helper at `src/runtime.rs:1166-1170` — no longer needed; Value is never wrapped
3. `Value::provenance()` helper at `src/runtime.rs:1178-1183` — was only meaningful for Tracked
4. `Value::into_tracked()` helper at `src/runtime.rs:1188-1201` — Phase 5 fix becomes redundant; callers use `TrackedValue::from(value)` directly

### Phase 5 — Match arm cleanup (now structurally dead)

REMOVE these Value::Tracked match arms (the variant no longer exists; arms would be compile errors):

- `src/runtime.rs:1011` — Hash impl `unreachable!` arm
- `src/runtime.rs:1159` — type_name() arm
- `src/runtime.rs:17940` — render_value() arm (unreachable!)
- `src/edn_shim.rs:1696` — value_to_edn_with passthrough arm
- `src/closure_extract.rs:1733` — closure_extract handling arm
- Any Eq/PartialEq Value::Tracked arms

### Phase 6 — Call-site sweep

- **~19 `.inner()` call sites** — strip the call entirely. Value is never wrapped post-233.2.k; `.inner()` is now a no-op on bare Value (it returned self when not Tracked). Replace `v.inner()` with `v`.
- **~26 `.into_tracked()` call sites** — replace with `TrackedValue::from(v)`. The helper's job was "extract Provenance from Tracked OR wrap with Unknown"; post-retirement only the second case matters; that's `TrackedValue::from`.
- **`tv.value().inner().type_name()` patterns** in ValueSnapshot::of_tracked etc. — strip the `.inner()` (e.g., `tv.value().type_name()`).

### Phase 7 — Probe cleanup

- Remove `// #[probe-3-exempt: ...]` mechanism from `tests/probe_stone_233_2_j_producer_migration.rs`. The exemption expired as documented. The Phase 5 re-wrap site is gone; probe 3's zero-construction assertion holds without exemption.
- Update Stone 233.2.j probe's own assertion to reflect post-retirement state if needed.

### Phase 8 — New regression-guard probe

Author `tests/probe_stone_233_2_k_variant_retired.rs`:

1. **Static scan: zero `Value::Tracked` references in src/** (no construction, no match, no comments-as-active-code — comments about historical retirement OK)
2. **Static scan: `Value` enum source doesn't contain `Tracked`** (variant deletion verified)
3. **Behavioral: producer-tagged value survives let-binding** (probes 6/7/8 baseline; regression guard for the structural fix)
4. **Behavioral: Environment.lookup() returns TrackedValue** (compile-time type check)
5. **Static scan: `Value::inner()` + `Value::provenance()` + `Value::into_tracked()` helpers are DELETED** (no `pub fn inner` / `pub fn provenance` / `pub fn into_tracked` on impl Value)

## Doctrine — what this enables structurally

**Pre-state:** Value::Tracked variant exists (post-233.2.j) but is unreachable at producer sites (modulo the one bind_let_binding exemption). The CLASS still exists in source — a future author could add `Value::AnotherWrap { ... }` and re-introduce the trap-door.

**Post-state (233.2.k):** Value::Tracked variant is DELETED from source. The exemption mechanism is REMOVED. Pattern-matching on Value never has to consider a wrapping variant because the substrate's Value enum genuinely contains zero wrapping variants. The class instance is structurally absent.

**Post-state (233.2.l):** The proc-macro `#[wat_value]` on the Value enum makes RE-INTRODUCTION compile-error. The class is annihilated at the meta-layer.

Per FAILURE-ENGINEERING.md ✅✅✅: with 233.2.k + 233.2.l together, the SITUATION that produces the trap-door (a Value variant wrapping another Value with metadata) becomes structurally impossible — both at the current substrate AND at future authoring time.

## Four-questions verdict

| Question | Verdict |
|---|---|
| **Obvious?** | YES — Value::Tracked deleted; Environment stores TrackedValue (mirroring eval_inner's return type); .inner()/.provenance()/.into_tracked() helpers gone |
| **Simple?** | YES — each piece is atomic (variant delete, helper delete, env storage flip, mechanical call-site sweep). Cascade is contained (6 lookup callers, 19 .inner() callers, 26 .into_tracked() callers; ~50-100 mechanical sites total) |
| **Honest?** | YES — exemption from Stone 233.2.j is DISSOLVED, not deferred; no #[ignore] markers; no convention enforcement; structural fix |
| **Good UX?** | YES — provenance Just Works through let-bindings (no exception); pattern-match-on-Value can NEVER miss a wrapping variant (because none exist); helper API simplifies (one TrackedValue::from path instead of Value::into_tracked + TrackedValue::new + TrackedValue::from) |

PROCEED.

## Scope (this stone)

- Environment.bindings storage flip + 6 lookup callers + bind builder
- bind_let_binding simplification (remove re-wrap)
- 4 helpers deleted (Value::Tracked variant, Value::inner, Value::provenance, Value::into_tracked)
- Match arm cleanup (Hash, Display, type_name, render_value, edn_shim passthrough, closure_extract)
- ~19 .inner() call sites stripped
- ~26 .into_tracked() call sites → TrackedValue::from
- Probe 3 exemption mechanism removed
- New regression-guard probe (5 contracts)
- SCORE doc

## Out of scope (affirmative scope-bounding)

- **Stone 233.2.l proc-macro structural seal** — sub-DESIGN at `57eced2`; lands after 233.2.k
- **Stone 233.2.e AST-derived provenance for destructure slots / recv / try-recv** — separate stone; doesn't gate 233.2.k
- **runtime_def_values HashMap<String, Value>** at line 1494 — separate concern; defmacro-level; not provenance-carrying in the let-binding sense. STAYS as Value unless evidence surfaces it needs TrackedValue (out of scope for 233.2.k)
- **holon-rs** — NOT touched
- **HARD CUT** — no deprecation alias for Value::inner() / Value::provenance() / Value::into_tracked()

## Calibration prediction

| Stone | Predicted |
|---|---|
| 233.2.k | **60–120 min Mode A; 180 min STOP** |

Smaller than 233.2.j cascade (~50-100 sites vs 383). Most work is mechanical deletion + sweep. Environment storage flip is the architectural touch but small (6 callers + 2 internal HashMap declarations + builder pattern).

## Trap-door audit (FM 2-bis pre-flight)

- [x] Environment.bindings storage type identified (HashMap<String, Value> × 2 instances at lines 1267 + 1304)
- [x] bindings.insert is the single insertion site (line 1310)
- [x] env.lookup caller count: 6 (small cascade)
- [x] bind_let_binding re-wrap site identified (line 6207-6212; the exemption)
- [x] Value::Tracked match arm sites enumerated (Hash 1011, type_name 1159, inner 1169, provenance 1181, into_tracked 1199, render_value 17940, edn_shim 1696, closure_extract 1733)
- [x] .inner() + .into_tracked() call-site counts measured (19 + 26)
- [ ] Verify destructure-path doesn't accidentally regress provenance through any path other than the intentional Unknown for destructured slots (probe at minimum; sonnet may need to audit destructure-related tests)
- [ ] Verify runtime_def_values doesn't carry producer-attached provenance (audit during sweep; STOP-6 if it does — escalate)

## Builds on / unblocks

**Builds on:**
- 233.2.h (TrackedValue mint) — the storage type Environment now uses
- 233.2.i (eval boundary returns TrackedValue) — provenance flow shape proven
- 233.2.j (producer migration + Phase 5 bind_let_binding fix) — establishes the value flow that this stone simplifies

**Unblocks:**
- **Stone 233.2.l** — proc-macro structural seal. CAN'T apply `#[wat_value]` to Value while Value::Tracked still exists (would fail seal at first compile). 233.2.k MUST land first.
- **arc216 stone1 7 probes** (task #496) — auto-resolve when Value::Tracked variant is structurally absent (the trap-door class instance ceases to exist)
- **Stone 233.2.e** — AST-derived provenance on the structurally-sealed substrate

## The annihilation (this is the stone where the class dies)

Stone 233.2.j prepared. Stone 233.2.k EXECUTES. Stone 233.2.l SEALS.

After 233.2.k commits, the Value enum has zero wrapping variants. `Value::Tracked` does not exist. The trap-door class has no living instance in source. Pattern-matching on Value is structurally honest — every variant is dispatchable; no shadowing possible.

After 233.2.l commits, the `#[wat_value]` proc-macro rejects future wrapping variants at compile time. The class is closed at the meta-layer too.

**The walk is proven by the commit chain:** arc 233 Stone 233.1 → 233.2.a/b/c/d/f/g/h/i/j → **233.2.k** → 233.2.l. Each stone + SCORE + INSCRIPTION is the journey. **The next-walk-impossible** is sealed by the proc-macro at 233.2.l.

## Cross-references

- `DESIGN-STONE-233.2.md` — sub-stone table (this stone is row 233.2.k)
- `DESIGN-STONE-233.2.j.md` — Phase 5 unplanned fix that this stone resolves
- `DESIGN-STONE-233.2.l.md` (commit `57eced2`) — proc-macro seal that depends on 233.2.k landing
- `SCORE-STONE-233.2.j.md` — establishes the let-binding exemption + recovery plan
- `tests/probe_stone_233_2_j_producer_migration.rs` — probe 3 exemption mechanism to remove
- `scratch/FAILURE-ENGINEERING.md` — annihilation-not-patch doctrine driving Option A choice over Option B
- `feedback_no_known_defect_left_unfixed` — disqualifies Option B's #[ignore] approach
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF discipline
