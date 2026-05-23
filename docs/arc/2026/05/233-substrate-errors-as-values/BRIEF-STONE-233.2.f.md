# BRIEF — Arc 233 Stone 233.2.f — apply Tracked-unwrap defect fix

## What we're doing

Fix `eval_apply`'s head-value pattern matches (`src/runtime.rs:7433` + `src/runtime.rs:7438`) to use `Value::inner()` before matching. Currently both `match head_val` sites pattern-match on the OUTER variant — a `Value::Tracked { inner: Box::new(Value::wat__core__keyword(...)), .. }` wrapping (introduced by Stone 233.2.b/c producer tags) falls through to the type-mismatch arm despite the inner type matching exactly.

**The defect surfaced during Stone 233.2.d verification** — `probe_diagnostic_dynamic_keyword_invocation` probe_2 + probe_3 fail with `TypeMismatch { expected: "wat::core::keyword", got: ValueSnapshot { type_name: "wat::core::keyword", rendered: ":ns::greeting", provenance: RuntimeBuilt { producer: ":wat::core::keyword/from-string", ... } } }`. The dishonest signal is unmissable: `expected` and `got.type_name` MATCH yet TypeMismatch still fires.

After this stone: `probe_diagnostic_dynamic_keyword_invocation` flips from 6/8 → 8/8. Stone 233.2.a transparency contract (`Value::inner()` unwraps Tracked) becomes the load-bearing convention for any code that pattern-matches a Value to extract its underlying variant.

Per `feedback_any_defect_catastrophic` + `feedback_no_known_defect_left_unfixed`: defect observed → defect eliminated. No deferral.

## Design substrate (READ FIRST; MANDATORY)

1. **`src/runtime.rs:7352-7448`** — `fn eval_apply` body. The two defective sites:
   - **Line 7433:** `if let Value::wat__core__fn(ref func) = head_val { ... }` — fast path for fn-valued head; Tracked-wrapped fn falls through silently.
   - **Lines 7438-7448:** `let head_kw = match head_val { Value::wat__core__keyword(ref k) => k.clone(), ref other => Err(TypeMismatch { ... }) };` — keyword-head extraction; Tracked-wrapped keyword falls through to TypeMismatch arm.

2. **`src/runtime.rs`** — `impl Value { pub fn inner(&self) -> &Value }` (added in Stone 233.2.a at `7cfeff1`). Recursively unwraps `Value::Tracked` to return the underlying variant. Transparency contract is documented + tested in `tests/probe_value_tracked_transparency.rs`.

3. **`tests/probe_diagnostic_dynamic_keyword_invocation.rs`** — probe_2 + probe_3 are the load-bearing assertion. Pre-stone they FAIL (verified independently via stash round-trip during Stone 233.2.d verification). Post-stone target: 8/8 PASS.

4. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md`** — Stone 233.2.d surfaced this defect via Row 6 honest delta. Stone 233.2.f closes the gap.

## Implementation surface (the fix)

Two edits in `fn eval_apply` (`src/runtime.rs:7433` + `src/runtime.rs:7438`). Use `head_val.inner()` before pattern-matching:

```rust
// Step 5 — fast path: fn-valued head (Arc 009 lift OR let-bound fn).
//   BEFORE: if let Value::wat__core__fn(ref func) = head_val { ... }
//   AFTER:  if let Value::wat__core__fn(func) = head_val.inner() { ... }

// Step 6 — keyword-valued head: extract name + dispatch chain.
//   BEFORE: let head_kw = match head_val { Value::wat__core__keyword(ref k) => k.clone(), ref other => Err(...) };
//   AFTER:  let head_kw = match head_val.inner() { Value::wat__core__keyword(k) => k.clone(), other => Err(TypeMismatch { got: ValueSnapshot::of(other), ... }) };
```

Note the ownership shift: `head_val.inner()` returns `&Value`. Existing arms used `ref` patterns; updated arms drop `ref` since the match is on `&Value` directly. Sonnet handles the `&` and `ref` adjustments per the borrow checker's guidance.

`ValueSnapshot::of(other)` still constructs from a `&Value` reference (signature unchanged from Stone 233.1).

## Out of scope (affirmative scope-bounding)

- **Wider audit** — other sites in src/ that pattern-match `Value` without `Value::inner()` may have the same latent gap. Out of scope for this stone; file as discrete follow-up task (audit task #491) after ship.
- **Refactoring apply's body** beyond the two match sites — keep edits minimal.
- **Renaming `head_val`** or other identifiers — scope discipline.
- **AST-derived provenance** (Stone 233.2.e)
- **Errors-as-EDN** (Stone 233.3)
- **holon-rs** — NOT touched
- **HARD CUT** — no deprecation aliases

## Verification flow

```
cargo build --release -p wat 2>&1 | tail -5                                       # 0 errors
cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 | tail -3   # 8 passed; 0 failed
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                   # ≥ 827 passed
cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 | tail -3  # 1 passed
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3     # 8 passed
cargo test --release --test probe_value_tracked_transparency 2>&1 | tail -3              # 8 passed
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"       # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                                  # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** 30 min elapsed (tight stone)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54 (post-233.2.d baseline)
- **STOP-6:** scope creep — refactoring apply's body beyond the two sites OR adding `.inner()` calls elsewhere in the file
- **STOP-7:** probe_diagnostic_dynamic_keyword_invocation still has failing tests post-fix
- **STOP-8:** existing arc 233 probes (233.1 / 233.2.a / 233.2.c-substrate-symmetry) regress

If any STOP fires: ship NOTHING; surface as honest delta in SCORE.

## Trap-door audit

- **NO wider audit in this stone.** Find-other-sites is task #491; out of scope here.
- **NO body refactor.** Just `.inner()` insertions + the `ref` → `&` adjustment the borrow checker forces.
- **NO renaming.** `head_val` stays.
- **DO NOT touch holon-rs.** STOP-4.
- The borrow checker WILL force minor adjustments (removing `ref` from arm bindings since match is now on `&Value`). Sonnet follows the compiler's errors.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases
- Per `feedback_inscription_immutable`: SCORE is a new file
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md` — where this defect was surfaced (Row 6 honest delta)
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.md` — Shape C + transparency contracts (Stone 233.2.a)
- `tests/probe_value_tracked_transparency.rs` — Value::inner() transparency tests
- `tests/probe_diagnostic_dynamic_keyword_invocation.rs` probes 2 + 3 — load-bearing
- `feedback_any_defect_catastrophic` — substrate trust binary; defect observed → defect eliminated
- `feedback_no_known_defect_left_unfixed` — no deferral
- `feedback_sonnet_writes_substrate` — protocol
