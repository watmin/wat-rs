# DESIGN — Arc 234 Stone 234.5 — `:wat::holon::*` auto-dispatch on `Value::wat__Record`

**Status:** ACTIVE (2026-05-24 — orchestrator-authored sub-DESIGN; sonnet implements per BRIEF).

**Predecessor:** Stones 234.0, 234.1, 234.1.5, 234.2a (+ correction), 234.2b SHIPPED. The hologram has STORAGE (variant) + SUBSTRATE PRIMITIVES + USER MACRO. Stone 234.5 is the **VSA-integration completion** — the step that proves the hologram property end-to-end.

**Discipline:** sonnet writes substrate; orchestrator briefs + scores. Per `feedback_sonnet_writes_substrate`.

---

## The thesis

The hologram is two simultaneously-addressable representations: `struct_form` (Rust-fast) + `holon_form` (VSA-aligned). For the hologram to be REAL — not just claimed — VSA operations must accept records DIRECTLY, using the pre-built `holon_form` automatically.

Today: `(:wat::holon::cosine record1 record2)` fails because the verbs expect HolonAST args, not records. The user must explicitly convert: `(:wat::holon::cosine (:wat::core::record->holon r1) (:wat::core::record->holon r2))`. Clunky; defeats the hologram's UX value.

After Stone 234.5: records flow through VSA verbs natively. The substrate auto-unwraps `holon_form` at the arg boundary; no user-facing conversion call. **The hologram property becomes externally observable.**

---

## Scope — five verbs auto-dispatch on `Value::wat__Record`

| Verb | Why auto-dispatch matters |
|---|---|
| `:wat::holon::to-holon` | THE bridge verb. Returns the record's `holon_form` directly (no recomputation; per arc 234 DESIGN line 205). |
| `:wat::holon::Bind` | Constructor. When either arg is a record, unwrap to holon_form. Enables compositional VSA construction: `(:wat::holon::Bind classifier-h record-h)`. |
| `:wat::holon::Bundle` | Constructor. When any child is a record, unwrap. Enables `(:wat::holon::Bundle [r1 r2 r3])`. |
| `:wat::holon::cosine` | VSA-proof verb. Returns f64 similarity. End-to-end demo: construct two records via macro + measure their cosine. |
| `:wat::holon::extract-classifier` | Algebraic type-extraction (sibling of `:wat::core::type`). Returns the classifier String. Records carry their class_fqdn in holon_form's outer Bind. |

Other `:wat::holon::*` verbs (`Permute`, `Thermometer`, `Blend`, `presence?`, `Atom`, `is?`, `is-Map?`, etc.) are NOT in 234.5's scope — they're covered by the broader migration sweep in Stone 234.6 if needed. The 5-verb scope above is the load-bearing minimum for end-to-end hologram proof.

---

## Locked decisions

### D1 — Implementation pattern: centralized "ensure HolonAST" helper

Add a helper fn in `src/runtime.rs` (e.g., `value_to_holon_ast(v: &Value) -> Option<HolonAST>` or `coerce_to_holon(v: Value, span: &Span) -> Result<HolonAST, RuntimeError>`) that:
1. If `v` is `Value::holon__HolonAST(h)` → returns `(*h).clone()` (existing case)
2. If `v` is `Value::wat__Record { holon_form, .. }` → returns `(*holon_form).clone()` (NEW; unwraps the pre-built holon_form)
3. Otherwise → returns Err with a clear "expected HolonAST or wat::Record" diagnostic

Each of the 5 verbs uses this helper for its HolonAST-typed args. Records and HolonASTs both flow.

Sonnet picks the exact helper name + signature shape; the contract above is what matters.

### D2 — `:wat::holon::to-holon` adds the `Value::wat__Record` arm

`to_holon_inner` (runtime.rs line 15198) is the polymorphic UP body. Add a `Value::wat__Record { holon_form, .. }` arm that returns `(*holon_form).clone()` (or unwraps Arc via `Arc::try_unwrap` / `as_ref().clone()`).

Per DESIGN umbrella line 205: "polymorphic bridge accepts wat_records natively." This makes `(:wat::holon::to-holon r)` work + reuses the existing polymorphic dispatch.

### D3 — `Bind`, `Bundle`, `cosine`, `extract-classifier` use the centralized helper

Each verb's HolonAST-arg extraction routes through the D1 helper. The verb's call shape stays the same; just the arg-extraction logic broadens.

### D4 — check.rs TypeScheme broadening

Each of the 5 verbs has a TypeScheme declaring HolonAST-typed params. The check.rs side needs to ACCEPT `:wat::Record` in those positions polymorphically.

Sonnet investigates:
- Are the existing TypeSchemes registered with `:wat::holon::HolonAST` as a fixed type?
- Can a custom inference handler (like Stone 234.2a-CORRECTION's `infer_record_of`) accept BOTH types in HolonAST positions?
- Or: does `:wat::holon::HolonAST` type-narrowing already work via subtyping/union?

The precedent: Stone 234.2a-CORRECTION minted `infer_record_of` as a custom handler. Similar pattern may apply for the 5 verbs in 234.5 — OR a centralized `accept_holon_or_record` type-position helper.

### D5 — Runtime safety: no class_fqdn check inside VSA verbs

The 5 verbs don't validate class_fqdn — they're VSA primitives. A wrong-type record passed to `cosine` is semantically valid (cosine measures similarity between any two holons). Stone 234.2c (D10 of 234.2b sub-DESIGN) is the runtime class-safety for per-field accessors; that's a SEPARATE concern.

### D6 — No changes to the 234.2b macro

The macro at `wat/Record.wat` is correct as-shipped. Stone 234.5 is pure substrate extension — no macro touches.

### D7 — Stone 234.2b probe stays GREEN

The 234.2b probe doesn't exercise VSA verbs on records; it's macro behavior. Stone 234.5 must not regress that probe.

### D8 — HARD CUT — no aliases or escape hatches

Records flow through the 5 VSA verbs natively after 234.5. The pre-234.5 escape hatch (`(:wat::core::record->holon r)` explicit conversion) is not minted; users compose directly OR via `(:wat::holon::to-holon r)` (which now works on records per D2).

### D9 — Atomic-commit shape: single stone, single commit

234.5 is one cohesive change (5 verbs + helper + check.rs broadening). Ships as ONE atomic commit when 11/11 PASS. No intermediate broken states.

---

## Trap-door audit

### T1 — `Value::wat__Record` field access pattern

The variant has `class_fqdn: Arc<String>`, `struct_form: Arc<Vec<Value>>`, `holon_form: Arc<HolonAST>`. Unwrapping the holon_form needs `(*holon_form).clone()` OR `holon_form.as_ref().clone()` (NOT `Arc::try_unwrap` because the Arc may be shared).

Pattern proven at Stone 234.2a `eval_record_field_at` (line ~14579). Mirror that.

### T2 — `to_holon_inner` is the polymorphic UP body

Stone 225 renamed `value_to_atom` to `to_holon_inner`. It's the polymorphic dispatch for `:wat::holon::to-holon`. Adding a `Value::wat__Record` arm makes the bridge work uniformly.

### T3 — `pair_values_to_vectors` for cosine

`eval_algebra_cosine` at runtime.rs line 17153 calls `pair_values_to_vectors(":wat::holon::cosine", a, b, sym, list_span)`. That helper extracts vector representations from the Value pair. Sonnet investigates whether the helper already has a HolonAST-extraction path; if so, the wat__Record arm threads through there.

### T4 — `eval_algebra_bind` and `eval_algebra_bundle`

These constructors take HolonAST args. The arg-extraction site is where the wat__Record arm fires. Sonnet locates the exact extraction pattern + threads the D1 helper through.

### T5 — `eval_extract_classifier` reads from HolonAST shape

`:wat::holon::extract-classifier` returns the classifier of a Bind-typed HolonAST. When called on a record, the record's holon_form's outer Bind carries the class_fqdn. Auto-dispatch unwraps + the existing logic returns the correct classifier.

### T6 — check.rs TypeScheme registration sites

Each of the 5 verbs has a TypeScheme. Locate each via grep (`grep -n '":wat::holon::cosine"\|":wat::holon::Bind"\|":wat::holon::Bundle"\|":wat::holon::to-holon"\|":wat::holon::extract-classifier"' src/check.rs`). Broaden each to accept `:wat::Record` in HolonAST positions.

### T7 — Custom-handler precedent

Stone 234.2a-CORRECTION minted `infer_record_of`. Similar pattern may apply if the 5 verbs need custom handlers. OR — a single centralized helper accepting both types in HolonAST positions could replace multiple custom handlers.

### T8 — Lib baseline + regression guards

After the substrate changes, ALL existing tests must continue to PASS. The lib baseline at 827 stays unchanged. All arc 234 + 232.0a + 233 + 227 regression guards stay green.

---

## What the FM 2-bis probe must demonstrate

`tests/probe_arc234_stone5_holon_auto_dispatch.rs` — contracts (6):

1. **`to-holon` returns holon_form directly** — construct a record via the 234.2b macro; call `(:wat::holon::to-holon r)`; verify result equals the record's `holon_form` field (Rust-side Eq).
2. **`cosine` accepts two records** — construct two identical-class records via macro; call `(:wat::holon::cosine r1 r2)`; verify returns f64.
3. **`Bind` accepts a record as right arg** — `(:wat::holon::Bind classifier-h r)` builds a Bind composition with r's holon_form.
4. **`Bundle` accepts records as children** — `(:wat::holon::Bundle [r1 r2 r3])` builds a Bundle with each record's holon_form.
5. **`extract-classifier` returns class_fqdn from a record** — `(:wat::holon::extract-classifier r)` returns the record's class FQDN String (same as `:wat::core::type r`).
6. **Mixed args work** — `(:wat::holon::Bind classifier-h (:wat::holon::Bundle [r1 (:wat::holon::Atom classifier-h)]))` composes records + raw HolonASTs in the same expression.

**Initial state (before sonnet ships):** 6/6 FAIL with `TypeMismatch { expected: :wat::holon::HolonAST, got: :wat::Record }` (or similar; the type-checker currently rejects records in HolonAST positions).

**Post-stone:** 6/6 PASS. Records flow through VSA verbs natively.

---

## STOP triggers (rejection criteria)

- **STOP-1** — unexpected compile errors not tracing to the 5 verbs' updates
- **STOP-2** — lib tests baseline regresses below 827
- **STOP-3** — 120 min elapsed (hard cap; medium-sized stone)
- **STOP-4** — `holon-rs` touched
- **STOP-5** — Rust changes outside `src/runtime.rs` + `src/check.rs`
- **STOP-6** — scope creep: additional `:wat::holon::*` verbs beyond the 5 named (deferred to 234.6 migration sweep); class_fqdn checks inside VSA verbs (D5 explicitly rejects)
- **STOP-7** — the new probe doesn't flip 6/6 PASS
- **STOP-8** — Stone 234.2b probe regresses
- **STOP-9** — any prior arc 234 regression guard regresses (234.0, 234.1, 234.1.5, 234.2a)
- **STOP-10** — clippy warnings exceed 54

Each STOP is REJECTION criteria, not permission slot. If hit: report; surface; do NOT ship workaround.

---

## What this unblocks

- **Stone 234.6** — migration sweep + `:wat::holon::defrecord` retirement. The migrated callers depend on records being first-class VSA operands.
- **Stone 234.7** — INSCRIPTION.
- **The hologram is externally observable** — users can write `(:wat::holon::cosine r1 r2)` and get a meaningful result. The DESIGN's "no conversion call needed by the user" claim becomes truth.

---

## Calibration prediction

**Target runtime:** 60–90 min Mode A
**Upper bound:** 120 min (STOP-3 hard cap)
**Confidence:** medium — substrate change touches 5 verbs + helper + check.rs broadening; precedent established (Stone 234.2a-CORRECTION's `infer_record_of` pattern).

**Rationale:**
- Runtime: centralized helper (~15 lines) + 5 verb threadings (~30-50 lines) + `to_holon_inner` arm (~5 lines) = ~50-70 lines
- check.rs: 5 TypeScheme broadenings (custom handlers OR centralized accept-helper) = ~30-50 lines
- Probe already committed pre-spawn: no probe-authoring time
- Compile cycles: 2-4 rounds expected
- SCORE writing: ~10-15 min

**Calibration precedents:**
- Stone 234.2a-CORRECTION (~25 min): single custom handler in check.rs
- Stone 234.2a (~58 min): substrate primitives + check.rs TypeSchemes
- Stone 232.0a (~52 min): 3 verbs + check.rs special-cases
- Stone 234.5 estimate: ~70-90 min predicted; band's upper-middle (5 verbs is more than 3)

**Risks:**
- **T4 + T6 are independent** — runtime helper + check.rs broadening might require different patterns; sonnet picks each
- **Vec containing records as Bundle children (Probe 4)** — splicing records inside `[...]` for `Bundle` requires the check-time to allow `:wat::Record` as Vec element where HolonAST is expected
- **Mixed-args case (Probe 6)** — proves the composition works; if any verb's threading is incomplete, this fails

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (lines 115-120: user surface; line 290: VSA integration scope)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — predecessor SCORE (the macro consumers)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a-CORRECTION.md` — custom-handler precedent (`infer_record_of`)
- `src/runtime.rs::to_holon_inner` (line ~15198) — polymorphic UP body
- `src/runtime.rs::eval_algebra_bind` (line ~16286)
- `src/runtime.rs::eval_algebra_bundle` (line ~16341)
- `src/runtime.rs::eval_algebra_cosine` (line ~17153)
- `src/runtime.rs::eval_extract_classifier` — extract-classifier eval
- `src/check.rs` line 5616 — cosine TypeScheme registration area
- `tests/probe_arc234_stone5_holon_auto_dispatch.rs` — FM 2-bis probe (6 contracts; 6/6 FAIL initial)
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_no_known_defect_left_unfixed.md` — STOP triggers are rejection, not deferral
