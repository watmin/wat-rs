# Arc 170 slice 3 Gap I-A SCORE — mint `is_declaration_form` + unify prelude lift

**Date:** 2026-05-12
**Branch:** arc-170-program-entry-points
**Status:** COMPLETE — 2238 passed / 0 failed (+6 probes over Gap H baseline of 2232)

## Scorecard (6 rows)

| Row | What | Pass criterion | Result |
|-----|------|----------------|--------|
| A | `pub fn is_declaration_form` minted in `src/freeze.rs` adjacent to `is_mutation_form`; covers exactly the 8 declaration keywords (def, define, defmacro, define-dispatch, struct, enum, newtype, typealias); no loads, no config setters | grep + read | PASS — `pub fn is_declaration_form` at src/freeze.rs:1271; covers all 8 forms; excludes load-file!/digest-load!/signed-load! and config::set-* |
| B | `is_prelude_form` in `src/closure_extract.rs` retired (deleted, NOT kept as wrapper); `split_body_prelude` consumes `is_declaration_form` via head-keyword extraction | grep + read | PASS — `is_prelude_form` deleted and replaced by `head_keyword` helper; `split_body_prelude`'s `take_while` calls `crate::freeze::is_declaration_form` via `head_keyword` |
| C | 6+ probes in `tests/probe_declaration_form_lift.rs` pass (def / defmacro / define-dispatch / newtype / typealias / mixed) | cargo test | PASS — `6 passed; 0 failed` |
| D | All 5 Gap H probes (`probe_closure_body_prelude_lift`) still pass — regression confirms `is_declaration_form` covers define/struct/enum identically to retired `is_prelude_form` | cargo test | PASS — `5 passed; 0 failed` |
| E | All 11 prior substrate probes still pass: do_splice_def/define/struct/enum, let_splice_def/define/struct/enum, spawn_process_parent_type, resolver_quote_awareness, deftest_hermetic_isolation | cargo test | PASS — all 11 probe test files pass; no regressions |
| F | `cargo check --release` green; workspace at 2232 + N passed / 0 failed (N ≥ 6 new probes) | full test run | PASS — `2238 passed; 0 failed` (+6 probes over baseline of 2232) |

**All 6 rows PASS.**

---

## Files changed

| File | Change |
|------|--------|
| `src/freeze.rs` | Added `pub fn is_declaration_form` after `is_mutation_form` (lines 1271-1302). Docstring names both current caller (closure_extract::split_body_prelude) and future caller (check::validate_def_position_with_wrapper, Gap I-B). Explains `defn` absence (macro-expansion precedence). |
| `src/closure_extract.rs` | (1) Retired `is_prelude_form` (lines 1762-1775 replaced by `head_keyword` helper). (2) Added `walk_defmacro_form` companion walker (Gap I-A companion bug fix — same class as Gap H's Delta 1 for struct/enum). (3) Added `":wat::core::defmacro"` dispatch arm in `walk_free_symbols`'s List arm, routing to `walk_defmacro_form`. (4) Updated `split_body_prelude`'s `take_while` to use `head_keyword` + `crate::freeze::is_declaration_form`. (5) Updated docstring: "prelude forms" → "declaration forms"; references `is_declaration_form` predicate. |
| `tests/probe_declaration_form_lift.rs` | CREATED — 6 Gap I-A probes (is_declaration_form unit test / defmacro / define-dispatch / newtype / typealias / mixed-7-forms). |
| `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-I-A-IS-DECLARATION-FORM.md` | CREATED — this file. |

---

## Workspace delta

- Baseline (post-Gap-H, commit `36030c3`): 2232 passed / 0 failed
- Post-Gap-I-A: 2238 passed / 0 failed (+6 probes)

---

## Head-keyword extraction choice rationale

The lift-site change in `split_body_prelude` uses a **factored `head_keyword` helper** rather than an inline closure:

```rust
fn head_keyword(node: &WatAST) -> Option<&str> {
    if let WatAST::List(items, _) = node {
        if let Some(WatAST::Keyword(k, _)) = items.first() {
            return Some(k.as_str());
        }
    }
    None
}
```

Rationale: The "extract head keyword from a WatAST node" pattern recurs throughout `walk_free_symbols` and `split_body_prelude`. A factored helper:
- Eliminates the nested `if let` inline in the `take_while` closure (reducing visual noise)
- Names the pattern at a single site (future callers in the same file can reuse it)
- Makes the `take_while` closure read as a clear single-expression `.map(fn).unwrap_or(false)` pipeline

The inline closure would have been equivalent but noisier. Four questions:
- **Obvious**: `head_keyword` is an obvious name for "extract keyword from list head."
- **Simple**: one nested match, returns `Option<&str>`.
- **Honest**: it is exactly what it does, no more.
- **Good UX**: call sites read `head_keyword(child).map(crate::freeze::is_declaration_form).unwrap_or(false)`.

---

## `is_prelude_form` retirement strategy rationale

`is_prelude_form` was **deleted entirely** (not kept as a wrapper). Rationale:

The BRIEF's doctrine: "delete (one source-of-truth; no aliases)." Keeping `is_prelude_form` as a thin wrapper over `is_declaration_form` would:
- Create a dead alias that future readers would need to follow
- Imply the two predicates are semantically distinct (they wouldn't be)
- Delay the name's full retirement (it would keep appearing in `grep` results)

The `split_body_prelude` call site now directly uses `head_keyword` + `is_declaration_form`. The Gap H docstring in `split_body_prelude` was updated to reference `is_declaration_form` explicitly. No callers of `is_prelude_form` existed outside `split_body_prelude`, so deletion was zero-blast-radius.

---

## Probe-shape rationale

**Positive-case-only** probes, consistent with Gap H's precedent. The failing-baseline is:
- Implicit for the 4 forms that the check-validator passes (defmacro / define-dispatch / newtype / typealias): they would reach child runtime and trigger `EvalForbidsMutationForm` or be silently ignored before Gap I-A's lift.
- Documented in probe 1's docstring for `def`: the failing baseline for `def` is at PARENT freeze time (check-time validator blocks it before the lift can run).

Gap H's 5 probes prove the lift mechanism; Gap I-A's probes prove additional coverage for the 5 newly-registered forms. Shipping before/after pairs would double the probe count without proportional information gain.

---

## `defn`-absence docstring decision

**Yes, document it.** `is_declaration_form`'s docstring includes:

> `` `defn` is intentionally absent: it is a macro that expands to `(:wat::core::def ...)` BEFORE `extract_closure` runs. By the time the prelude-lift or position-validator consults this predicate, `defn` has already been rewritten to its `def` form; `def` is covered here. ``

Rationale: future readers WILL ask "where is `defn`?" The substrate already has this pattern documented in `src/runtime.rs:2369-2387` (`try_parse_fn_shape_def` docstring). Repeating the explanation at the predicate site closes the gap between the predicate's form-list and the macro-expansion pipeline. Four questions:
- **Obvious**: readers familiar with Clojure will expect `defn` here.
- **Honest**: the explanation is true, not a rationalization.
- **Good UX**: caller never needs to grep `runtime.rs` to understand the absence.

---

## Honest deltas

### Delta 1 — `def` at fn body do-prefix blocked at PARENT check time (not child runtime)

The BRIEF stated: "Pre-Gap-I-A: child fails with `DefNotTopLevel`." This is incorrect for `def`. The failure happens at the PARENT's `startup_from_source` during step 8 (`check_program` → `validate_def_position_with_wrapper`), which emits `DefNotTopLevel` and halts the parent freeze — before `extract_closure` ever runs.

Impact: the end-to-end spawn probe for `def` at fn body do-prefix CANNOT pass under Gap I-A alone. The check-time validator must be extended (Gap I-B) to understand that a `def` at a fn body's `do`-prefix is safe because `split_body_prelude` will lift it before the body is evaluated.

Response: Probe 1 was redesigned as a direct unit test of `is_declaration_form` covering all 8 keywords and verifying exclusion of loads/config-setters/defn. This verifies Gap I-A's core contribution (the predicate) while being honest about the check-time barrier. The probe docstring explains the Gap I-B dependency explicitly. The mixed probe (Probe 6) covers 7 of 8 forms (excluding `def`), documenting why `def` is absent.

Gap I-B is the explicit follow-on slice. This delta is an honest scope discovery, not a deferral — the Gap I-B BRIEF already exists as a named step.

### Delta 2 — `walk_defmacro_form` companion walker required (same class as Gap H Delta 1)

`is_prelude_form`'s 3-form match never included `defmacro`, so `walk_free_symbols` never encountered `defmacro` in the do-prefix. After routing through `is_declaration_form`, the lift now includes `defmacro` — which caused `walk_free_symbols` to fall through to "Plain list — recurse on every child." The defmacro body contains bare Symbol references (macro parameter names like `x` in `~x`). These were misclassified as free variable references, causing `MalformedForm: free symbol 'x' does not resolve`.

Fix: added `walk_defmacro_form` (a no-op that returns `Ok(())`) and dispatched from `walk_free_symbols`'s List arm. The defmacro body is a macro template; its parameter Symbols are binding positions, not free variable references to the parent scope. Skipping the defmacro body entirely is correct.

This is the same companion-bug class as Gap H's Delta 1 (struct/enum needed `walk_struct_form`/`walk_enum_form`). The pattern: each newly-lifted form that contains Symbol-position children needs a dispatch arm in `walk_free_symbols` that treats those children correctly. `defmacro`'s parameters are bindings; skip the body. `define-dispatch`'s arm impl names are Keywords (not Symbols); the plain-list walk is safe. `newtype`/`typealias` are Keyword-only forms; safe. `def` is blocked at the check-time validator and never reaches `walk_free_symbols` in the affected parent path.

### Delta 3 — `define-dispatch` works without a companion walker

The initial run confirmed that `define-dispatch` (and `newtype`/`typealias`) work correctly with the plain-list walk in `walk_free_symbols`. Analysis: `define-dispatch` arm references are Keywords (`:impl-fn-name`), not bare Symbols — so the Symbol arm of `walk_free_symbols` never fires for them. `newtype`/`typealias` are purely Keyword forms (`:TypeName :InnerType`/`:AliasName :TargetType`). No companion walkers needed for these 3 forms.

This confirms that the companion-walker requirement is NOT universal to all newly-covered declaration forms — only to forms that have bare Symbol children in binding positions. Gap H covered this for struct/enum; Gap I-A covers it for defmacro. The `def` form's body (the RHS expression) would need careful analysis if the check-time validator were relaxed — but that analysis belongs to Gap I-B.

### Delta 4 — `head_keyword` helper confirms pattern needs centralizing

The `head_keyword` function implemented here ("extract Keyword from the head of a WatAST::List") already exists in spirit at several call sites in `walk_free_symbols` (the dispatch arms all do the same nested `if let` pattern). A future refactoring arc could consolidate those sites to use `head_keyword`. Gap I-A introduces the helper at `split_body_prelude`'s site; the broader centralization is out of scope here but surfaces as a structural observation.

---

## Verification commands run

```
# New Gap I-A probes
cargo test --release --test probe_declaration_form_lift
# → 6 passed; 0 failed

# Gap H regression (CRITICAL)
cargo test --release --test probe_closure_body_prelude_lift
# → 5 passed; 0 failed

# All 11 prior substrate probes
cargo test --release \
  --test probe_do_splice_def --test probe_let_splice_def \
  --test probe_do_splice_define --test probe_let_splice_define \
  --test probe_do_splice_struct --test probe_do_splice_enum \
  --test probe_let_splice_struct --test probe_let_splice_enum \
  --test probe_spawn_process_parent_type \
  --test probe_resolver_quote_awareness \
  --test probe_deftest_hermetic_isolation
# → all 11 files pass; no regressions

# Workspace
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | \
    awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# → passed:2238 failed:0
```

---

## Cross-references

- `36030c3` Gap H (the precedent slice; `is_prelude_form` + lift mechanism origin; companion walker pattern)
- `021884a` Gap G (the blockage that surfaced Gap H; `DefineInExpressionPosition` analysis)
- `fe06bb1` Gap F-3 (extract_closure extension precedent; type-registry sweep adjacent to I-A's predicate routing)
- After Gap I-A: Gap I-B is the next slice — extend `validate_def_position_with_wrapper` to cover all 8 declaration forms via `is_declaration_form`. Gap I-B's BRIEF can reference this SCORE's Delta 1 as the authoritative explanation of why the check-time validator must be extended before `def` at fn body do-prefix works end-to-end.
- `src/freeze.rs` — modified (is_declaration_form addition)
- `src/closure_extract.rs` — modified (is_prelude_form retired; head_keyword helper; walk_defmacro_form companion; split_body_prelude updated)
- `tests/probe_declaration_form_lift.rs` — new probe file (6 probes)
