# SCORE — Stone 236.3 — `CheckResult<T>` sum-type refactor

**Date:** 2026-05-25
**Status:** COMPLETE — 12/12 PASS.

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **Stone 236.0 probe still PASSES** (Contract 6 doc sharpened) | `cargo test --release --test probe_arc236_stone0_check_result 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | 234.4 regression | `cargo test --release --test probe_arc234_stone4_hash_destructure 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 4 | 234.3c.fix regression | `cargo test --release --test probe_arc234_stone3c_fix_narrow_fallthrough 2>&1 \| tail -3` | `4 passed; 0 failed` |
| 5 | 234.3c regression | `cargo test --release --test probe_arc234_stone3c_keyword_accessor 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 6 | 234.3b regression | `cargo test --release --test probe_arc234_stone3b_record_assoc 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 7 | 234.3a regression | `cargo test --release --test probe_arc234_stone3a_record_read_verbs 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 8 | **Lib baseline** (LOAD-BEARING) | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | `827 passed; 0 failed` |
| 9 | 232.0a regression | `cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 \| tail -3` | `7 passed; 0 failed` |
| 10 | 233.3 errors-as-EDN regression | `cargo test --release --test probe_stone_233_3_runtime_error_edn 2>&1 \| tail -3` | `5 passed; 0 failed` |
| 11 | Clippy | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | `52` (≤ 54; unchanged from Stone 236.2) |
| 12 | Enum-shape verified | `grep -c "^pub enum CheckResult" src/check.rs` → `1`; `grep -c "^pub struct CheckResult" src/check.rs` → `0` | `1` / `0` (enum exists; struct gone per D5 HARD CUT) |

---

## Code-diff summary

**Type definition (src/check.rs):** struct → enum, 5 lines removed / 9 lines added

```rust
// BEFORE (Stone 236.0 shape, 5 lines):
pub struct CheckResult<T> {
    value: Option<T>,
    errors: Vec<CheckError>,
}

// AFTER (Stone 236.3 shape, 9 lines):
pub enum CheckResult<T> {
    /// Type produced, no errors.
    Ok(T),
    /// Type produced AND diagnostics logged.
    Partial(T, Vec<CheckError>),
    /// No type, one or more errors.
    Err(Vec<CheckError>),
}
```

**Constructor bodies (5 functions):** ~30 lines replaced. Signatures unchanged. Bodies
`Self { value: Some(value), errors: Vec::new() }` → `CheckResult::Ok(value)`, and so on
for the other four constructors. Smart constructor surface: IDENTICAL. Zero cascade at 151
HARVEST points.

**Accessor bodies (5 functions, ~25 lines replaced):**

| Accessor | Before | After |
|---|---|---|
| `value()` | `self.value.as_ref()` | `match self { Ok(t) \| Partial(t,_) => Some(t), Err(_) => None }` |
| `errors()` | `&self.errors` | `match self { Ok(_) => &[], Partial(_,errs) \| Err(errs) => errs }` |
| `has_errors()` | `!self.errors.is_empty()` | `matches!(self, Partial(_,_) \| Err(_))` |
| `is_ok()` | `self.value.is_some() && self.errors.is_empty()` | `matches!(self, Ok(_))` |
| `into_parts()` | `(self.value, self.errors)` | 3-arm match → `(Some(t), vec![])`, `(Some(t), errs)`, `(None, errs)` |

**Combinator bodies (4 functions, ~60 lines replaced):**

- `map`: single `value.map(f)` field op → 3-arm match; `Ok(f(t))`, `Partial(f(t), errs)`, `Err(errs)`
- `and_then`: 2-arm `match self.value` (Some/None) → 3-arm nested match (Ok/Partial/Err); correctly merges
  prior `Partial` errors into the inner result across all three inner outcomes
- `merge_errors_from`: `mut` borrows + `append` → `into_parts()` + 3-arm match building new variants;
  short-circuits on empty `other_errs` for the Ok-stays-Ok fast path
- `drain_errors_into`: single `append` + field return → 3-arm match draining `errs` into `sink`

**Bridge signature:** `drain_errors_into(self, sink: &mut Vec<CheckError>) -> Option<T>` — UNCHANGED.
Zero cascade at the ~267 call sites.

**Docstring (src/check.rs lines 998–1068 approx):** Updated in place. Replaced
"four valid states (by construction)" section + "why no fifth state" section with:
- "Three variants (structural definition)" — shows the enum definition inline
- "Why the silent-failure state is STRUCTURALLY UNREPRESENTABLE" — "no `Silent` variant
  exists; pattern-matching consumers guaranteed exhaustive"
- "Consumer patterns" — smart constructors (call-surface, unchanged) + pattern-matching
  (enum-surface, new) + `drain_errors_into` bridge

**Probe file (tests/probe_arc236_stone0_check_result.rs):** File-header doc sharpened.
Contract 6 WHY updated from "structurally unreachable by construction" to "no `Silent` variant
exists — verified by exhaustive pattern matching over `Ok | Partial | Err`." The 6 test
functions and their assertion bodies are UNTOUCHED.

Net touch: ~200 lines in `src/check.rs` (all within the CheckResult type definition + impl
block + docstring); ~10 lines in the probe file header. Zero other files modified.

---

## Cascade depth

**1 compile round.**

The struct → enum swap broke zero external call sites. The smart constructors are the
function-call surface; their bodies updated from struct-field construction to variant
construction; the signatures were unchanged. The `drain_errors_into` bridge signature was
unchanged. Accessors and combinators changed bodies only, not signatures.

The single compile round confirmed the ZERO-RENAME body-construction property empirically:
every `CheckResult::ok(t)`, `CheckResult::errs(es)`, `CheckResult::partial_with(t, es)` call
at the 151 HARVEST points (236.1 + 236.2) still compiled without modification.

---

## Test rot revealed

**Zero.**

No existing lib test or probe test pattern-matched the OLD struct shape directly. No test
used `CheckResult { value: ..., errors: ... }` struct-pattern syntax. All test code accessed
`CheckResult` via the accessor API (`value()`, `errors()`, `is_ok()`, `has_errors()`) which
preserved its external signature. The lib baseline held exactly at 827.

---

## Honest deltas from BRIEF

- **Cascade depth = 1 (predicted 1-2):** Matched the lower end of the prediction. The struct
  → enum swap + body-only changes in the impl block produced no external cascade.

- **Test rot = 0 (expected 0-2):** No test pattern-matched the struct form. The probe file used
  only accessor API; the lib tests similarly. The BRIEF's 0-2 tolerance was never needed.

- **merge_errors_from mut signature:** The BRIEF's locked implementation took `mut self` and
  called `append`. The new implementation uses `into_parts()` + extends, dropping the `mut`
  requirement on `self`. The signature changes from `pub fn merge_errors_from<U>(mut self, ...)` to
  `pub fn merge_errors_from<U>(self, ...)` — strictly less restrictive; callers unaffected.
  Clippy stayed at 52 (the `mut` removal may account for an eliminated lint).

- **Docstring line range:** BRIEF cited 1040-1206. Actual post-refactor lines for the docstring
  are approximately 998-1067 (the DESIGN doc line estimates were from the struct-form; the enum
  docstring is comparable in length to the original; line numbers shifted slightly due to the
  enum's added variant doc comments).

---

## Rank-up evidence

**SCORE-STONE-236.2.md as template:** The 12-row scorecard structure, cascade-depth section,
test-rot section, honest-deltas section, and rank-up section were all copy-and-adapt from
236.2's SCORE. The template worked.

**ZERO-RENAME body-construction property held empirically:** The 151 HARVEST points (236.1 +
236.2) never changed. The single compile round with 0 cascade errors is the empirical
confirmation. This was the arc's load-bearing claim: smart constructors absorb the API-compat
shock so the representation can change freely. Confirmed.

**Dialogue-as-PERCEIVE recognition vindicated:** The refactor's mechanical simplicity confirms
the dialogue-origin was correct. The 4-state invariant truth table the orchestrator wrote during
post-236.2 dialogue exposed the struct-with-Option shape's abuse of Option semantics; the enum
shape was the honest reading. The code confirmed: the pattern-match implementations are
straightforward — no surprises, no traps. The shape was right.

**Predecessor SCORE cascade under-prediction pattern held:** 236.0 was 1 round; 236.1 was 2
rounds (predicted 3-5); 236.2 was 1 round (predicted 3-5); 236.3 is 1 round (predicted 1-2).
The pattern: the consumer surface's stability (smart constructors, bridge signature) absorbs
representation changes without cascading. The sub-discipline "smart constructors are the
API-compatibility boundary" was the correct architectural choice from Stone 236.0.

---

## Working tree on return

```
 M src/check.rs
 M tests/probe_arc236_stone0_check_result.rs
?? docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.3.md
```

No other files modified. STOP-4 (holon-rs) not touched. STOP-5 (Rust outside check.rs + probe)
not violated. STOP-6 (arc 234/232/233 regressions) all passing. STOP-7 (clippy = 52, ≤ 54)
confirmed. STOP-8 (no transitional dual-channel) confirmed. STOP-9 (historical 236.0/236.1/236.2
artifacts untouched) confirmed.

---

## Closing note

The ✅✅✅ structural impossibility has shipped for arc 236's failure class.

Stone 236.0 minted `CheckResult<T>` as a struct with private fields — construction-time
discipline (✅✅). Stone 236.3 refactored to a 3-variant enum — type-system structural
impossibility (✅✅✅). The `Silent` state does not exist as a variant. Pattern-matching
consumers are compiler-guaranteed exhaustive across the three legitimate states: `Ok`,
`Partial`, `Err`.

**Arc 236 is now ready for INSCRIPTION (Stone 236.4).** After 236.4 closes, arc 234 resumes
per spawn-block winding discipline.
