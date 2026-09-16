# BRIEF 7g — replay batch 4g: grok-rete #261 → #280

Anchor: `/home/john/work/holon/wat-rs`, branch `replay/grok-rete`. Verify with `pwd` and
`git rev-parse --abbrev-ref HEAD`. Expect 260 REPLAY commits and a clean tree.
⛔ Do NOT touch `/home/john/work/holon/` (FROZEN) or `main`. **Never use worktrees. Never push. Do not
spawn subagents. Do NOT run `scripts/floor.sh`, clippy or run5** — the orchestrator weighs those centrally
and uncontended; a gate run while you work in the tree is a FALSE result.

⚠ `wat-rs/CLAUDE.md` does not reach an executor. The load-bearing doctrine is carried here.

## Doctrine — each line was paid for

- **R21:** `.wat` corpus rewrites go through a recorded wat-fix codemod (`wat-scripts/fixes/*.wat`), NEVER
  hand edits or sed. Scratch `.wat` → `wat-scripts/scratch-pad/`.
- ⛔ **WAT EMBEDDED IN `.rs`/`.sh` STRING LITERALS IS THE EXCEPTION — AND THIS REPLAY'S MOST PERSISTENT
  DEFECT SOURCE** (finding 33). No codemod reaches it, `convert.sh` never sees it, no gate parses it. It has
  bitten #162 (7 sites), #167 (a `perl` substitution broken for weeks), and #238. **When a step's subject
  mentions a rename or rehome, grep the `.rs`/`.sh` side too.** Hand-fix and LOG each edit.
- ⛔ **A rename census must be built from the RECORDED MIGRATIONS, not the shape of the name** (finding 33).
  `rete::core::{i64,f64,string}` were rehomed; `rete::core::keyword::=` was deliberately NOT — it is a live
  `#[wat_special_form]`. Check `wat-scripts/fixes/rename-*.wat` before calling a spelling stale.
- ⛔ **NEVER pipe a gate through `head`/`tail`/`grep` to decide anything — redirect to a file and read the
  file** (finding 28).
- ⛔ **THERE IS NO KNOWN FLAKE.** On any red: do NOT re-run; copy the whole stdout+stderr block verbatim;
  name the exact assertion; STOP and report.
- ⛔ **Ending your turn ENDS you.** Every verification in the FOREGROUND, blocking.
- ⛔ **After ANY commit or `--amend`: assert `git status --porcelain` is EMPTY and that the commit's diff
  names every path its body claims** (finding 27). Read every COUNT off `git show --stat`/the diff.
- ⛔ **ANY REPAIR MUST BE VISIBLE TO `push`** (finding 29). No `git replace`, no overlay. If blocked, STOP.
- ⛔ **A mutation proof must falsify the proposition you rely on** (finding 30). `rc=1` says nothing about WHY.
- ⛔ **Each verdict line must be ON ONE LINE** (finding 31) — the gate matches within a single line.
- ⛔ **A PREDICATE CAN BE WRONG IN BOTH DIRECTIONS.** Validate any census pattern two-sided, with a control
  that MUST come back negative. The orchestrator burned four patterns on this batch's own analysis: a
  classifier that could not tell `violations.is_empty()` (a VERDICT) from `!tracked.is_empty()` (a GUARD)
  produced a sizing wrong in both directions. **When a third pattern fails, stop patterning and READ.**
- **Every step, docs-only included**, commits as `REPLAY(grok-rete #N): <C's subject>` with the
  `(cherry picked from commit <sha>)` trailer. BRIEF-1's item 1 is correct as written — follow it.

## The work

Replay **#261 → #280** (20 steps) per `BRIEF-1` § "One step". **Fifteen are docs-only:**

    261 263 264 265 267 268 269 271 272 273 275 276 277 279 280

**Five carry code:**

    262 266 270 274 278

That is 15 + 5 = 20. **Count both lists against the census rather than trusting this sentence** — the
orchestrator shipped "twelve" over a list of fourteen into BRIEF-7f, an hour after recording finding 27,
which is the rule that a count must be READ OFF THE DATA and never written from recollection.

| N | C | kind | files | note |
|---|---|---|---|---|
| 262 | `c0c883082` | code | 15 (6 `.wat`, 4 `.rs`, 5 `.edn`) | ⚠ **modifies two files created at #258** — see below |
| 266 | `452953cb9` | shared | 7 (7 `.rs`) | all 7 present; the ceiling set becomes a closed type |
| 270 | `76e221bbb` | shared | 2 (2 `.rs`) | **ADDS `tests/lint/no_new_broken_doc_link.rs`** — #274 then modifies it |
| 274 | `58a10e1f8` | shared | 20 (20 `.rs`) | ⚠ **THE BATCH'S TRAP — a new meta-gate that WILL go red. See below.** |
| 278 | `f4800ef97` | shared | 7 (5 `.wat`, 1 `.rs`, 1 `.md`) | ⚠ **absent-on-main row + three codemod EDITS** |

**No stdlib-touch row and no `future-macro-changes.txt` entry in this range.** The next two-phase stdlib
step is #377, so nothing here needs the door's stdlib mode. `wat/` is untouched by all twenty steps.

## ⚠ #262 — it modifies two files that batch 4f creates at #258

`tests/rete/probe_arc278_field_span.rs` and `tests/rete/probe_arc278_field_span_nested.wat` are **absent
from the tree today** and are **added by grok step #258**, inside batch 4f. By the time you run, #258 has
landed, so this is NOT a hazard — but it IS a tripwire: if #262 reports "no such file", #258 did not land
correctly and the fault is upstream of you. **STOP and report rather than creating the files yourself.**
`absent-on-main.tsv` has no row for #262, which rules out a main-side deletion as the cause.

## ⚠⚠ #274 — A NEW GATE LANDS AND IT WILL GO RED. THIS IS EXPECTED. REPAIR IT AT #274.

#274 adds `tests/lint/every_walking_gate_declares_non_vacuity.rs` (496 lines) and retrofits declarations
into 18 existing gates. **It walks `tests/lint/` at RUNTIME — no hardcoded list, no allowlist, no exemption
mechanism.** Any `tests/lint/*.rs` containing `read_dir` or `Command::new` is IN SCOPE and must declare how
it knows its walk was not empty, in one of two forms:

1. a `// NON-VACUITY: <reason>` plain-comment line with a real assertion **within 12 lines below it**, or
2. a `// rune:lint(vacuity-guard) <reason>` plain `//` comment (NOT `///`, NOT `//!`).

**Grok's #274 tree has 32 lint gates. Ours has 43.** The 13 gates main owns and grok has never seen
received no declaration, so the gate lands red. **Measured in advance, on this tree:**

- `assert!(undeclared.is_empty())` fires with **9** gates.
- `assert!(hollow.is_empty())` fires with **1** gate — a SECOND, independent arm.
- The positive control `no_ceiling_raise_in_rete.rs` is healthy (in scope, two properly-guarded markers).
- Corpus floors pass comfortably: 43 files (needs ≥ 25), 32 in scope (needs ≥ 18).

**THE REPAIR SET — ten files, all under `tests/lint/`:**

    every_tracked_wat_parses.rs              holon_is_vsa_only.rs
    ignore_reason_justified.rs               nested_program_starts.rs
    no_bare_is_err.rs                        no_bootstrap_path_in_committed_rust.rs
    no_error_flattening_helper.rs            one_variant_separator.rs
    tracked_wat_dir_is_stdlib_sources.rs     every_ungated_wat_checks.rs   <-- the HOLLOW one

**This is the #184 precedent: a gate introduced at step N repairs the rot it reveals AT N.** Do NOT fold
this backward into whichever earlier step added each gate. Do NOT weaken, allowlist or skip the meta-gate.
These 13 gates genuinely do not say how they know they reached something — that is a real flaw of ours,
and the gate is right to catch it. **We do not leave known flaws.**

**For each of the ten, OPEN THE FILE AND READ IT**, then do one of two things:

- **A guard already exists** → put a `// NON-VACUITY: <why an empty walk would be a lie here>` comment
  within 12 lines above it. Confirmed guards, read directly (use as starting points, verify each yourself):
  `every_tracked_wat_parses.rs:35` (`paths.len() > 1000`), `holon_is_vsa_only.rs:355` (`files.len() > 50`),
  `nested_program_starts.rs:565` (`total >= 141`), `tracked_wat_dir_is_stdlib_sources.rs:53`
  (`!tracked.is_empty()`), `every_ungated_wat_checks.rs:46` (`!paths.is_empty()`).
- **No guard exists** → **AUTHOR one.** A gate whose only assertion is `violations.is_empty()` passes when
  its walk finds nothing at all. Assert the discovered count against a floor derived from the tree, and say
  in the comment what the floor means. Do not invent a number you did not measure.

⛔ **`every_ungated_wat_checks.rs` is HOLLOW, not undeclared, and the difference matters.** Its line 20
carries `//! ⛔ NON-VACUITY IS MANDATORY … a wall that cannot fail is a claim, not a check` — correct
doctrine, written as module prose 26 lines above the guard at `:46`. The gate reads a marker with no
assertion under it as standing IN PLACE OF a guard. **Do not delete that prose.** Add a proper
`// NON-VACUITY:` line directly above `:46`.

⛔ **The orchestrator's split between "add a marker" and "author a guard" is NOT reliable — it was produced
by a pattern that proved wrong in both directions, twice.** The ten filenames are solid; the per-file
verdict is yours, from reading. Report what you actually found, including any file where our count differs.

## ⚠ #278 — an absent-on-main row, and three codemod EDITS that are EXPECTED

**(a) The probe has no landing site.** #278 adds a 4-line rune to
`wat-scripts/scratch-pad/probe-f64-comparator-bogus-head.wat`. **That path does not exist here.** Main
renamed it (R055) to `tests/resolve/probe_arc255_the_blanket_hides_a_phantom_head__bogus_rete_head.wat`,
which is present and still calls `:wat::rete::f64::>X` at line 36. `absent-on-main.tsv` carries the row.

- ⛔ **Do NOT recreate the file at the dead path.** Main moved it deliberately.
- ⛔ **Do NOT add the rune to the `tests/resolve/` copy either.** The new #278 gate walks `wat-scripts/`,
  `src/` and `wat/` — **never `tests/`** — so a rune there would be inert, and an inert rune advertises a
  watcher that does not exist (finding 30's shape). Drop that hunk and LOG it.
- **Nothing is lost by dropping it.** That probe is the *mint's* negative control (proving `--check`
  refuses a typo'd head), and it is alive at its new home. The rune exists only to stop the new gate
  reading the deliberate absence as rot; with the file outside the walk, the rune has no job.

**(b) It EDITS three recorded codemods** — `rete-oracle-sigil.wat`, `rete-where-per-type-spelling.wat`,
`type-query-to-defquery.wat` — all present. Every batch from 4b to 4e scored "no `wat-scripts/fixes/` edit"
as a PASS condition; **#278 breaks that invariant on purpose** and is the ONLY step in #261–#280 that
touches `wat-scripts/fixes/`. Do not treat it as a STOP. It deletes two phantom rename rows (41 pairs → 39)
and adds per-name runes. Nothing counts runes, so our tree carrying one fewer than grok's reddens nothing.

**(c) `CLAUDE.md` applies clean** — verified: our copy is byte-identical to grok's #278 parent.

## ⚠ The path-based verdict-line rule

The record gate derives requirements from each commit's OWN diff: a `.wat` or `src/` change needs
`census: … --diff no STOP-8` and `nested-program-gate: PASS`; a `.rs` change needs `lint-subset`,
`kind(lib)` and `doctest`. Misjudging this cost a record repair at #215. Note #262 carries `.wat`, `.rs`
AND `.edn`; #274 and #266 and #270 are `.rs`-only; #278 carries `.wat`, `.rs` and `.md`.

## ⛔ STOP triggers — rejections, not permission to defer

STOP-1 … STOP-13 per BRIEF-1; in particular **STOP-8** (a file going rc 0 → non-zero the step did not
produce), **STOP-10** (stone 3's nested-program gate), **STOP-11** (`kind(lib)` or doctests red).
**#274's meta-gate going red is NOT a STOP — it is the step's declared work.** A STOP there is only
warranted if the repair cannot be made honestly.

## The fold rule

A composition defect belonging to an earlier step FOLDS into that step; rebuild the steps after it. **Never
a repair commit after the batch, never a knowingly-red REPLAY commit.** A fold's proof must name WHICH
paths it carries, checked with `git diff <old-tip> <new-tip>`. **A gate that did not exist at step N is not
a reason to fold into N — the #184 precedent — and #274 is exactly that case.**

## Tier

Commit each step on green. **Do not push.** Before yielding, run in the FOREGROUND and paste:

    scripts/replay/verify-step-record.sh <BATCH-START-SHA> HEAD 261 280

It must exit 0. Yield after #280, or at the first STOP, with `SCORE-7g-replay-batch-4g.md` and a
`REPLAY-LOG.md` section, leaving the tree CLEAN.
