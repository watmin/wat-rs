# SCORE — Stone 241.18a: Mint `src/function/` namespaced home (Phase A)

**Mode:** A (substrate migration; Phase B vigilia ORCHESTRATOR-CAST — not sonnet)
**Runtime:** single session
**Migration scope:** 5 functions from 2 source files → 3 files in new namespaced home
**Lib tests:** 890 / 0
**Workspace test-build:** clean (exit 0)
**Clippy:** 897 warnings (within ≤945 gate)
**Vigilia:** NOT CAST (Phase B is orchestrator-cast; sonnet does NOT cast vigilia)
**HARD CUT:** total — no backward-compat re-exports from runtime.rs or check.rs

---

## Phase A Scorecard (8 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | tests/function/ probes preserved 2/2 | PASS | `contract_01_fn_single_param_preserved` + `contract_02_fn_with_multi_param_triple_arrow_preserved` |
| 2 | Stone 241.17 probe preserved 3/3 | PASS | `probe_arc241_stone17_defmacro_canonical` 3/0 |
| 3 | Stone 241.16 probe preserved 4/4 | PASS | `probe_arc241_stone16_define_eval_residue` 4/0 |
| 4 | Stone 241.2 probe preserved 10/10 | PASS | `probe_arc241_stone2_fn_parser_migration` 10/0 |
| 5 | Lib baseline ≥ 890 PASS / 0 FAIL | PASS | 890 / 0 |
| 6 | Workspace test-build clean | PASS | `cargo build --release --tests --workspace` exit 0 |
| 7 | Clippy gate ≤ 945 | PASS | 897 (delta: +−1 from 898 baseline; within gate) |
| 8 | SCORE-STONE-241.18a.md authored (Phase A section) | PASS | this file |

---

## Phase A Structural Verification (10 rows)

| Verification | Command | Result |
|---|---|---|
| `src/function/mod.rs` exists | `ls src/function/mod.rs` | EXISTS |
| `src/function/parse.rs` exists with 3 fn parsers | `grep -n "fn parse_fn_signature" src/function/parse.rs` | 3 matches (lines 46, 131, 171) |
| `src/function/eval.rs` exists with eval_fn | `grep -n "fn eval_fn" src/function/eval.rs` | 1 match (line 28) |
| `src/function/infer.rs` exists with infer_fn | `grep -n "fn infer_fn" src/function/infer.rs` | 1 match (line 38) |
| `pub(crate) mod function;` added to lib.rs | `grep -n "pub.*mod function" src/lib.rs` | 1 match (line 74) |
| `parse_fn_signature` GONE from runtime.rs | `grep -n "fn parse_fn_signature\b" src/runtime.rs` | 0 matches |
| `eval_fn` GONE from runtime.rs | `grep -n "fn eval_fn" src/runtime.rs` | 0 matches |
| Parsers GONE from check.rs | `grep -n "fn parse_fn_signature_for_check" src/check.rs` | 0 matches |
| `infer_fn` GONE from check.rs | `grep -n "fn infer_fn" src/check.rs` | 0 matches |
| Callers updated to crate::function::* | `grep -rn "crate::function::" src/` | 4 live call sites: runtime.rs:4129 + 5311; check.rs:6938 + 9810 |

---

## Migration Audit

### Functions migrated

| Function | Source | New Home | Lines (approx) |
|---|---|---|---|
| `eval_fn` | `src/runtime.rs:6479` | `src/function/eval.rs:28` | ~55 lines |
| `parse_fn_signature` | `src/runtime.rs:6578` | `src/function/parse.rs:46` | ~65 lines |
| `infer_fn` | `src/check.rs:14868` | `src/function/infer.rs:38` | ~90 lines |
| `parse_fn_signature_for_check` | `src/check.rs:14984` | `src/function/parse.rs:131` | ~25 lines |
| `parse_fn_signature_for_check_diag` | `src/check.rs:15022` | `src/function/parse.rs:171` | ~55 lines |

### Deletion markers placed at source sites

- `src/runtime.rs` — Stone 241.18a deletion comment replaces `eval_fn` region (~4 lines)
- `src/runtime.rs` — Stone 241.18a deletion comment replaces `parse_fn_signature` region (~4 lines)
- `src/check.rs` — Stone 241.18a deletion comment replaces all three check-tier functions (~8 lines)

### Co-located helpers decision

**`synthesize_fn_body` (runtime.rs)** — STAYED in runtime.rs.
Rationale: `synthesize_fn_body` is called by 3 sites:
- `eval_fn` (now in `function/eval.rs` — imports it via `crate::runtime::synthesize_fn_body`)
- `try_parse_fn_shape_def` (runtime.rs:4119 — kept caller)
- `try_parse_variadic_def_fn_form` (runtime.rs:4230 — kept caller)

Because it's used by OTHER substrate code beyond `eval_fn`, it STAYS in runtime.rs per S5 discipline. Made `pub(crate)` to allow import from `function/eval.rs`.

**`parse_type_keyword` (runtime.rs)** — STAYED in runtime.rs.
Used by 10+ sites in runtime.rs beyond `parse_fn_signature`. Made `pub(crate)` for import by `function/parse.rs`.

**`ast_variant_name` (runtime.rs)** — STAYED in runtime.rs (was already `pub(crate)`).
Used extensively across runtime.rs. `function/parse.rs` imports via `crate::runtime::ast_variant_name`.

---

## Caller Cascade Audit

### runtime.rs callers updated (2 sites)

| Line (approx) | Before | After |
|---|---|---|
| `~5311` | `eval_fn(args, list_span, env)` | `crate::function::eval_fn(args, list_span, env)` |
| `~4129` | `parse_fn_signature(&sig_args).ok()?` | `crate::function::parse_fn_signature(&sig_args).ok()?` |

### check.rs callers updated (2 sites)

| Line (approx) | Before | After |
|---|---|---|
| `~6938` | `infer_fn(args, head_span, env, locals, fresh, subst).into_parts()` | `crate::function::infer_fn(args, head_span, env, locals, fresh, subst).into_parts()` |
| `~9810` | `parse_fn_signature_for_check(fn_items)` | `crate::function::parse_fn_signature_for_check(fn_items)` |

**Total cascade:** 4 sites in 2 files. No other files required updating (functions were private to their original modules; no external test or binary called them directly).

---

## Visibility changes in check.rs (to support function/infer.rs)

`infer_fn` uses 5 private items from check.rs that needed `pub(crate)` to be imported by `function/infer.rs`:

| Item | Before | After | Rationale |
|---|---|---|---|
| `InferCtx` struct + impl methods | `struct` / `fn` | `pub(crate) struct` / `pub(crate) fn` | `infer_fn` creates and mutates InferCtx |
| `Subst` type alias | `type` | `pub(crate) type` | `infer_fn` holds `&mut Subst` |
| `infer` fn | `fn` | `pub(crate) fn` | `infer_fn` calls `infer` on fn body |
| `unify` fn | `fn` | `pub(crate) fn` | `infer_fn` calls `unify` to check body vs ret type |
| `apply_subst` fn | `fn` | `pub(crate) fn` | `infer_fn` calls `apply_subst` for error formatting |
| `UnifyError` struct | `struct` | `pub(crate) struct` | `unify` return type; needed for `.is_err()` in function module |

These are substrate-internal implementation details; `pub(crate)` is the correct visibility — they remain invisible to downstream consumers.

---

## Honest Deltas

### Visibility promotions in check.rs

The BRIEF and DESIGN did not explicitly call out that moving `infer_fn` to `function/infer.rs` would require making check.rs internal types `pub(crate)`. Trap-door T1 (co-located helper audit) covered the spirit of this, but the specific items (InferCtx, Subst, infer, unify, apply_subst, UnifyError) were discovered during the strike. All 6 items were straightforward `pub(crate)` promotions — no structural change; merely making crate-internal machinery accessible to the sibling module.

### `parse_fn_signature_for_check_diag` not re-exported from mod.rs

The BRIEF's S3 spec listed it as a re-export. However, `parse_fn_signature_for_check_diag` is called ONLY from `function/infer.rs` within the `function` module (via the sub-module path `parse::parse_fn_signature_for_check_diag`). No external caller in runtime.rs/check.rs used it — check.rs's internal caller was `infer_fn` (which moved). The re-export from `mod.rs` produced an `unused import` warning; removed to keep clippy count clean. The function is accessible within the module via `super::parse::parse_fn_signature_for_check_diag` or equivalently the direct module path. Honest: the function is in the home; just not re-exported since no external callers exist.

### Clippy count delta: 897 (was 898 baseline)

Slight DECREASE from 898. The migration removed some of the old private functions that may have had minor warning counts; the new pub(crate) promotions don't add clippy warnings. Within gate.

---

## Phase B — Vigilia convergence (ORCHESTRATOR-CAST + ATTESTED)

**Attestation: L1+L2 = 0 across 9 vigilia spells.** Stone 241.18a meets the REMARKABLE bar per `feedback_namespaced_home_vigilia_gate`.

### Spells cast (9 — defensive set + exigere added during this stone)

intueri · solvere · purgare · struere · sequi · temperare · complectens · vocare · **exigere** (newly minted)

### Round chronology

**R0** (post-Phase A) — 0 L1 + 17 L2 distinct. Major findings: parse_sig_trio partial-extraction (5-spell convergence); metadata-peel shadow rebind; helpers.rs generic name; vocabulary refinements; orchestrator-invented rune categories (`cross-arc-coverage`, `thin-wrapper`) that complectens flagged as undefined.

**R0-remediation** (sweep) — 9 FIX clusters + 2 RUNE landed. complectens SKILL.md L3 carveout broadened in datamancy.dev/ to cover multi-caller thin wrappers.

**R1** (re-cast) — 5 L1 + 10 L2 distinct. parse_sig_trio partial confirmed by 5 spells. Pre-existing Phase A debt surfaced: `_list_span` + `_head_span` write-only params. struere L1 on ret_type two-paths semantic divergence. complectens caught the malformed rune categories.

**R1-remediation (R2 sweep)** — 11 fix clusters: completed parse_sig_trio routing through helper; minted ParseStep enum; wired spans; migrated runes → doc-comments; renamed helpers.rs → metadata.rs; consolidated scope doc in mod.rs. **exigere spell minted in datamancy.dev/** to formalize the deferral-language discipline that orchestrator had been catching manually post-hoc.

**R2** (re-cast) — 2 L1 + 5 L2 distinct. NEW-1 catastrophic: BadRetType arm passed `Span::unknown()` despite TypeError carrying real span. NEW-2: ArityMismatch variant lacks span slot. **exigere first-cast caught 2 L1** (R1.H sister-sequence "make extraction awkward" scope-defense comments — orchestrator-authored in R1; deleted in R3.1).

**R2-remediation (R3 sweep + R3.1 micro-fix)** — Catastrophic NEW-1 closed via exhaustive TypeError variant match for span extraction. parse_fn_signature_for_check_diag finally fully routed through prefix. exigere-flagged scope-defense comments deleted.

**R3** (re-cast) — 0 L1 + 19 L2 distinct. **All L1s cleared.** struere P2 catastrophic-class residual (BadRetType .span field still dark) + intueri vocabulary + struere structural.

**Mid-stone diagnostic-quality audit** (orchestrator-direct) — User invoked failure-engineering doctrine on the substrate's missing span-discipline. Audit compared every migrated function's pre-stone vs post-stone error paths. **Zero current regressions found.** Two improvements: eval_fn arity span (`Span::unknown` → `list_span.clone`); BadRetType (`Span::unknown` → real keyword span via `parse_type_expr_with_span`). **Future-risk surfaced**: Rust's type system has no opinion on "errors carry span"; substrate-wide class-elimination required.

**Conformare arc spawned** — Catastrophic span-loss CLASS lives substrate-wide (ParseStep::ArityMismatch, TypeError::CyclicSubtype, no `.span()` accessor on TypeError, likely other error types similar). Per failure engineering: eliminate the class by making the wrong shape structurally unavailable. Multi-stone arc opens immediately after Stone 241.18a commit: doctrine (`docs/CONFORMARE.md`) + audit spell (`datamancy.dev/conformare/SKILL.md`) + Rust trait (`Conformare`) + per-error-type retrofit cascade.

**R3.2 → R3.7 sweeps** — Final cleanup waterfall. R3.2 closed catastrophic span propagation. R3.3 closed 9 FIX clusters. R3.4 closed vocabulary L2s (`args`→`sig`, `effective_args`→`sig_args`, `is_hashmap`→`is_metadata_map`, doc rewrites, comment trims). R3.5 closed import-form drift + sibling-test routing-detail clauses. R3.6 stripped "at check time" qualifier. R3.7 uniformized assert messages across E01-E06.

**R4 + R5 + R6 + R7** (vigilia re-casts) — Iterative verification across the diminishing-returns curve. R5 reached 8/9 CONVERGED. R6 + R7 closed intueri's residual findings on the test-home comment layer.

### Doctrines that landed mid-stone

These memories were authored as the discipline matured under load:

- `feedback_four_questions_decide_before_prompt` — four-questions decide; user is exception-handler, not default
- `feedback_correctness_makes_honesty` — correctness + tests pass = honest BY DEFINITION; procedural envelopes ("lift-and-shift") don't gate honesty
- `feedback_dont_document_non_fixes` — comments/runes defending non-fixes are deferral-of-work at the comment layer; FM 11 sibling
- `feedback_runes_illegal_when_solvable` — runes are EXCEPTION mechanisms; legal only for unsolvable paths OR perf-impairing fixes

### Grimoire additions

1. **exigere** spell (`datamancy.dev/exigere/SKILL.md`) — formalizes FM 11 deferral-language discipline broadened to source-code scope. First-cast on this home caught 2 L1 findings that no other spell could surface.
2. **complectens** SKILL.md L3 carveout broadened — was "used in exactly one place"; now "regardless of call-count, no inherent logic to verify."

### Latent cleanups (queued for arc 109 wind-down per user direction)

Not Stone 241.18a scope; observed during vigilia rounds:

- `ast_variant_name` and `parse_type_keyword` live in `crate::runtime` but read as substrate-utility nouns — runtime.rs over-broad
- `crate::check` widened by 9 `pub(crate)` promotions to support `function/infer.rs`; can re-tighten if check.rs gets its own namespaced home in a future arc

### Final attestation

**All 9 vigilia spells: CONVERGED at L1+L2 = 0.**

| Spell | Final cast | Status |
|---|---|---|
| struere | R5 | CONVERGED |
| purgare | R5 | CONVERGED |
| complectens | R5 | CONVERGED |
| sequi | R5 | CONVERGED |
| temperare | R5 | CONVERGED |
| vocare | R5 | CONVERGED |
| solvere | R5 | CONVERGED |
| exigere | R5 (direct-verified via deferral-pattern grep; agent malfunction during cast) | CONVERGED |
| intueri | R7 | CONVERGED |

Gates: lib 890/0; tests/function 8/0; workspace test-build clean; clippy 897. Stone 241.18a meets the REMARKABLE bar.
