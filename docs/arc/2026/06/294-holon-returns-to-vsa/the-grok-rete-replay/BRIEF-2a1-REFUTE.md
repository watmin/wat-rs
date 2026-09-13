# REFUTE — 2a1: the door drops a program's macro-generated types, silently

The strike is solid:
- the snapshot and the one shared `TypeInfo` constructor (`type_info_value`) are right;
- 12–17 ms a call;
- sift_rules and W2f register despite stale bodies;
- the orchestrator's own mutation (the acronym pre-registration neutralised) turned exactly
  `declared_types_acronym_create_web_acl` red.

Four things keep it from closing. Each was measured by the orchestrator on `175b49ea9`.

## R1 — a top-level MACRO CALL that declares types is dropped (the silent drop the brief ruled out)

`tests/diagnostics/probe_diagnostic_c3_macro_emits_record_def.wat` defines `:t::mk`, whose expansion
declares `:demo::Req` (defrecord) and `:demo::Op` (defenum), then calls `(:t::mk :demo)` at top
level. The verb, called from wat on that file's forms (`bootstrap/era/probe-R/door.wat`), returns
`Ok` with **no types at all**.
- **Cause:** `is_declaration_form` (`src/freeze/env.rs`) keeps a top-level form only when its head is
  on a hand-written list. `:wat::core::defmacro` is on it, so the macro is registered; the CALL
  `(:t::mk …)` is not, so it is never expanded.
- **The contract** (the verb's own doc): "the types those forms added". A program whose macro call
  declares types must report them.
- **Keep the reason for filtering.** Expanding a `defn` body evals macros like `mem-store/start`.
  The fix must not expand bodies.
- **The case to pin:** c3 returns `:demo::Req` and `:demo::Op` (`Go [req]`).

## R2 — the filter is a hand list; make its edge a check, not a convention

A declaration form added to the substrate later would be dropped without a sound. Derive the admitted
set from the substrate where it can be derived. Where it cannot, add a test that FAILS when a
type-registering head exists that the filter does not admit, naming it. That test must go RED once
(drop a head from the list) before it counts.

## R3 — `Refused` names the program's FIRST form, not the one that failed

Every failure is built with `fail(first.clone(), …)`. The case-7 test has one form, so it cannot
tell. Name the declaration that failed. Pin it with a two-form program whose SECOND form is
malformed: `Refused.form` must be that second form.

## R4 — the oracle covers one case of the four the brief named

`declared_types_oracle_matches_type_of_after_startup` compares one plain `defenum` (Colour). The
brief asked for each of cases 1–4 that loads normally. `tests/macros/probe_arc265_acronym_registry_svc.wat`
loads on HEAD (defsurface `Op`/`Reply` plus the `CreateWebACL` acronym). Add the cases that load; for
any that cannot, say why in the SCORE.

## STOP triggers

- **STOP-1:** R1 cannot be fixed without expanding `defn` bodies. Report the verbatim failure.
- **STOP-2:** the floor is red. Paste the whole block verbatim, and do not re-run.

## Tier

Commit on green (floor + clippy 0). **Do not push.** Yield with `SCORE-2a1-REFUTE.md`, one row per R.
