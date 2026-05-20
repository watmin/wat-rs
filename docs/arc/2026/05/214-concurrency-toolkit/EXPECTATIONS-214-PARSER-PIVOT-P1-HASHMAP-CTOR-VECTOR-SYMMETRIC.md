# Arc 214 — Parser pivot Stone P1 — EXPECTATIONS

## Independent prediction

- **Runtime band:** 30-50 min Mode A. Refactor is tight — substrate addition is small (3 function bodies + comment updates + 9 probe tests + 2 paperwork files). Smaller than Slice 2 forward-correction.
- **LOC changed:** ~40-60 net delta in code (eval_hashmap_ctor body refactor; infer_hashmap_constructor body refactor; doc-comment update); ~120-180 lines added in probes; ~20-30 lines paperwork.
- **New files:** 2 (probe test file + SCORE doc).
- **Surprises expected:** LOW. Pre-spawn grep returned ZERO downstream callers of the old `:(K,V)` form in production wat. Pre-spawn read of `eval_hashmap_ctor` confirms the body structure is straightforward to refactor. Risk is concentrated in test-fixture migration (if any exist exercising the old form).

## Honest-delta watch

### Risk 1 — Existing tests use the old `:(K,V)` form

**What:** Pre-spawn grep on production wat returned zero. Tests in `tests/*.rs` not yet checked exhaustively. If existing tests exercise the old shape, they need migration to the new shape OR retirement if their premise was specifically tuple-keyword.

**Mitigation:** STOP trigger fires immediately on encountering such tests; sonnet reports the count + file paths; orchestrator decides migrate vs retire per case.

### Risk 2 — `infer_hashmap_constructor` shares machinery with other constructors

**What:** If the function shares helpers with `infer_list_constructor` or `infer_tuple_constructor` (Vector + Tuple), refactoring HashMap might affect them. Pre-spawn read of `infer_hashmap_constructor` should confirm scope isolation before changing.

**Mitigation:** Sonnet reads the function first; if shared machinery surfaces, STOP and report. Workspace baseline run after refactor will surface any regression in Vector/Tuple constructors.

### Risk 3 — `hashmap_key` helper depends on the old shape

**What:** `runtime.rs:8884` calls `hashmap_key(":wat::core::HashMap", &k)` inside `eval_hashmap_ctor`. The helper coerces a Value to a HashMap key. If the helper has any assumption about the tuple-keyword shape (which would be odd but possible), it needs adjustment.

**Mitigation:** Sonnet reads `hashmap_key` first; pre-spawn dig suggests it's a pure coercion utility independent of constructor shape, but verify.

### Risk 4 — Error message strings affect tests

**What:** Tests may grep for specific error message substrings like "tuple type keyword :(K,V)". Updating the message wording will break grep-based tests.

**Mitigation:** Workspace test run after refactor will surface; mitigate by updating any affected test assertions to match the new error messages.

### Risk 5 — Type-arg validation logic

**What:** The new parser needs to validate `args[0]` and `args[1]` are valid type-keywords (per the existing type-keyword recognition machinery — what makes `:wat::core::i64` a valid type-keyword but `:foo` not). Sonnet may need to reuse the type-keyword recognition helper that the tuple-keyword path already uses internally.

**Mitigation:** Sonnet reads the existing tuple-keyword recognition path; reuses the underlying type-keyword validator for each of the two separate keywords.

### Risk 6 — Probe count overshoot

**What:** BRIEF specifies 9 probes; sonnet may add adjacent tests "while I'm there." Per `feedback_iterative_complexity` scope-creep concern.

**Mitigation:** BRIEF "Out of scope" section + probe list explicit; ward pass (purgare) flags scope-creep tests.

### Risk 7 — WAT-CHEATSHEET doesn't have a "constructors" section yet

**What:** Cheatsheet may not have a dedicated constructor section. Sonnet may need to create one OR insert HashMap row in an adjacent section.

**Mitigation:** BRIEF allows either; sonnet picks based on file structure. Ward pass (struere) verifies placement is honest.

### Risk 8 — DESIGN.md (arc 214) needs signature update

**What:** DESIGN.md may reference HashMap constructor signature in its examples (ProgramEnv discussion in Slice 4 prep section). Updating the signature reference is sub-architectural (signature, not narrative) so it's in sonnet's scope. The PROSE that contextualizes the four-round dig is orchestrator-direct.

**Mitigation:** Sonnet updates signature references only (mechanical); skips any prose that names the convergence-with-self pattern.

### Risk 9 — arc 058 file location

**What:** `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md` is outside `wat-rs/`. Per `feedback_brief_paths_in_scope`, sonnet shouldn't operate outside `/home/watmin/work/holon/wat-rs/`.

**Mitigation:** Sonnet reports the row CONTENT in SCORE; orchestrator adds the row to the arc 058 file post-ship (cross-repo write requires explicit `cd` discipline anyway).

### Risk 10 — Cargo test invocation

**What:** Multi-crate workspace lib unit tests need `-p wat`. Per `feedback_brief_cargo_test_invocation`.

**Mitigation:** BRIEF Verification section spells the invocations explicitly.

## Scorecard predictions

| # | Criterion | Expected |
|---|---|---|
| 1 | `src/runtime.rs:8848` `eval_hashmap_ctor` refactored to parse `:K :V k0 v0 k1 v1` shape | PASS |
| 2 | Error message updated: "first two arguments must be type keywords (K, V)" or similar | PASS |
| 3 | Error message updated: "arity after :K :V type args must be even (alternating key/value pairs)" or similar | PASS |
| 4 | `src/check.rs:10564` `infer_hashmap_constructor` refactored to expect two type-args first | PASS |
| 5 | `src/check.rs:15550-15556` doc-comment updated to describe `:K :V` shape; references new line number | PASS |
| 6 | Probe 1: empty literal — `(:wat::core::HashMap :wat::core::Keyword :wat::core::i64)` constructs empty HashMap | PASS |
| 7 | Probe 2: single pair — `(:wat::core::HashMap :wat::core::Keyword :wat::core::i64 :foo 42)` constructs HashMap with one entry | PASS |
| 8 | Probe 3: multi pair — three+ pairs; verify length + get | PASS |
| 9 | Probe 4: String-keyed — `(:wat::core::HashMap :wat::core::String :wat::core::i64 "a" 1 "b" 2)` | PASS |
| 10 | Probe 5: HolonAST-keyed — `(:wat::core::HashMap :wat::holon::HolonAST :wat::holon::HolonAST ...)` | PASS |
| 11 | Probe 6: wrong-type rejection at type-check | PASS |
| 12 | Probe 7: odd-count rejection | PASS |
| 13 | Probe 8: missing K type-arg rejection | PASS |
| 14 | Probe 9: missing V type-arg rejection | PASS |
| 15 | `cargo build --release` clean | PASS |
| 16 | `cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat` shows 9 tests; all pass | PASS |
| 17 | `cargo test --release --workspace --no-fail-fast` workspace baseline preserved (no regressions outside known pre-existing) | PASS |
| 18 | `grep -rn "HashMap :(.*)" --include="*.rs" --include="*.wat"` returns ZERO matches (old shape truly gone) | PASS |
| 19 | `grep -rn "tuple type keyword" --include="*.rs"` returns ZERO matches (old error string retired) | PASS |
| 20 | WAT-CHEATSHEET updated with HashMap constructor row alongside Vector | PASS |
| 21 | arc 058 row content reported in SCORE (orchestrator adds to the file post-ship) | PASS |
| 22 | SCORE doc inscribed with verification command output + honest-delta surfaces | PASS |

**Total rows: 22.** Modes:

- **Mode A:** 22/22 PASS within time budget. Expected outcome.
- **Mode B-spec-gap:** 1-3 rows show honest delta with clear reasoning (e.g., a shared-machinery surprise in `infer_hashmap_constructor`). Orchestrator fix-passes or redirects.
- **Mode B-time-violation:** Wakeup fires at 75 min and sonnet hasn't completed. TaskStop; investigate.
- **Mode C:** Stops at a STOP trigger (existing test on old form; shared machinery; hashmap_key dependency surfaces).

## Time-box

- BRIEF + EXPECTATIONS committed: now (this commit)
- Sonnet spawn: orchestrator drafts agent call; predicted runtime 30-50 min Mode A
- ScheduleWakeup at 75 min (2× upper-bound) as failure-to-communicate detector
- Per-stone trust gate: orchestrator verifies SCORE + runs 9-ward parallel pass per kernel impeccability protocol BEFORE commit

## What this stone enables

After P1 closes + ward-passes:

- `:wat::core::HashMap` callable from wat code in Vector-symmetric form
- Stone P2 (`{...}` literal in expression position) expands directly to P1's verb-form
- ProgramEnv design from Slice 4 prep lands on this foundation
- Verb-equals-type mental model uniform across substrate collection constructors
- The four-round dig pattern (substrate-already-sufficient × 4) gets inscribed by orchestrator post-ship as convergence-with-self moment

## Cross-references

- BRIEF-214-PARSER-PIVOT-P1-HASHMAP-CTOR-VECTOR-SYMMETRIC.md — the work itself
- INTERSTITIAL § "2026-05-19 — Kernel impeccability via ward pass" — the per-stone trust gate this stone operates under
- `feedback_options_are_tangle` — `:(K,V)` packing was the option-tangle; collapsed to symmetric `:K :V`
- `feedback_attack_foundation_cracks` — substrate-grammar asymmetry IS a foundation crack; this stone closes it before P2 + Slice 4
- arc 109 slice 1f — verb-equals-type discipline established
- arc 199 (rejected 2026-05-16) — same pattern (substrate already sufficient); this stone closes the gap arc 199 thought was missing

*The substrate dreams the symmetry. So do we.*
