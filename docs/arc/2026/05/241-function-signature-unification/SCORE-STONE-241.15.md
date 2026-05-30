# SCORE — Stone 241.15: ZOMBIE PURGE — `:wat::core::try` + `:wat::core::option::expect` + `:wat::core::result::expect` HARD CUT

**Mode:** A (substrate + cascade; vigilia NOT required — no new namespaced home)
**Runtime:** single session (under-band; apparatus mature from 241.13/14 pattern)
**Cascade size:** 4 src files modified; 4 doc files migrated
**Lib tests:** 890 / 0
**Workspace test build:** clean
**Clippy:** 889 warnings (within ≤930 gate)
**Vigilia:** NOT CAST (legacy flat substrate; no new namespaced home)
**Auto-fixer:** NOT minted (cascade was mechanical; sed + targeted edits)

---

## Phase A Scorecard (11 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | Probe C01 PASS (:wat::core::try HARD-CUT-rejected with Stone 241.15 marker) | PASS | `contract_01_try_hard_cut_rejected` |
| 2 | Probe C02 PASS (C01 rejection remedy names :wat::core::Result/try) | PASS | `contract_02_try_rejection_remedy_names_result_try` |
| 3 | Probe C03 PASS (:wat::core::option::expect HARD-CUT-rejected) | PASS | `contract_03_option_expect_lowercase_hard_cut_rejected` |
| 4 | Probe C04 PASS (C03 remedy names :wat::core::Option/expect PascalCase) | PASS | `contract_04_option_expect_lowercase_rejection_remedy_names_pascal` |
| 5 | Probe C05 PASS (:wat::core::result::expect HARD-CUT-rejected) | PASS | `contract_05_result_expect_lowercase_hard_cut_rejected` |
| 6 | Probe C06 PASS (C05 remedy names :wat::core::Result/expect PascalCase) | PASS | `contract_06_result_expect_lowercase_rejection_remedy_names_pascal` |
| 7 | Probe whole-suite 6/6 | PASS | `probe_arc241_stone15_zombie_purge` |
| 8 | Stone 241.14 probe preserved 6/6 | PASS | `probe_arc241_stone14_restricted_absorbed` |
| 9 | Stone 241.13 probe preserved 2/2 | PASS | `probe_arc241_stone13_define_dispatch_hard_cut` |
| 10 | Lib baseline ≥ 890 PASS / 0 FAIL | PASS | 890 / 0 |
| 11 | Workspace test-build clean | PASS | `cargo build --tests --workspace` exit 0 |

---

## Structural Verification (10 rows)

| Verification | Result |
|---|---|
| 3 HARD-CUT-rejection arms added to `src/check.rs` (infer_list expression dispatch path) | confirmed; at lines ~5798-5825 (Zombie A/B/C) |
| Soft-deprecation dispatch arms (arc 109 Pattern 2 poison) at check.rs:5862-5916 DELETED | confirmed; replaced by HARD-CUT arms |
| `arc_109_try_verb_migration_hint` fn DELETED | confirmed; replaced by retirement comment |
| `arc_109_option_expect_migration_hint` fn DELETED | confirmed; same |
| `arc_109_result_expect_migration_hint` fn DELETED | confirmed; same |
| 3 callers removed from `collect_hints` array | confirmed; comment noting Stone 241.15 deletion added |
| Routing helpers (validate_comm_positions + collect_consumed_names_in_let) surgical-remove of retired-head clauses | confirmed; `|| head_str == ":wat::core::option::expect"` and `result::expect` clauses removed from both functions |
| 3 retired-form dispatch arms DELETED from `src/runtime.rs` | confirmed; comment explaining Stone 241.15 HARD CUT replaces the 3 arms |
| 3 registry entries DELETED from `src/special_forms.rs` + test reference updated | confirmed; comment block replaced with Stone 241.15 note; `:wat::core::try` removed from `registry_covers_audited_forms` spot-check |
| 10th/11th/12th RETIREMENT_TABLE entries verbatim | `(":wat::core::try", ":wat::core::Result/try")` + `(":wat::core::option::expect", ":wat::core::Option/expect")` + `(":wat::core::result::expect", ":wat::core::Result/expect")` |

---

## HARD-CUT Arms (check.rs — infer_list expression dispatch)

```rust
// Stone 241.15 — Zombie A: :wat::core::try HARD CUT.
":wat::core::try" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.15); use ':wat::core::Result/try' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}

// Stone 241.15 — Zombie B: :wat::core::option::expect HARD CUT.
":wat::core::option::expect" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.15); use ':wat::core::Option/expect' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}

// Stone 241.15 — Zombie C: :wat::core::result::expect HARD CUT.
":wat::core::result::expect" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.15); use ':wat::core::Result/expect' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

---

## RETIREMENT_TABLE post-stone (12 entries)

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    (":wat::core::enum",              ":wat::core::defenum"),
    (":wat::core::define",            ":wat::core::defn"),
    (":wat::core::Char",              ":wat::core::char"),
    (":wat::runtime::define-alias",   ":wat::core::defalias"),
    (":wat::core::define-dispatch",   ":wat::core::defclause"),
    (":wat::core::def-restricted",    ":wat::core::def"),
    (":wat::core::defn-restricted",   ":wat::core::defn"),
    // Stone 241.15 — zombie purge: arc-109-slice-1j retirements now HARD CUT.
    (":wat::core::try",               ":wat::core::Result/try"),
    (":wat::core::option::expect",    ":wat::core::Option/expect"),
    (":wat::core::result::expect",    ":wat::core::Result/expect"),
];
```

---

## Cascade Audit

### S1+S5 — HARD-CUT-rejection arms + dispatch arm deletion (check.rs)

**`src/check.rs`:**
- Three soft-deprecation dispatch arms (arc 109 Pattern 2 poison at lines 5862-5916) DELETED
- Three HARD-CUT-rejection arms (MalformedForm, Stone 241.15 marker, retirement remedy) inserted at same expression-dispatch position
- `arc_109_try_verb_migration_hint`, `arc_109_option_expect_migration_hint`, `arc_109_result_expect_migration_hint` functions DELETED (~63 lines)
- Three caller entries removed from `collect_hints` array + Stone 241.15 comment added
- `validate_comm_positions`: two `|| head_str == ":wat::core::result::expect"` / `option::expect` clauses removed (comments updated)
- `collect_consumed_names_in_let`: two retired-head `|| head_str ==` clauses removed (doc comment + inline comment updated)

### S2 — RETIREMENT_TABLE (retirement.rs)

- `src/remedy/retirement.rs`: 10th/11th/12th entries added; arc history table updated with Stone 241.15 rows

### S3 — Dispatch arm deletion (runtime.rs)

**`src/runtime.rs`:**
- 3 dispatch arms deleted: `":wat::core::try" => eval_try(...)`, `":wat::core::option::expect" => eval_option_expect(...)`, `":wat::core::result::expect" => eval_result_expect(...)`
- Comment block updated to explain Stone 241.15 HARD CUT (canonical forms remain; eval functions unchanged)

### S4 — Soft-deprecation helper functions deleted (check.rs)

- `arc_109_try_verb_migration_hint` (~13 lines): DELETED
- `arc_109_option_expect_migration_hint` (~13 lines): DELETED
- `arc_109_result_expect_migration_hint` (~13 lines): DELETED
- Stone 241.15 comment marks the deletion site

### S6 — special_forms.rs registry entries deleted

- Entire "RETIRED (arc 109 § D' Pattern 2 poison)" comment block + 3 `insert(...)` calls DELETED
- Replacement comment: Stone 241.15 note explaining HARD CUT forms are not reflected
- `registry_covers_audited_forms` test: `:wat::core::try` spot-check entry removed (Stone 241.15 comment left in its place)

### S7 — Doc cascade (4 files)

| File | Migrations |
|---|---|
| `docs/USER-GUIDE.md` | 6 `:wat::core::try` → `:wat::core::Result/try`; 2+ `:wat::core::option::expect` → `:wat::core::Option/expect`; 10+ `:wat::core::result::expect` → `:wat::core::Result/expect` |
| `docs/SERVICE-PROGRAMS.md` | 11 `:wat::core::option::expect` → `:wat::core::Option/expect` |
| `docs/CLOJURE-ROSETTA.md` | 1 `:wat::core::try expr` → `:wat::core::Result/try expr` in Rosetta-stone table |
| `docs/WAT-CHEATSHEET.md` | 2 entries (lines 217-218): `result::expect` → `Result/expect`; `option::expect` → `Option/expect` |

### S8 — Reflection emitter audit

```
grep -rn "Keyword.*::try\b\|Keyword.*::option::expect\|Keyword.*::result::expect" src/
```

**0 AST-construction sites emitting retired forms.** Gate CLEAN.

---

## Pre-INSCRIPTION Grep Gate

All three forms checked:

```
grep -rn ":wat::core::try\b" src/ tests/ wat/
grep -rn ":wat::core::option::expect\b" src/ tests/ wat/
grep -rn ":wat::core::result::expect\b" src/ tests/ wat/
```

| Category | Status |
|---|---|
| HARD-CUT arms in `src/check.rs` (3) | REQUIRED — the retirement arms themselves |
| RETIREMENT_TABLE entries in `src/remedy/retirement.rs` (3) | REQUIRED — table entries drive remedy |
| Stone probe fixture in `tests/probe_arc241_stone15_zombie_purge.rs` | ACCEPTABLE — tests the HARD CUT |
| Historical/docstring comments in `src/check.rs`, `src/runtime.rs`, `src/special_forms.rs` | ACCEPTABLE — historical context |

**Active substrate callers: 0**

Gate CLEAN for all 3 forms.

---

## Honest Deltas

### HARD-CUT arm placement: expression-dispatch path, not top-level declaration path

Prior Stone 241.14 HARD-CUT arms landed in the top-level declaration processing branch (check.rs:5676). The zombie forms (`:wat::core::try`, option/result expect) are EXPRESSION forms used inside function bodies — not declaration forms. Their dispatch arms were in the `infer_list` expression path (formerly ~5862-5916). The Stone 241.15 HARD-CUT arms landed in the same expression-dispatch location, replacing the soft-deprecation arms directly. Correct placement confirmed by probe 6/6 PASS.

### Routing helpers: 4 sites total, not 2

The BRIEF specified `check.rs:2703-2734` and `2823-2839` (the `validate_comm_positions` function). In practice `collect_consumed_names_in_let` (a sibling walker) also had the same dual-head clauses at lines ~2825 and ~2840. All 4 sites cleaned. Honest delta: the BRIEF count was 2 target areas; 4 actual clause deletions across 2 functions (validate_comm_positions + collect_consumed_names_in_let). Both cleaned for totality per `feedback_hard_cut_admits_no_bypasses`.

### Clippy down to 889 (from 905 at Stone 241.13 baseline)

~16 warnings removed by deleting the three soft-deprecation helper functions (~63 lines) and removing their callers. Downward delta is healthy.

### Eval/infer functions CONFIRMED UNCHANGED

`eval_try`, `eval_option_expect`, `eval_result_expect`, `infer_try`, `infer_option_expect`, `infer_result_expect` — all untouched. Canonical dispatch arms (`Result/try`, `Option/expect`, `Result/expect`) confirmed preserved. T1 trap-door: no damage.

---

## Calibration

| Phase | Predicted | Actual |
|---|---|---|
| S1+S5 HARD-CUT arms + soft-deprecation dispatch deletion | 25-30 min | ~20 min |
| S2 RETIREMENT_TABLE | 5 min | ~3 min |
| S3 runtime.rs dispatch arms | 5 min | ~3 min |
| S4 soft-deprecation helper fn deletion | 10 min | ~5 min |
| S6 special_forms.rs | 5 min | ~3 min |
| S7 doc cascade (4 files) | 20-30 min | ~15 min (bulk sed pattern) |
| S8 reflection emitter audit | 5 min | ~2 min (zero hits) |
| S10 pre-INSCRIPTION grep gate | 10 min | ~5 min |
| S11 SCORE | 10-15 min | ~15 min |
| **Total** | **60-120 min** | **~71 min** |

Under-band; consistent with Stone 241.13 (25 min) + 241.14 (26 min reported; apparatus mature).

---

## What This Unblocks

**Stone 241.16** — Enemy 3 (`:wat::core::define` eval-time residue completion; closes Stone 241.11's partial HARD CUT). Board is clean; no zombie distractions. Gets focused attention.

**Stone 241.17** — INSCRIPTION closes arc 241.

**Arc 237.8b** — reopens after Stone 241.17 per `feedback_no_regression_until_arc_done`.

**One-canonical-path doctrine** — three more violations annihilated:
- `:wat::core::Result/try` is THE try propagation form
- `:wat::core::Option/expect` is THE option-panic form
- `:wat::core::Result/expect` is THE result-panic form
PascalCase Type/method canonical. Lowercase-namespace duplicates DEAD.
