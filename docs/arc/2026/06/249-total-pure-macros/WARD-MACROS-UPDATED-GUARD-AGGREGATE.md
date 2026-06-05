# src/macros/ — updated-guard ward aggregate (the HELD-stamp round)

> Written mid-round as compaction insurance. The 12 inward casts are COMPLETE;
> the fight-sweep (items A–W below) was IN FLIGHT at this writing (agent task
> a23418bc2a10b8708; its report, if finished, is in the session task outputs).
> ON RECOVERY: `git status` — if src/macros/ is dirty, the sweep landed some or
> all items; verify against this list + the gates below, commit when green.
> Then: convergence re-cast of the touched lenses → circumspicere LAST → the
> `vigilatum` stamp HELD since 249.2a. Cast mechanic: fetch spells from the
> signed datamancy channel, embed verbatim in workers (embed-never-fetch).

## The 12 inward verdicts (complete updated guard)

| spell | verdict |
|---|---|
| sequi | CONVERGED 0+0 — registry &mut correctly narrowed; ScopeId threaded explicitly; pure validator |
| excusare | CONVERGED 0+0 — 7 exemptions ALL HOLD (verified vs live tree); L3: nav-hint line-numbers drifted |
| vocare | CONVERGED 0+0 — all 28 tests at correct (internal-caller) vantage; L3: arc138 stale comment |
| solvere | 1 L2 — EXPANSION_DEPTH_LIMIT misplaced in parse.rs (belongs expand.rs); runes EARNED |
| conformare | 1 L2 — parse.rs:89 dead `_` arm Span::unknown() + FALSE "no span" comment; Pattern A holds everywhere else (full audit recorded) |
| intueri | **1 L1** + 5 L2 — L1: variadic under-arity renders "expects N" when truth is "at least N" (expand.rs:185). L2: qb×2, WHAT-banner, "— UNCHANGED", as_quasiquote_body returns Option |
| perspicere | 1 L2 — `Result<Vec<WatAST>, MacroError>` at 7 sites → mint `ExpandBatch`; collect-idiom wildcards runed CLEAR |
| exigere | **1 L1** — tests.rs:609 ",,@X NOT yet supported" untracked deferral → FM-11 affirmative form. 13 cited arcs all verified on disk |
| temperare | 1 L2 — hygiene gate + purity gate are pure predicates of the immutable MacroDef body, re-run per invocation → hoist to definition time (fail-at-definition) |
| purgare | 3 L2 — is_pure_total dead quasiquote/quote arms + factually-wrong comment (eval.rs:445-452); validate_pure_total over-visible; expand_macro_call over-visible |
| complectens | **1 L1** + 5 L2 — L1: 26-line inline closure, message-less panic arms (tests.rs:302). L2: drill helper ×2, expand_once setup dup, 3-way test split, find_defmacro_body unproven; rune WARRANTED on the embedded-program fixture |
| struere | 4 L2 — validate_pure_total → pub(super); register_stdlib pub→pub(crate) (TRUSTED-by-convention); 3 allows missing struere ledger lines; literal-binder gate's silent non-Vector pass-through undocumented (behavior CORRECT — document it) |

**AGGREGATE: 3 L1 + 22 L2 (+L3 notes).** All names cast-proposed (naming protocol satisfied):
ExpandBatch (perspicere), ArityTooFew (intueri), extract_typed_binding_sym + drill_let_binder_ident (complectens), quasiquote_inner (intueri).

## The fight-sweep items (A–W)

**L1s:** A. `MacroErrorKind::ArityTooFew{name,minimum,got}` for the variadic branch + honest Display + test. B. tests.rs:609 → FM-11 affirmative cut (state present behavior). C. extract `extract_typed_binding_sym` named helper, messages on every panic arm, + sibling test.

**expand.rs:** D. qb→body ×2. E. delete WHAT-banner (:248). F. delete "— UNCHANGED" (:261). G. as_quasiquote_body→quasiquote_inner. H. add `rune:struere(host-constraint)` at the 3 too_many_arguments sites + the expand_template subset note (macro_scope→qq path only; rest_param→program-body only). I. document the deliberate non-Vector pass-through at :444/:465 (no name introduced; eval owns malformed diagnostics). J. EXPANSION_DEPTH_LIMIT → expand.rs + mod.rs re-export.

**parse.rs:** K. dead arm: form.span().clone() + fix the false comment.

**eval.rs:** L. delete dead is_pure_total arms + wrong comment. M. validate_pure_total → pub(super). N. nav-hint: fn NAMES not line numbers. O. THE HOIST: run check_program_body_hygiene + validate_pure_total(template) ONCE at registration (single chokepoint), fail-at-definition; per-call validations of substituted forms stay; STOP-O if not cleanly relocatable.

**registry.rs:** P. register_stdlib → pub(crate).

**alias:** Q. `pub(crate) type ExpandBatch = Result<Vec<WatAST>, MacroError>` + 7 sites.

**tests.rs:** R. drill_let_binder_ident + refactor 2 scope tests + sibling test. S. expand_setup helper for expand_once test. T. split substitute_bindings into 3. U. find_defmacro_body sibling test. V. rune:complectens(inline-fixtures) on make_deftest_shaped test. W. fix arc138 stale comment.

## Gates (every round, orchestrator re-run)

```
cargo build --release --tests -p wat
cargo test --release --lib -p wat            # baseline 911/0/1 + new tests
cargo test --release --test probe_macro_hygiene_capture --test probe_argspec_rest_param_hygiene \
  --test probe_check_scoped_param_resolution --test probe_hash_scope_renumber \
  --test probe_hygiene_scopes_reader_gate    # 3+1+2+2+2
cargo clippy --release -p wat | grep src/macros   # clean
```

## After the sweep converges

1. Re-cast the lenses whose findings were fought (fresh eyes, no priming).
2. circumspicere LAST (embed its text; hand it the inward coverage; hunt the surround — claims-vs-code on mod.rs's hygiene claims [now probe-backed], the engine's default-deny claims, unenforced invariants, negative space).
3. Termination by judgment (no new L1, every L2 fought-or-ledgered) → apply the HELD `vigilatum` stamp to src/macros/mod.rs (mirror src/scope/mod.rs's stamp form: the 12-spell muster, conditional determinations, declared invariants w/ living gates).
4. Then: collection/ re-earn (light) + wat/core.wat (spec/DSL set: cernere/probare/conferre + exigere + circumspicere; + the named threading deftest) → 249.N INSCRIPTION (FM-11 grep) → arc 249 CLOSED.
