# REFUTE — 0b batch 2c: an oracle's provenance is CHECKED, not claimed

Batch 2 (`1938294c7`) is green: gate 18/18, floor 5391/5391, clippy 0. It is pushed. The orchestrator
re-ran each item below and **keeps** it:
- **the 7 gaps are closed.** Read in the fixture diffs.
- **the 5 runes are honest.** Each reproduces with rc=2 at its quoted byte offset. Each tool loads on
  readable input (rc=0). All five match INSIDE `<…>`, so their input grammar IS angle brackets.
- **the repairs.** The `to-faithful` inlines equal the original helper bodies verbatim.
  `mandate-request-malformed` updates the syntax of its emitted type, not its behaviour.
  `rename-wat-record-to-core-record`'s call equals its landing version.
- **the ledger's deletion, arms (e)–(h), and the shard layout** (`positional_ctor` measured at 63.0 s
  under the floor, against a 120 s kill).

Four things fail, and all four are yours to fix.

## Finding 1 — 38 oracles appear NOWHERE in history, and none says so

The brief said history first, and to say which fixtures fell back to spec. `SCORE-0b-batch-2.md`
gives no per-fixture source. It says *"history or header spec as cited in 2a/2b and the fixture
comments"*, but the fixture comments are near-miss notes and the commits cite nothing per fixture.

Measured (`bootstrap/step0-probes/b2-oracle-history-search.txt`): for 38 fixtures, the most
distinctive changed `after.post` line never appears in any `.wat` in history (`git log -S`). They
are hand-written minimal forms.
- **35 of the 38 had history available:** their landing commit changed `.wat` files
  (`bootstrap/step0-probes/0b-oracles.txt`).
- **3 had none:** `rename-sourcefile-to-source-file`, `sweep-lint-fixes`, `to-faithful-clojure-net`.
- The list is in `bootstrap/step0-probes/b2-38-no-history.txt`.

A spec oracle written by the agent that also ran the tool cannot be told apart from one written
after watching the tool's output. The oracle rule exists to prevent exactly that. This is the second
time this session that a prose citation failed, so the fix climbs the ladder: **the gate checks
provenance.**

## The fix — an `ORACLE` file per fixture, verified by the gate

Four questions (Obvious / Simple / Honest / Good UX):
- **Prose citations in SCOREs (today):** Y / Y / **N** (it failed: 38 uncited) / **N**.
- **The orchestrator re-checks by hand every batch:** Y / Y / **N** (it depends on memory, and two of
  its own pattern checks were wrong this session) / **N**.
- **`wat-scripts/fixes/replay/<stem>/ORACLE`, verified by the gate:** Y / Y / Y / Y.

`ORACLE` holds one entry per line:
```
history <commit> <path> [<path> …]
spec <the after.post line, verbatim> :: <a verbatim substring of a comment line in the codemod's header>
```

The gate's new arm, in `every_recorded_migration_is_fixtured_or_runed` or a sibling fn:
- **A fixture with no `ORACLE`, or an unparseable entry, is RED.**
- **`history` entries:** every CHANGED line of `after.post` must be found in the union of
  `git show <commit>:<path>`. Every CHANGED line of `before.pre` must be found in the union of
  `git show <commit>^:<path>`. "Found" means whitespace-normalized substring. Changed lines come from
  a line diff of `before.pre` against `after.post`; unchanged lines (near-misses) are not checked.
- **`spec` entries:** a changed `after.post` line not found in the history union must be named by a
  `spec` entry, and its quote must occur in a `;;` comment line of `wat-scripts/fixes/<stem>.wat`.
  Otherwise RED, naming the stem and the line.
- **Prove the arm can fail before trusting it.** Each goes RED, is reverted, and its verbatim text
  goes in the SCORE:
  - (i) a `history` line citing a commit that lacks the line;
  - (j) a `spec` quote that is not in the header;
  - (k) a fixture with no `ORACLE`.

**Backfill all 90.** The 13 from 0a and 16 from batch 1 carry citations in their SCOREs: transcribe
them, and the gate now verifies them. For the 38, see the next section.

## Finding 1's 38 — replace with history where it exists

- **For the 35 whose landing commit changed `.wat`:** rebuild the fixture from whole top-level forms
  that commit changed ONLY by this codemod, then cite them as `history`. If a changed form's hunk
  mixes other hand edits, choose another form, or another commit that applied the codemod
  (`git log -S'<a token it rewrites>'`).
- **Spec is allowed only where no such form exists anywhere in history.** Then the `spec` entries
  say so, and the SCORE names why history failed for that stem.
- **For the 3 with no `.wat` in their landing commit:** look for a later commit that applied the
  codemod. If there is none, use spec.

## Finding 2 — the `to-faithful-clojure` pair's fixtures exercise almost nothing

Both fixtures are the same trivial `defn`/`if` form. It is uncited, and it names no near-miss.
- `to-faithful-clojure-net` has 12 rules: `g1-keyword`, `g2-symbol`, `g3-genuine`, `g4-namespaced`,
  `g5-type-shaped`, `g6-arrow`, `g7-post-arrow`, `tc-from-shaped`, `tc-from-postarrow`,
  `t1-head-conv`, `t2-type-conv`, `t3-arrow-conv`.
- `to-faithful-clojure-rete` has 3: `head-keyword->conv`, `arrow->conv`, `type-keyword->conv`.
- Each rule gets a fixture line that fires it, with a near-miss, and an `ORACLE` entry (history from
  `83b291f9b` for rete if it has forms, else spec).

## Finding 3 — four dead helpers left by the repair

`:fix::has-ns?` and `:fix::type-shaped?` (net), and `:fix::head-keyword-str?` and
`:fix::type-shaped-keyword-str?` (rete), are each defined once and now referenced nowhere. Delete
them. A dead thought costs every future reader.

## Finding 4 — `rename-wat-record-to-core-record.wat:7,12,14` still lie

The code was restored, but lines 7, 12 and 14 still read *"The old `:wat::core::Record` symbol ceases
to exist"* and *"The prefix `:wat::core::Record` IS the full name …"*. The prose was corrupted by the
codemod's own corpus application, `dedcb74a7`. Restore those lines to their text at the landing
commit `086321255` (`git show 086321255:wat-scripts/fixes/rename-wat-record-to-core-record.wat`,
lines 6–14). Comment-only.

## EXPECTATIONS (batch 2c), fixed before the strike

| # | what | expected |
|---|---|---|
| C1 | every fixture has an `ORACLE` | `ls wat-scripts/fixes/replay/*/ORACLE \| wc -l` == 90 |
| C2 | the new arm verifies them | the gate passes; the arm is shown RED (i), (j), (k), with verbatim text |
| C3 | history where it exists | of the 38, the SCORE lists which became `history` (commit + path) and which remain `spec` (with why history failed). The orchestrator re-runs `git log -S` on every remaining `spec` line |
| C4 | the `to-faithful` coverage | 12 + 3 rules, each mapped to a fixture line with a near-miss |
| C5 | the dead helpers are gone | `grep -c 'defn :fix::\(has-ns?\|type-shaped?\|head-keyword-str?\|type-shaped-keyword-str?\)'` == 0 in both files |
| C6 | the prose is restored | lines 6–14 equal `086321255`'s |
| C7 | floor / clippy | 0 failed · 0 lines. The per-shard times are reported again |

## Tier

Commit on green. **Do not push. Do not touch main.** Yield once, with `SCORE-0b-batch-2c.md`.
