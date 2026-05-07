# Arc 145 — Typed Let Substrate EXPECTATIONS (sweep 1a)

**Drafted 2026-05-06.** Pre-handoff scorecard for sweep 1a of arc
145's typed-let work.

**Brief:** `BRIEF-SUBSTRATE.md`
**Output:** EDITS to `src/check.rs` + `src/runtime.rs` +
`src/special_forms.rs` + NEW `tests/wat_arc145_typed_let.rs`.
NO consumer-side wat edits. NO commits.

## Setup — workspace state pre-spawn

- Last commit: `268526a` (arc 119 closure)
- Workspace clean (0 failed across all crates)
- Working tree clean
- DESIGN locked at `docs/arc/2026/05/145-typed-let/DESIGN.md`
  with Q1 (HEAD position) + Q2 (REQUIRED) resolved
- Predecessor: arc 108 (typed expect — `-> :T` precedent for
  value-bearing special forms); arc 144 slice 2 (special-form
  registry sketches that need updating)

## Hard scorecard (10 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | File diff scope | EXACTLY 4 files modified: `src/check.rs`, `src/runtime.rs`, `src/special_forms.rs`, NEW `tests/wat_arc145_typed_let.rs`. NO wat substrate edits. NO consumer test file edits. NO other crate. |
| 2 | `infer_let` requires `-> :T` at HEAD | `args[1]` checked for `->` Symbol; `args[2]` parsed as Keyword type expr. Untyped form emits MalformedForm with migration-hint reason. Pattern mirrors `infer_match`. |
| 3 | `infer_let_star` requires `-> :T` at HEAD | Same shape as #2 but sequential semantics preserved (cumulative env walks bindings). |
| 4 | Body type unification | Body's inferred type unifies with declared `:T`; mismatch surfaces a clean TypeMismatch at body's position. |
| 5 | Runtime: eval_let + eval_let_star skip `->` + `:T` tokens | The `-> :T` token sequence is a no-op at eval layer. Body still evaluates correctly; bindings still bind correctly. Tail-call paths + incremental-step paths consistently handle the new args layout. |
| 6 | Special-form registry updated | `src/special_forms.rs` let/let* sketches show required `-> :T` slot at HEAD. Sketches consumable by arc 144's reflection trio. |
| 7 | New test file: 6-10 unit tests | `tests/wat_arc145_typed_let.rs` covers: typed parallel pass, typed sequential pass, type mismatch parallel, type mismatch sequential, untyped parallel parse error, untyped sequential parse error, nested typed let, tail-call sanity, sequential binding visibility, reflection round-trip. All pass. |
| 8 | Migration-hint reason text | The MalformedForm reason for untyped form contains "now requires `-> :T`" substring (sonnet's choice on full text but must mention the migration). Mirrors `infer_match`'s migration-hint pattern. |
| 9 | Workspace consumer failures shape (Mode A predicted) | `cargo test --release --workspace` shows MANY consumer test failures with MalformedForm errors carrying the migration-hint substring. NOT panics. NOT type errors at substrate boundary. NOT parser errors inside the new substrate code. EXACTLY the sweep-1b workload (every existing let/let* call site fails until 1b adds `-> :T`). |
| 10 | Honest report | Per BRIEF reporting requirements (pre-flight crawl, edit summary, LOC delta, verification, path classification, honest deltas). |

**Hard verdict:** all 10 must hold. Rows 2 + 3 + 8 are the
load-bearing rows (substrate semantics + migration-hint
pattern fidelity).

## Soft scorecard (4 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 11 | LOC delta | 150-400 LOC across the 4 substrate files + new test file. >500 = re-evaluate (probably scope creep). |
| 12 | Pattern fidelity to infer_match | Migration-hint detection shape matches `infer_match` verbatim where applicable; reason text mentions migration. |
| 13 | clippy clean | No new clippy warnings. |
| 14 | No-grinding discipline | Sonnet does NOT modify consumer wat files to silence migration-hint MalformedForms. Does NOT add backwards-compat. STOP at first red and report. |

## Independent prediction

- **Most likely (~70%) — Mode A clean.** `infer_match` is the
  canonical pattern; mechanical adaptation to let/let*. Eval-layer
  changes are simple (skip 2 args). Tests follow the pattern.
  ~20-30 min wall-clock.
- **Mode B-substrate-internal-bug (~15%):** an edge case in
  tail-call paths or step_let_star args layout that the brief
  didn't anticipate. Honest STOP + report.
- **Mode C-unexpected-failure-shape (~10%):** consumer failures
  surface as something OTHER than MalformedForm migration-hint
  (e.g., panic, runtime error, type-system interaction). Surface
  the gap.
- **Mode D-special-form-registry-edge (~5%):** arc 144's sketch
  format doesn't accommodate the new `-> :T` slot cleanly
  without registry refactor. Surface as honest delta.

## Time-box

60 minutes wall-clock (2× predicted 30-min upper-bound). Mode
B-time-violation if wakeup fires + sonnet still running.

## What sonnet's success unlocks

**Mode A clean**: sweep 1b (consumer migration) runs immediately
after — sweep 1a's substrate change made every existing call site
fail with MalformedForm; sweep 1b's job is to add `-> :T` to all
those sites. Atomic commit when workspace = 0-failed.

**Mode B/C/D**: surface gap; orchestrator adjusts brief; reland.

## After sonnet completes

- Read this file FIRST.
- Score each row of both scorecards explicitly.
- Verify the migration-hint MalformedForm shape via a sample of
  consumer failures (read 2-3 actual error messages).
- Run `cargo test --release --test wat_arc145_typed_let` to
  confirm the new tests pass.
- **DO NOT COMMIT YET** — sweep 1b runs next; commits happen
  atomically together.

## Why this matters

User direction 2026-05-06: "typed let then do" — sweep 1a is the
substrate side of typed let; sweep 1b ships consumer migration;
slice 2 closes paperwork. After arc 145 closes, arc 136 (do form)
spawns next.

The mutual-agreement chain:
- User → Orchestrator: "typed let then do" + (2026-05-03 evening)
  "the ret val of a let statement /must be declared/"
- Orchestrator → Sonnet (this brief): substrate change + tests;
  REQUIRED `-> :T` at HEAD; migration-hint pattern fidelity to
  infer_match
- Sonnet → Reality: substrate ships; consumer failures match
  expected MalformedForm shape; sweep 1b queued

Mode A clean = the typed-special-form discipline (arc 108) extends
to let/let*; the substrate becomes consistent across all
value-bearing forms.
