# Sub-DESIGN — Arc 234 Stone 234.0 — `:wat::core::type` polymorphic primitive

**Status:** ACTIVE (2026-05-24). Sub-DESIGN authored; FM 2-bis probe + BRIEF + EXPECTATIONS in flight.

**Builds on:**
- Arc 232 Stone 232.0 — `:wat::core::apply` polymorphic TypeScheme precedent (commit `50e82d9`)
- Arc 232 Stone 232.0a — `:wat::holon::extract-classifier` (consumed for HolonAST arm; commit `a1e4b02`)
- Arc 230 — uniform classifier encoding (HolonAST classifier-wrap shape)
- Arc 203 — `:wat::core::struct` (TypeDef registration; precedent for struct type-name lookup)

**Unblocks:**
- Revised Stone 232.1 — `:wat::core::defprotocol` polymorphic dispatcher consumes `:wat::core::type`
- Arc 234.1 — `Value::wat_record` variant + Eq/Hash/Display/HolonRep impls
- All subsequent arc 234.x stones

---

## Doctrine

`:wat::core::type` is the substrate primitive that extracts a value's record-type FQDN as a String, regardless of underlying storage backend. It's the dispatch primitive defprotocol consumes for routing and the basis for the polymorphic record-y verbs in arc 234.3.

Stone 234.0 is the smallest substrate addition in arc 234 — one new eval fn + one dispatch arm + one TypeScheme registration. Pure substrate primitive; no macros; no Value variants added.

---

## Locked decisions

### D1 — Signature

```
(:wat::core::type <any-value>) -> :wat::core::String
```

Accepts ANY Value variant; returns the type FQDN as String. Single arg; no Option-wrapping (every Value has a type).

### D2 — Dispatch table (initial; arc 234.0 scope)

| Value variant | Returns | Notes |
|---|---|---|
| `Value::holon__HolonAST(h)` | `extract_classifier(h).unwrap_or_else(\|\| "wat::holon::HolonAST".to_string())` | HolonAST classifier-wraps (defrecord instances) return the wrapped class name; non-wrap HolonAST returns the variant name |
| `Value::Struct(sv)` | `sv.type_name.trim_start_matches(':').to_string()` | `StructValue.type_name` is the FQDN WITH leading colon (e.g., `:myapp::Foo`); strip the colon for consistency with `extract_classifier` convention (FQDN without leading colon, e.g., `"myapp::Foo"`) |
| `Value::wat__std__Vector(...)` | `"wat::core::Vector"` (or parameterized if available) | Verify via `Value::type_name()` |
| `Value::wat__std__HashMap(...)` | `"wat::core::HashMap"` | Verify via `Value::type_name()` |
| `Value::wat__std__HashSet(...)` | `"wat::core::HashSet"` | Verify via `Value::type_name()` |
| Any other Value | `Value::type_name().to_string()` | Existing Rust method returns FQDN per arc 224 Stone 224.5 ("L1-runtime-2: Value::type_name() Sender/Receiver returns honest wat-visible kind") |

### D3 — wat-record arm deferred to Stone 234.1

`Value::wat_record { class_fqdn, .. }` does NOT exist yet (Stone 234.1 mints the variant). Stone 234.0's dispatch table handles ALL CURRENTLY-EXISTING Value variants. When 234.1 ships, type's dispatch gains one new arm (~5 lines) returning `class_fqdn`.

This is honest stepping-stone discipline: 234.0 ships something complete-and-useful (`:wat::core::type` works on every Value that exists today, including defrecord instances via the HolonAST arm); 234.1 extends seamlessly.

### D4 — Polymorphic TypeScheme

```rust
env.register(
    ":wat::core::type".into(),
    TypeScheme {
        type_params: vec!["T".into()],
        params: vec![TypeExpr::Var("T".into())],
        ret: TypeExpr::Path(":wat::core::String".into()),
        rest_param_type: None,
    },
);
```

Forall T. T -> String. Apply's TypeScheme (Stone 232.0) is the precedent for "accepts any value" registration.

### D5 — Single canonical name

`:wat::core::type`. No aliases. HARD CUT per `feedback_wat_llm_first_design`.

### D6 — Not for type-checking; for runtime dispatch

`:wat::core::type` returns a String at RUNTIME. It is not a type-check operation; it's a type-NAME extraction for dispatch purposes (per the defprotocol dispatcher template in arc 234 DESIGN).

For compile-time type checking, the existing `:wat::holon::is?` + `:wat::holon::is-<X>?` family (arc 226) handles that. Different concerns; both ship.

---

## Implementation surface

**`src/runtime.rs`:**

1. New `fn eval_type` (~30 lines) — accepts args + list_span + env + sym; arity check (1 arg); evaluates the arg; matches on the resulting Value variant per D2's dispatch table; returns `Value::String(Arc::new(type_str))`.

2. New dispatch arm in `dispatch_keyword_head_value` (1 line):
   ```rust
   ":wat::core::type" => eval_type(args, list_span, env, sym),
   ```

3. (Possibly) helper for struct type-name lookup if TypeDef reference isn't directly accessible from Value — verify during FM 2-bis probe phase.

**`src/check.rs`:**

1. New `register_builtins` entry per D4 (TypeScheme registration; ~10 lines).

2. (Possibly) special-case in `infer_list` if the polymorphic TypeScheme doesn't naturally infer correctly — verify during probe.

**Tests:**

1. `tests/probe_diagnostic_polymorphic_type.rs` — FM 2-bis probe (authored BEFORE BRIEF; commits as design substrate).

---

## FM 2-bis probe plan

Probe authored + committed BEFORE BRIEF. Contracts:

1. `(:wat::core::type 5)` → `"wat::core::i64"` (primitive arm)
2. `(:wat::core::type "hello")` → `"wat::core::String"` (primitive arm)
3. `(:wat::core::type [1 2 3])` → `"wat::core::Vector"` or similar (Vector arm)
4. `(:wat::core::type {:a 1})` → `"wat::core::HashMap"` (HashMap arm)
5. `(:wat::core::type (:myapp::Voltage 5.0))` → `"myapp::Voltage"` (HolonAST classifier-wrap; defrecord instance; consumes extract-classifier)
6. `(:wat::core::type (some-struct-instance))` → `"myapp::SomeStruct"` (struct arm; if struct TypeDef lookup is straightforward)
7. `(:wat::core::type :foo)` → `"wat::core::keyword"` (keyword primitive)
8. `(:wat::core::type true)` → `"wat::core::bool"` (bool primitive)

Probe initially FAILS (verb doesn't exist). Post-stone PASSES all contracts.

If a probe surfaces an unexpected `Value::type_name()` output (e.g., `"i64"` instead of `"wat::core::i64"`), the design adjusts at the FM 2-bis level BEFORE the BRIEF ships. Substrate-as-teacher applied early.

---

## Substrate-as-teacher cascade

The probe is the BRIEF's design substrate. Sonnet's iteration cycle:
- Write `eval_type` per D2's dispatch table
- Run probe; observe which variants need adjustment
- Iterate (likely 1-2 cycles maximum given the small substrate change)

The 8 probe contracts cover the surface area; cargo's error output names what's wrong.

---

## Trap-door audit (per FM 2-bis BRIEF discipline)

Pre-emptive concerns to verify in the probe + BRIEF:

1. **`Value::type_name()` output strings** — verify each variant returns FQDN. Arc 224 Stone 224.5 explicitly fixed Sender/Receiver to return honest wat-visible kind. Other variants may need similar verification; if probe surfaces non-FQDN output for any variant, sub-DESIGN's D2 needs adjustment.

2. **HolonAST classifier-wrap fallback** — `extract_classifier(holon)` returns `Option<String>`. For non-wrap HolonAST (e.g., bare Atom, Bundle without classifier-wrap shape), the fallback is `"wat::holon::HolonAST"`. Document the fallback explicitly so future readers know non-wrap HolonAST returns the variant name, not a sentinel like `"unknown"`.

3. **Struct TypeDef lookup** — `Value::wat__core__struct` carries a TypeDef reference. Verify (via grep) that the TypeDef's FQDN name is reachable from the eval context (probably via SymbolTable lookup). If not directly reachable, may need a helper that walks SymbolTable's TypeEnv.

4. **Polymorphic TypeScheme inference** — `infer_list` may need a special-case for `:wat::core::type` if the generic T-var doesn't propagate correctly through inference. Apply's `infer_apply` in `src/check.rs` is precedent; mirror its handling if needed.

5. **Parametric type display** — `Value::type_name()` may return parametric forms like `"wat::core::Vector<wat::core::i64>"` for typed collections. Decision: return whatever `type_name()` produces (no synthesis); the user gets the type as the substrate sees it. Honest.

6. **Future wat-record arm coordination** — note in eval_type's doc comment that arc 234.1 will add a `Value::wat_record` arm; sonnet adds a stub comment marker (`// TODO: arc 234.1 adds wat_record arm here returning class_fqdn`) so the future addition is friction-free.

---

## Risks

- **Value::type_name() returns inconsistent FQDN** — mitigation: FM 2-bis probe surfaces empirically; sub-DESIGN adjusts; small cost
- **Struct TypeDef lookup requires substrate plumbing** — mitigation: arc 203 + 215 already have precedent for struct type access; reuse pattern
- **infer_list parametric T-var handling** — mitigation: apply primitive (Stone 232.0) is precedent for "accepts any value" inference; follow same pattern
- **Parametric collection types** — mitigation: return whatever `type_name()` produces; no synthesis (D6 is "extract, don't synthesize")

---

## Out-of-scope (explicit)

- `Value::wat_record` arm — Stone 234.1
- Field access (record-y verbs) — Stone 234.3
- Hash-destructure — Stone 234.4
- VSA verb auto-dispatch on records — Stone 234.5
- Migration of `:wat::holon::defrecord` user surface — Stone 234.6
- defrecord macro itself — Stone 234.2 (the macro uses `:wat::core::type` internally but ships in its own stone)
- Type-checking semantics (`is?` family) — already shipped per arc 226; not duplicated here

---

## Calibration prediction

**Target band:** 30–60 min Mode A
**Upper bound (STOP-3):** 90 min
**Confidence:** high — smallest substrate addition in arc 234; one eval fn + one dispatch arm + one TypeScheme. Apply primitive (Stone 232.0) at ~30 min real precedent for "single polymorphic substrate verb" calibration.

**Rationale:**
- ~30 lines `eval_type` Rust
- ~10 lines `register_builtins` entry
- ~150 lines new probe (8 contracts)
- ~5 min compile + iterate cycle
- ~10 min SCORE writing

Sonnet's calibration trend post-arc-233 stays under-band consistently; this stone fits the pattern.

---

## STOP triggers (REJECTION criteria)

- STOP-1: unexpected compile errors not tracing to the new eval_type / dispatch arm / TypeScheme entry
- STOP-2: baseline lib tests regress below 827
- STOP-3: 90 min elapsed (apply partial-state-grading per `feedback_partial_state_grading`)
- STOP-4: holon-rs touched (frozen since 530650c)
- STOP-5: clippy warnings above 54
- STOP-6: scope creep — Value::wat_record variant, macro work, record-y verbs, destructure
- STOP-7: FM 2-bis probe doesn't flip 0/8 → 8/8 (the load-bearing row)
- STOP-8: any arc 233 regression guard regresses
- STOP-9: any arc 232.0a probe (typed-entities reflection) regresses

---

## What this unblocks

- **Revised Stone 232.1** — `:wat::core::defprotocol` + `:wat::core::extend-type` polymorphic via `:wat::core::type` (no longer at `:wat::holon::*` per arc 234 doctrine)
- **Stone 234.1** — `Value::wat_record` variant (type primitive extends with one arm)
- **Stone 234.3** — polymorphic record-y verbs (assoc + record->map + record? + record->holon + keyword-as-accessor); all consume `:wat::core::type` for dispatch routing
- **All subsequent arc 234.x stones** — the polymorphic type primitive is the dispatch foundation

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc umbrella; the hologram thesis
- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.0.md` — forthcoming
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.0.md` — forthcoming
- `tests/probe_diagnostic_polymorphic_type.rs` — FM 2-bis probe (forthcoming; commits before BRIEF)
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0.md` — apply primitive precedent (TypeScheme + dispatch arm + eval fn shape)
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.0a.md` — extract-classifier precedent (small primitive shape)
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
- `feedback_dr_branch_salvage.md` — pattern for handling supersession (not expected here; this stone is small enough not to thrash scope)
