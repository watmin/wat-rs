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

## R2 — the convergence re-cast (2026-06-05, on the fixed tree at `6f57630e`)

The A-W fight sweep landed (`6f57630e`, 25/25 applied, 3 L1 killed, lib 917/0/1).
Nine fought lenses re-cast fresh-eyes (spell texts re-fetched from the signed
channel, embedded verbatim; sequi/excusare/vocare stayed converged from R1, not
re-cast). Verdict: **0 L1 + 13 L2 + ~9 L3; all 11 runes in the home audited
CLEAR by their owning spells.**

| spell | R2 verdict |
|---|---|
| exigere | CONVERGED 0+0 — every R1 L1 stayed dead; tests.rs ,,@ scope-bound verified affirmative |
| solvere | 2 L2 — divergent quasiquote discriminant (parse.rs head-only `matches!` vs expand.rs `quasiquote_inner` exact-2; malformed `(qq a b)` misroutes); parse_defmacro_form braids hygiene+purity validation inline (hoist kept, delegation missing) |
| conformare | 1 L2 — eval.rs ~83 coarsens `e.span` → `form.span()`; the runtime's precise span discarded at the mapping |
| intueri | 3 L2 — `splice_children` names mechanism not contract (cast proposes `flatten_template_children`); parse.rs ~175 `temperare —` spell-jargon label; expand_template arm param-asymmetry invisible in the arms. +2 L3 declined (expand_setup 4-tuple; is_pure_total rename would desync the runtime-mirror rune language) |
| perspicere | 0 L2 + 3 L3 — internal `Result<Vec<_>, _>` bindings should speak `ExpandBatch` (expand.rs ~112/~146/~914) |
| temperare | 1 L2 — THE HOIST IS INCOMPLETE: macro_eval re-runs validate_pure_total on the definition-validated `def.body` per invocation (expand_program_body path); unquote/splice paths pass substituted forms and must KEEP validating. +2 L3 (as_str rebinding; with_capacity) |
| purgare | 3 L2 — ExpandBatch pub(crate) has zero external consumers; two structurally-dead defensive arms unmarked (parse.rs `_ =>` guarded by is_defmacro_form at all 4 call sites; expand.rs `_ => &[]` silent-empty on an impossible shape) |
| complectens | 1 L2 — expand_keeping_defmacros (only direct expand_form exerciser; keep-vs-strip semantics) lacks a sibling test. Rune tests.rs:740 audited CLEAR (outer bindings = 1, verified) |
| struere | 2 L2 — parse.rs ~68 doc claims metadata "stored" (parse drops it); `depth` name collision (expand_form fixpoint guard usize vs quasiquote nesting u32 → rename `expansion_depth`). All 4 struere/sequi-adjacent runes re-verified CLEAR |

**Orchestrator weighing — rejected findings (recorded, not fought):**
- struere F4 ("complectens rune misfiled in struere's namespace") — PHANTOM: runes
  are per-spell namespaced by design; tests.rs:740 belongs to complectens, whose
  own cast audited it CLEAR this same round.
- intueri's two L3 test-taste items — declined (L3 does not gate; the
  is_pure_total rename would desync the rune prose mirroring the runtime
  dispatch arm).
- complectens' two L3s — the cast itself judged extraction costs more than it gains.

**R2 fight sweep: 13 L2 + 4 mechanical L3 dispatched** (verify-claim-first,
blocked-not-worked-around; gates = test-build + lib 917-baseline + 5 hygiene
probes + macros-clippy-empty). On completion: re-run gates orchestrator-side,
commit, then R3 convergence check → circumspicere LAST → the HELD stamp.
