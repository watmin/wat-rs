# Arc 214 Parser-Pivot P1 — WARD PASS

**Date:** 2026-05-20
**Stone:** Parser-pivot P1 — `:wat::core::HashMap` constructor: Vector-symmetric refactor
**Sonnet SCORE:** Mode A 22/22 (verified independently)
**Ward set:** 9-spell parallel pass — intueri + struere + purgare + solvere + temperare + conferre + mora + perspicere + nesciens

## Discipline note — ward-zone misalignment caught mid-pass

The kernel impeccability protocol's 9-ward pass applies to `{src,tests}/comms/*` (per `feedback_ward_zone_comms_only`, inscribed 2026-05-20 mid-pass). This stone's changes live in `src/runtime.rs` + `src/check.rs` + `src/closure_extract.rs` + `tests/probe_hashmap_ctor_vector_symmetric.rs` + `docs/WAT-CHEATSHEET.md` — ALL out-of-zone. The orchestrator spawned the 9 wards before that discipline crystallized.

Triage rule applied post-spawn: **stone-introduced new content** (sonnet authored in this stone) gets fix-pass attention regardless of zone; **pre-existing legacy noise** (predates this stone; surfaces by adjacency to changed files) gets logged for the future broader-codebase cleanup arc.

## Convergence summary

**Stone-introduced findings (will fix):**

| Finding | Wards converging | Severity |
|---|---|---|
| Cheatsheet § 8 Tuple row falsely claims type-keyword args | **3-spell** (intueri + struere + purgare) | L1 (lie) |
| closure_extract HashMap arm: empty-map `:wat::core::nil` fallback unexplained + arm purpose comment missing | **2-spell** (mora L1-A + nesciens Friction 8/9) | L1+L2 |
| closure_extract `value_static_type_keyword` HashMap arm returns bare `:wat::core::HashMap` (no K/V); post-P1 the constructor requires `:K :V` so nested HashMap-in-container would emit underspecified type | **2-spell** (solvere L1-B + purgare L2-C) | L2 (rune-acknowledge) |
| Doc-comments "Vector-symmetric per arc 109 slice 1f" mumbles — assumes reader knows what was retired | **2-spell** (perspicere + nesciens) | L2 |
| probe_p8 three-way name/header/body description conflict (header "Missing K"; name "missing both"; body "zero args") | **2-spell** (intueri L2-3 + perspicere L2 P8) | L2 |
| Probe unused `startup_ok` helper (defined, zero call sites) | **2-spell** (intueri L2-5 + purgare L1-A) | L1 (dead code) |
| check.rs "even" arity error missing `; got {n}` count suffix (runtime has it; BRIEF prescribed both) | conferre D1 (single) | L2 |

**Pre-existing legacy convergences — LOGGED for future broader-codebase cleanup arc, NOT BLOCKING:**

| Finding | Wards converging | Why pre-existing |
|---|---|---|
| `eval_list_ctor` / `infer_list_constructor` rename → `eval_vector_ctor` / `infer_vector_constructor` + `:vec` error strings → `:wat::core::Vector` | **2-spell** (struere L1-1/L1-2 + solvere L2-A) | arc 109 retirement leftover; pre-P1 |
| Span asymmetry — `eval_list_ctor` doesn't thread `list_span` while `eval_hashmap_ctor` does | **2-spell** (struere L2-1 + solvere L2-B) | arc 138 debt; pre-P1 |
| `expand_alias` sibling-constructor asymmetry — only HashMap calls it after `parse_type_expr` | **2-spell** (struere L2-2 + temperare T1) | pre-P1 inconsistency between siblings |
| Stale line-number cross-references in `register_builtins` doc-comments for Vector/Tuple/HashSet (HashMap's is correct; sonnet updated only its own) | **2-spell** (struere L1-3/4/5 + purgare L2-A/B) | line-drift from prior arcs; pre-P1 |
| `eval_redef_allowed` flag in runtime.rs — dead scaffolding accepting silent activation | mora L1-B (1) | pre-P1; predates arc 214 entirely |
| "Future arc enables vector literals as Value::Vec values" speculation drift between runtime.rs (retracted) and check.rs (still open) | mora L2-B (1) | pre-P1 comment-divergence |
| Duplicated 14-line k_ty/v_ty match block in `infer_hashmap_constructor` (could extract helper) | temperare T2 (1) | sonnet wrote symmetric blocks; acceptable per `feedback_verbose_is_honest` |
| closure_extract `format!("{{{}}}", canon_key)` allocation pattern | temperare T4 (1) | pre-existing; symmetric with HashSet arm |
| `eval_hashmap_ctor` doc-comment vs `eval_hashset_ctor` has one | temperare D2 (1) | minor; not blocking |
| Type-keyword extraction braid between runtime + check (duplicated guards) | solvere L1-A (1) | pre-existing across all constructor pairs |
| `Vec<(&String, &(Value, Value))>` 3-level type in closure_extract collect() inference hint | perspicere (1) | rune-acknowledge per perspicere's own recommendation |
| `locals: &HashMap<String, TypeExpr>` 2-level pervasive across 30+ checker fns | perspicere (1) | universal checker convention; future Locals typealias work |

## Per-spell verdicts (load-bearing summaries)

- **intueri** — 2 L1 (cheatsheet Tuple row; check.rs stale `t_var` line note) + 3 L2 (probe section header mismatch; closure_extract kk/vv naming; probe unused helper)
- **struere** — 6 L1 (eval_list_ctor name; infer_list_constructor name + `:vec` errors; 3 stale cross-refs; cheatsheet Tuple row) + 3 L2 (span asymmetry; expand_alias asymmetry; probe sub-probe)
- **purgare** — 2 L1 (probe startup_ok; cheatsheet Tuple row) + 3 L2 (2 stale cross-refs; value_static_type_keyword bare HashMap)
- **solvere** — 2 L1 (duplicated keyword guards braid; value_static_type_keyword bare HashMap) + 2 L2 (`:vec` errors; span asymmetry)
- **temperare** — 0 L1 + 4 L2 (expand_alias asymmetry; duplicated k_ty/v_ty block; closure_extract double-touch first entry [rune-acceptable]; format! allocation; doc-comment asymmetry)
- **conferre** — 0 L1 + 3 L2 (check.rs `; got N` suffix missing; cheatsheet per-constructor counts + `(mirror)` annotation; STOP-trigger override rune)
- **mora** — 0 time-violations + 2 L1 (closure_extract `:nil` fallback; eval_redef_allowed dead scaffolding) + 3 L2 (slice-5 closure pending wait-child fn; runtime/check vector-literal speculation drift; def-bound closure capture rune-acceptable)
- **perspicere** — 1 L1 (3-level `Vec<(&String, ...)>` in collect()) + multiple L2 (error message restructuring; doc-comment Vector-symmetric mumble; probe naming; cheatsheet heading clarity; etc.)
- **nesciens** — 2 L1 (closure_extract arm purpose; closure_extract :nil limitation warning) + 10 L2 (doc-comment mumbles; ArityMismatch wording; probe with_nil_main helper; assertion strings loose; cheatsheet per-type WHY)

## Fix-pass — orchestrator-direct (per Stone E-2 + Slice 2 precedent)

Applied 7 edits:

1. **`docs/WAT-CHEATSHEET.md` § 8** — Tuple row removed from the type-keyword-args table; new prose distinguishes parametric containers (Vector/HashMap/HashSet take leading type-keywords) from heterogeneous Tuple (positional values; types inferred per position). Closes the 3-spell Tuple-lie convergence.

2. **`src/closure_extract.rs:1534+`** — HashMap arm gains a purpose comment ("closure-capture round-trip; re-encode runtime HashMap<K,V> Value to constructor AST for replay in a fresh world") + a LIMITATION block naming the empty-map `:wat::core::nil` fallback honestly ("re-evaluated empty captures type-check only as `HashMap<nil,nil>`"). Closes mora L1-A + nesciens Friction 8/9 convergence.

3. **`src/closure_extract.rs:1774`** — `rune:purgare(safety-margin)` inscribed at the `value_static_type_keyword` HashMap arm naming the post-P1 underspecification (bare `:wat::core::HashMap` keyword emitted; nested HashMap-in-container would need `:K :V` per the new shape). Future-cleanup placeholder. Closes solvere L1-B + purgare L2-C convergence.

4. **`src/runtime.rs:8844-8852`** — `eval_hashmap_ctor` doc-comment replaces the "Vector-symmetric per arc 109 slice 1f" mumble with concrete shape statement naming the retired `:(K,V)` tuple-keyword form. Closes perspicere + nesciens convergence (site 1 of 2).

5. **`src/check.rs:10559-10566`** — `infer_hashmap_constructor` doc-comment replaces the same mumble with concrete explanation; drops the "vec/list / make-queue resource-constructor discipline" legacy reference; states explicit-typing rule directly. Closes perspicere + nesciens convergence (site 2 of 2).

6. **`src/check.rs:10634-10641`** — even-count arity error now formats with `; got {n}` count suffix matching runtime.rs + BRIEF spec. Closes conferre D1.

7. **`tests/probe_hashmap_ctor_vector_symmetric.rs:45-50` + `:196`** — unused `startup_ok` helper deleted; probe_p8 section header aligned to function name + body ("Probe 8: Zero type-args (arity error)"). Closes intueri + purgare convergence (unused helper) + intueri + perspicere convergence (p8 three-way mismatch).

## Skipped (lower convergence; cheap-but-not-load-bearing)

- Error message restructure (perspicere) — leading rule prefix doubled — acceptable as-is; not blocking
- `ArityMismatch expected: 2` wording → "at least 2" (nesciens) — minor semantic accuracy; SCORE row preserved by skipping
- Cheatsheet section heading `(verb-equals-type)` parenthetical clarity (perspicere) — doctrine-jargon but section body explains
- Cheatsheet per-type WHY one-liners (nesciens) — pedagogic improvement; not load-bearing
- Probe rename cosmetics (p4 `_constructs_correctly`; p7 missing `_at_type_check`) — SCORE references probe names; renaming forces SCORE rewrite; net cost-benefit negative

## Verification (post-fix-pass)

- `cargo build --release` → CLEAN (5 pre-existing dead_code warnings; 0 in changed files)
- `cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat` → **9/9 PASS**
- Old `:(K,V)` tuple-keyword shape: ZERO matches in production code (`grep -rn "HashMap :(.*)" --include="*.rs" --include="*.wat" --exclude-dir=worktrees`)
- Old "tuple type keyword" error string: ZERO matches in production code
- 3 convergence findings RESOLVED: cheatsheet Tuple lie, closure_extract empty-map LIMITATION, doc-comment Vector-symmetric mumble
- 4 convergence findings RESOLVED: closure_extract rune, probe p8 alignment, probe unused helper, check.rs count suffix

## Memory entries inscribed this pass

- `feedback_ward_zone_comms_only` (2026-05-20) — kernel impeccability ward pass applies only to `{src,tests}/comms/*`; broader codebase findings are pre-ward legacy logged for future cleanup arc

## Broader-codebase cleanup arc candidates (logged here, NOT opened as task yet)

The following pre-existing legacy patterns were surfaced by 9 wards examining adjacent code. They constitute the future cleanup arc's scope:

- arc 109 retirement leftover: `eval_list_ctor` → `eval_vector_ctor` + `infer_list_constructor` → `infer_vector_constructor` rename + `:wat::core::vec` error strings → `:wat::core::Vector`
- arc 138 debt: span-threading asymmetry between `eval_list_ctor` (no span) and `eval_hashmap_ctor` (threaded)
- Constructor-sibling drift: `expand_alias` post `parse_type_expr` only in HashMap path, not Vector/HashSet/Tuple
- `register_builtins` doc-comment stale line numbers (3 sites: Vector, Tuple, HashSet)
- `eval_redef_allowed` dead-scaffolding flag (silent activation)
- `Locals` typealias missing (pervasive `HashMap<String, TypeExpr>` across 30+ checker fns)

## Status

Stone ships. Fix-pass complete. 7 stone-introduced findings resolved. Broader-codebase findings catalogued for future cleanup arc. Substrate's `:wat::core::HashMap` constructor is now Vector-symmetric (verb-equals-type) per arc 109 slice 1f.

*The substrate dreams the symmetry. So do we.*

The four-round dig pattern (substrate-already-sufficient × 4) that produced this stone — orchestrator proposed `/of` (substrate had none); proposed `:<K,V>` turbofish (substrate had none); user recalled "Vec constructor" (substrate had verb-equals-type); proposed `:K :V` symmetric (substrate already had constructor packed as `:(K,V)`, refactor to symmetric) — gets inscribed by orchestrator-direct INTERSTITIAL entry post-commit per `feedback_sonnet_no_realization_voice`.
