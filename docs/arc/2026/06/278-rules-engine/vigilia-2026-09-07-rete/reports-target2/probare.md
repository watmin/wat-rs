## PROBARE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

SCOPE: 20 Rust files (21,523 lines) + 5 wat files (2,363 lines). `src/rete/kernel/**` untouched.

Commands run this session (all read-only): `wc -l`, per-file `grep -c` for blank/comment/code classification; `grep -n "rune:"` and `grep -n "⛔"` across scope; `grep -noE` sweeps for numeric caller/count claims; `grep -rn "<fn>(" src/ --include=*.rs` to re-derive every caller count cited below; `cat tests/lint/rete_header_claims_are_asserted.rs` and `grep -rln "<fn>" tests/` to check gate coverage; form counts on the `.wat` files to confirm real forms, not stubs.

## Claim-falsifiability analysis (the sharp surface)

Two ungated, checkable, load-bearing header claims are **false today**, in the same shape as the two already rowed this cast:

**1. `src/rete/matcher.rs:114-131`, `enum_variant_ctor` — claims "THREE independent sites" / "the three callers"; actual count is four.**
The doc names purity's `constructor_meta`, `expr_ir/mod.rs`'s lowerer, and `validate/mod.rs`'s `walk_nested_constructors` as the three callers whose needs justify the `(enum, variant, arity)` return shape. Re-derived via `grep -rn "enum_variant_ctor(" src/ --include=*.rs`:
- `src/rete/purity.rs:976`
- `src/rete/validate/typing.rs:335` (`classify_keyword_constant`, **NOT named in the doc's three**)
- `src/rete/validate/mod.rs:1056`
- `src/rete/expr_ir/mod.rs:1087`

Four call sites, four purposes (the fourth resolves `Unit`/`Tagged` for a diagnostic classification, distinct from the arity-check purpose the doc attributes to "the validator"). No gate references `enum_variant_ctor` by call-count. This likely postdates the `validate.rs` → `mod.rs`/`typing.rs`/`error.rs` split (`partire`'s 2026-08-30 cut, noted in both files' headers) — the split added a caller in the new `typing.rs` that the still-"three" doc in `matcher.rs` never absorbed.

**2. `src/rete/validate/mod.rs:60-72`, `reorder_kwargs_by_field_name` — claims "both callers" (two); actual production call-site count is three.**
The doc says: `eval_kwargs_construct` (runtime.rs) — enforced via `construct_aggregate`'s arity check *plus* `check.rs`'s `infer_kwargs_construct_check` — and `validate_rule_when_and_reorder_then`. Re-derived, excluding the two `#[cfg(test)]` sites inside `mod.rs` itself (`:1664`, `:1675`):
- `src/runtime.rs:18965` (`eval_kwargs_construct`)
- `src/check.rs:13115` — read at `:13095-13118`: a **direct, independent call**, not merely "enforcement" of the runtime.rs caller's arity as the doc frames it
- `src/rete/validate/mod.rs:1284`

Three call sites, not two. No gate checks this.

Both are exactly the ward's quarry: a number, in a doc comment, checkable by grep, with no gate behind it, currently wrong.

## Per-file ratio table (code : comment, exemptions marked, not dropped)

| File | Total | Code | Comment | Blank | Ratio | Exemption |
|---|---|---|---|---|---|---|
| alpha_tree.rs | 410 | 243 | 135 | 32 | 1.8:1 | doc-comment-rich, real bodies |
| clause.rs | 594 | 325 | 243 | 26 | 1.3:1 | doc-comment-rich |
| collect.rs | 75 | 50 | 21 | 4 | 2.4:1 | — |
| compiled_cond.rs | 1615 | 1034 | 503 | 78 | 2.1:1 | doc-comment-rich |
| compiled_rhs.rs | 772 | 532 | 204 | 36 | 2.6:1 | — |
| eval_insert.rs | 356 | 204 | 137 | 15 | 1.5:1 | doc-comment-rich |
| eval_test.rs | 151 | 84 | 55 | 12 | 1.5:1 | — |
| export.rs | 2565 | 1962 | 517 | 86 | 3.8:1 | ⛔-corrected-belief headers (load-bearing) |
| matcher.rs | 1119 | 685 | 368 | 66 | 1.9:1 | doc-comment-rich |
| **mod.rs** | 91 | 20 | 70 | 1 | **0.29:1** | **EXEMPT — module index/architecture map; each `pub(crate) mod` carries a one-line stone-map annotation. Low form count is structural (~15 module declarations), not sparse substance.** |
| purity.rs | 2633 | 1310 | 1246 | 77 | 1.05:1 | doc-comment-rich; ⛔-corrected-belief headers |
| reachability.rs | 2180 | 1186 | 855 | 139 | 1.4:1 | **see judgment below — not a naive ratio call** |
| step_payload.rs | 295 | 172 | 103 | 20 | 1.7:1 | — |
| vocabulary.rs | 1885 | 995 | 865 | 25 | 1.15:1 | doc-comment-rich; ⛔-corrected-belief headers |
| where_tree.rs | 946 | 733 | 157 | 56 | 4.7:1 | — |
| expr_ir/eval.rs | 1576 | 1164 | 367 | 45 | 3.2:1 | ⛔-corrected-belief headers |
| expr_ir/mod.rs | 1113 | 772 | 295 | 46 | 2.6:1 | ⛔-corrected-belief headers |
| validate/error.rs | 578 | 299 | 255 | 24 | 1.2:1 | doc-comment-rich |
| validate/mod.rs | 1765 | 1102 | 593 | 70 | 1.9:1 | ⛔-corrected-belief headers |
| validate/typing.rs | 804 | 383 | 400 | 21 | 0.96:1 | doc-comment-rich; ⛔-corrected-belief headers |
| wat/rete.wat | 541 | 216 | 286 | 39 | 0.76:1 | **spec file — mixed by design.** 34 top-level forms, all real declarations (verified by spot-read); comment weight is design rationale attached to them |
| wat/rete/compile.wat | 1162 | 829 | 306 | 27 | 2.7:1 | 26 top-level forms; spec file |
| wat/rete/syntax.wat | 368 | 199 | 156 | 13 | 1.3:1 | 10 top-level forms; spec file |
| wat/rete/acc.wat | 177 | 119 | 46 | 12 | 2.6:1 | 9 top-level forms |
| wat/rete/factbag.wat | 115 | 76 | 29 | 10 | 2.6:1 | 10 top-level forms |

No file fell below a genuinely worrying ratio once the house style is accounted for. `mod.rs` is the one row whose raw number would misread as "almost no substance" under the ward's literal table — it is a manifest file where the count of `pub(crate) mod` lines is the correct denominator.

## Described / hollow forms

**None found.** Swept `unimplemented!|todo!(|not yet implemented|stub\b` across all 20 Rust files — the only hits are citations of a briefing document filename (`BRIEF-the-f64-surface-is-a-stub.md`) or prose about "a native stub has no body AST" (a *different* kind of Rust-native intrinsic). Zero `TODO`/`FIXME`/`XXX`/`HACK` (re-confirmed). Swept the `.wat` files for placeholder bodies — zero.

## Naming alignment

No misalignment found. Every file's declared purpose matches its content.

## Judgment: `src/rete/reachability.rs`

**Verdict: substance, not description.** A real, executable instrument, not a write-up of one.

Evidence read this session: **20** `#[test]` functions; **54** assertions; **42** function definitions across 1,186 code lines. The assertions are non-vacuous and specific — e.g. `assert_ne!(fence_src, inline_src, …)` at `:505` checking two independently synthesized programs actually diverge, not merely that something ran. The high comment density (855 lines, 31 `⛔` corrected-belief blocks) is the ward's own exemption case: each block sits beside real generation/assertion code and states the methodological risk that code closes (e.g. `:26-34`'s *"an instrument that has not reproduced a known answer is not an instrument"* is immediately followed by the four calibration cells that pin exactly that).

The `#[cfg(test)]`-gating via `include!` at `mod.rs:86-89` is the correct shape for what this file declares itself to be, not evasion: a disconfirming probe answering "can a user actually get here" is by nature a test-time instrument over the compiled tree. Committing it whole is the project's own stated methodology working as designed — the size is proportional to covering a ledger with real per-cell assertions, not padding.

## Runes encountered

`grep -rn "rune:"` across scope found **zero** `rune:probare(...)`. All runes present belong to other wards and are out of this ward's jurisdiction to verdict.

## FINDINGS

Two ungated, currently-false header claims, plus the required judgment on `reachability.rs` (substance, not description — 2,180 lines, 20 tests, 54 assertions, 42 functions).
