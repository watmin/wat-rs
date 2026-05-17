# Arc 201 Slice 4 BRIEF — rename `signature-of` → `signature-of-defn` + consumer sweep

**Phase:** Fourth slice of arc 201. Slices 1+2+3 (`0706949`, `c9445a4`, `815d597`) shipped structured type-AST emission + Bundle accessors + `signature-of-fn` (fn-VALUE sibling). Slice 4 makes the asymmetry explicit at the API surface by renaming the existing name-keyword primitive.

**Originating signal:** post-slice-3 the substrate has two reflection primitives:
- `:wat::runtime::signature-of`     — takes a NAME keyword; symbol-table lookup
- `:wat::runtime::signature-of-fn`  — takes a fn VALUE; closure introspection

The `-fn` suffix on slice 3's sibling makes the input-shape contrast explicit, but the base name `signature-of` no longer reads honestly — it doesn't name "any signature lookup," it names "by NAME (defn lookup)." The asymmetric pair `signature-of-defn` / `signature-of-fn` reads true.

## Goal

Rename `:wat::runtime::signature-of` → `:wat::runtime::signature-of-defn` across:
- Substrate Rust registration + eval + dispatch + check
- Internal Rust identifiers (`eval_signature_of` → `eval_signature_of_defn`) — per FM 14 (`feedback_surface_retirement_internals`; arc 162 precedent)
- The single wat consumer (`wat/runtime.wat` `define-alias` macro)
- 13 test files that call the verb
- 3 active docs (USER-GUIDE.md, ZERO-MUTEX.md, MODULARIZATION-NOTES.md)

**No back-compat alias.** Per `feedback_refuse_easy_solutions` + `project_wat_llm_first_design` (one canonical path per task) — short-term sweep churn is the honest cost; alias mints a synonym that violates the doctrine.

## Required path (NO new substrate types/structs/special-forms/verbs)

This slice is PURELY MECHANICAL rename. Zero shape changes. Zero new behavior. Zero new tests.

- No new primitives
- No new types
- No new helpers
- No back-compat alias
- No `signature-of-fn` touched (slice 3 sibling — preserve as-is; the substring `signature-of` is contained in `signature-of-fn` — sweep must NOT corrupt the sibling)

## Scope (concrete site list from orchestrator's grep — sonnet verifies + adjusts)

### Rust substrate

`src/runtime.rs`:
- Dispatch arm at `:4046` — `":wat::runtime::signature-of" => eval_signature_of(args, env, sym),`
- Eval handler `fn eval_signature_of` at `:9782` + `OP` constant at `:9787`
- Comments at `:9018`, `:9532`, `:9774`, `:9837`, `:9888` (and any others sonnet finds)

`src/check.rs`:
- String-literal callee match at `:4721`
- `env.register` entry at `:14192`
- Check-side comment refs (`:11283`, `:14175`, `:14214`, `:14221`, `:14699`, `:14962`, `:14970`) — keep words that refer to the verb generally; update string literals that ARE the verb's spelling.

`src/freeze.rs`:
- Comment at `:760` and `:783`

`src/stdlib.rs`:
- Comment at `:211`

### Wat consumer

`wat/runtime.wat`:
- `define-alias` macro (`:22-29` area) calls `(:wat::runtime::signature-of target-name)` in its expansion body. Update to `(:wat::runtime::signature-of-defn target-name)`.

### Test files (13)

Sonnet must:
1. Sweep every literal call `(:wat::runtime::signature-of ...)` → `(:wat::runtime::signature-of-defn ...)`
2. Sweep Rust identifier `signature_of` → `signature_of_defn` ONLY where it refers to THIS primitive (test fixture function names, comments, doc strings)
3. PRESERVE `signature_of_fn`, `eval_signature_of_fn`, `wat_arc201_signature_of_fn.rs` filename — these are slice 3 sibling references and the substring is part of a longer name

Files (sonnet greps to confirm and acts):
- `tests/wat_arc143_lookup.rs`
- `tests/wat_arc143_define_alias.rs`
- `tests/wat_arc143_manipulation.rs`
- `tests/wat_arc136_do_form.rs`
- `tests/wat_arc144_lookup_form.rs`
- `tests/wat_arc144_uniform_reflection.rs`
- `tests/wat_arc144_special_forms.rs`
- `tests/wat_arc144_hardcoded_primitives.rs`
- `tests/wat_arc146_dispatch_mechanism.rs`
- `tests/wat_arc150_variadic_define.rs`
- `tests/wat_arc201_signature_of_fn.rs` (docstring/comment refs to `signature-of` as the named-callable counterpart — verify the file's own primitive references stay `signature-of-fn`)
- `tests/wat_arc201_holon_ast_accessors.rs`
- `tests/wat_arc201_structured_signature_types.rs`

### Active docs

- `docs/USER-GUIDE.md` — 4 hits at `:1202`, `:1315`, `:1383`, `:2824`, `:2870` (mechanical text replace; preserve surrounding prose)
- `docs/ZERO-MUTEX.md` — 1 hit at `:84`
- `docs/MODULARIZATION-NOTES.md` — 1 hit at `:75` (`eval_signature_of` → `eval_signature_of_defn`)

### Out of scope (DO NOT TOUCH)

Per `feedback_inscription_immutable` — past artifacts describe state at write-time:
- `docs/arc/2026/05/143-*/` (all)
- `docs/arc/2026/05/144-*/` (all)
- `docs/arc/2026/05/146-*/` (all)
- `docs/arc/2026/05/148-*/` (all)
- `docs/arc/2026/05/201-reflection-structured-type-ast/SCORE-SLICE-{1,2,3}.md` (historical)
- `docs/arc/2026/05/201-reflection-structured-type-ast/BRIEF-SLICE-{1,2,3}.md` (historical)
- `docs/arc/2026/05/201-reflection-structured-type-ast/EXPECTATIONS-SLICE-3.md` (historical)
- `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` (historical; orchestrator owns)
- `docs/COMPACTION-AMNESIA-RECOVERY.md` (orchestrator owns)

## Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release --workspace --no-fail-fast
```

**Verification:** existing test suites for arcs 143, 144, 146, 150, 201 must continue to pass against the renamed primitive. No new test file needed; the rename is verified by the EXISTING tests calling the new name and passing.

## Workspace baseline (commit `9105e17`)

Captured by orchestrator pre-spawn. Slice 4 EXPECTATIONS-SLICE-4 carries the actual baseline pass/fail numbers; this BRIEF's contract is "≤ baseline failure count."

## STOP triggers (true emergencies — surface, do not paper over)

1. **Grep finds `signature-of` references NOT in this BRIEF's scope list** — surface the count + file paths; the BRIEF may be missing a consumer. DO NOT silently sweep — names that aren't this primitive's spelling stay.
2. **A test fails after the rename** — surface which test + assertion. Distinguish: (a) test references stale name (sweep missed a site) — FIX in-slice; (b) test depends on primitive's BEHAVIOR — escalate, the rename should NOT change behavior.
3. **Substring corruption** — if a `signature-of-fn` reference gets mangled to `signature-of-defn-fn` or similar, that's substring-corruption — sweep must preserve the slice-3 sibling.
4. **The renamed primitive's eval/check path looks DIFFERENT from the original** — the rename is mechanical; if the eval handler logic appears to change, STOP — that's a behavior shift sneaking into a rename slice.
5. **Workspace baseline regresses (pass count drops, failure count rises) beyond pre-slice baseline** — STOP, surface diff.
6. **Any urge to mint a back-compat alias** — STOP. Per `feedback_refuse_easy_solutions` the rename is hard-cut.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may report `.claude/worktrees/agent-<id>/` paths — ignore; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT touch slice 1, 2, or 3 work — preserve `Bundle/children`, `Bundle/first`, `signature-of-fn` and all their tests.
- DO NOT touch historical artifacts (past BRIEFs, SCOREs, INSCRIPTIONs, DESIGNs, INTERSTITIAL-REALIZATIONS, recovery doc, past arc dirs).
- DO NOT mint a back-compat alias (the alias decision is RESOLVED in DESIGN; no second path).
- DO NOT modify slice 4 DESIGN.md (orchestrator owns).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::runtime::signature-of` is GONE from substrate registration (check.rs `env.register`); `:wat::runtime::signature-of-defn` REGISTERED | grep `":wat::runtime::signature-of"\b` (word-boundary) returns 0 in src/; `:wat::runtime::signature-of-defn` returns expected sites |
| B | Internal Rust identifiers renamed (`eval_signature_of` → `eval_signature_of_defn`); slice 3 sibling preserved (`eval_signature_of_fn` unchanged) | grep both names; preserve count verified |
| C | wat/runtime.wat `define-alias` macro uses new name; macro expansion still works | `cargo test --test wat_arc143_define_alias` passes |
| D | 13 test files swept; all calls updated; all pass | `cargo test` for each listed test file passes |
| E | Workspace failure count ≤ baseline (captured in EXPECTATIONS-SLICE-4); no regression | `cargo test --release --workspace --no-fail-fast` failure delta = 0 |

## Honest deltas to capture in SCORE

- **Site count accuracy.** Orchestrator estimated ~150 edits across ~18 files via grep. SCORE reports actual counts after sweep.
- **STOP-trigger fires.** Each fire is data; honest deltas section captures.
- **Substring preservation paranoia.** Did any tool/Edit mis-sweep `signature-of-fn` → `signature-of-defn-fn`? If yes — what mechanism caught it?
- **Comment-vs-code distinction.** Did any comments mention `signature-of` as a CONCEPT (general reflection lookup) where the new name doesn't fit, vs. mention it as THIS specific primitive? Report any ambiguity that warranted judgment.
- **Active docs prose.** USER-GUIDE.md prose may need light rewording beyond simple text replace (e.g., if a sentence says "signature-of returns Option" — that sentence stays accurate with the rename; but if it says "every queryable entity has signature-of" — the new spelling reads differently). Capture any prose edit that wasn't pure mechanical replace.

## Time-box

60-90 min predicted. Hard stop 120 min. The sweep is wide (18 files, ~150 edits) but mechanical (no design judgment beyond substring-preservation discipline).

## On completion

1. Write `docs/arc/2026/05/201-reflection-structured-type-ast/SCORE-SLICE-4.md` per § SCORE methodology + § Honest deltas.
2. Return final summary to orchestrator: rows passed/failed + workspace baseline delta + actual site count + any STOP-trigger fires.

You are launching now. T-minus 0.
