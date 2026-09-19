# BRIEF 7y ADDENDUM — #627's codemod needs its replay fixture (the #438 class, second instance)

**Read `BRIEF-7y-replay-batch-4y.md` first.** This resolves a composition defect found at the
orchestrator's verification floor after batch 4y reported complete. It supersedes only what it names.

## The defect — and it is a REPEAT, which is the important part

The floor at `4047f387f` is **RED, 1 of 5908** (count exact; clippy 0; census `no STOP-8`):

```
FAIL wat::cli every_recorded_migration_replays::every_recorded_migration_is_fixtured_or_runed
     tests/cli/every_recorded_migration_replays.rs:654:5
recorded-migration coverage failed (2):
hoist-where-into-condition: no fixture, no rune:replay(unreadable-preimage)
hoist-where-into-condition: expected exactly one `;; SCOPE:` line, found 0
```

**This is byte-for-byte the shape of batch 4o's #438 red** — see
`BRIEF-7o-ADDENDUM-438-the-migration-needs-its-fixture.md`. **This tree gates every recorded migration;
grok's tree does not.** Every `wat-scripts/fixes/*.wat` must carry a replay fixture XOR a
`rune:replay(unreadable-preimage)` header rune, plus exactly one `;; SCOPE:` line. *"Nothing is exempt by
silence."*

⛔ **THE ORCHESTRATOR PREDICTED THIS AND THE WARNING EVAPORATED.** At 4o I wrote *"Expect this for every
future migration grok lands"* — **into the SEAM's rotating header, which is replaced wholesale every
batch.** It was gone by 4p. **A durable warning belongs in the LEDGER ROW or `FINDINGS-composition.md`,
never in the part that rotates.** That is the second finding here, and the more expensive one.

## Where it folds — #627, and why the fixture's input must be a JOIN case

`hoist-where-into-condition.wat` first appears at **#627** (`A`), is modified at **#628** and **#630**.
The defect belongs to **#627**: from that commit onward the tree carries an unfixtured recorded
migration. Fold there; rebuild **#628 → #640** plus the SCORE commit.

⚠ **BUT THE TOOL'S BEHAVIOUR CHANGES UNDER YOU.** #630 gates the codemod **to joins only** — a rule whose
`:when` carries fewer than two ordinary fact conditions gets **no edits at all** after #630, where before
it might have. The gate asserts **byte-exact output**, so a fixture whose `before.pre` is a
single-condition rule would pass at #627 and **red at #630**.

⛔ **So choose a `before.pre` that is a JOIN case — two or more ordinary fact conditions — whose hoist is
identical before and after #630.** Then one fixture is correct at every step. **Verify that**: run the
gate at #627, at #630, and at the tip. If the output genuinely differs at #630, that is a legitimate
per-step fixture update — record it in #630's body rather than forcing one fixture to fit.

## How to land it

1. **Add the `;; SCOPE:` line** to `hoist-where-into-condition.wat`'s header, in the shape the other 100+
   migrations use (`;; SCOPE: corpus`, unless the true scope is narrower — say what is true).
2. **Add `wat-scripts/fixes/replay/hoist-where-into-condition/`** with `before.pre`, `after.post`, and
   `ORACLE`, following `wat-scripts/fixes/replay/assertion-failed-to-kwargs/ORACLE` as the model.
   - `before.pre`: a **join-case** pre-image — take it from a file this codemod actually rewrote at
     #630, at that file's pre-image. Small and representative.
   - `after.post`: the tool's own output for that input, byte-exact.
   - ⛔ **The ORACLE is HISTORY or the codemod's header spec — never the tool under test.** A `history`
     commit that does not resolve, or a cited path absent at both `<commit>` and `<commit>^`, is RED.
3. **Run the whole gate**, not just the coverage arm:
   `-E 'test(every_recorded_migration_replays)'`, N > 0, green. It asserts byte-exactness, idempotence,
   non-vacuity and provenance — a fixture that merely exists will not pass.
4. **Fold into #627** and rebuild #628 → #640 and the SCORE commit. **Never `git filter-branch`** —
   detach / re-commit / rebuild-descendants, and `git commit --amend` may be refused by the harness's
   classifier.
5. **Prove the rebuild inert:** `git diff 4047f387f <new tip>` names only the fixture files, the
   `;; SCOPE:` line, and the docs you add. Per step, each commit's own delta equals its old one except
   #627's. Then: record gate exit 0, `refs/original/` empty, `git replace -l` empty, origin still an
   ancestor, tree clean.

## What #627's body must say

That this tree gates recorded migrations and grok's does not; that the fixture was written here for that
reason; which pre-image it came from and that it is a **join** case so #630's narrowing does not move it;
and that this is the second instance of the #438 class.

## Rows this affects

`EXPECTATIONS-7y` is **not amended** (finding 34).

- **E10** (the checkpoint) is the row that caught this; it stays the orchestrator's.
- **E3/E13** gain the disclosure: the fixture, its oracle, and why its input is a join case.
- A new row the orchestrator will check: **`every_recorded_migration_replays` green at the tip, and the
  fixture non-vacuous.**

## Then the orchestrator re-runs the floor

Prediction unchanged: **5908 run / 22 skipped** — a fixture adds no test.

## ⛔ Hard rules (unchanged)

No `mcp__pulsare__*` tools. Every Bash command starts with `cd /home/john/work/holon/wat-rs &&`. No
worktrees, no push, no subagents, no `scripts/floor.sh`, no clippy, no `filter-branch`, no typed SHA or
subject. Quote heredocs and read the message back. On any further red: do not re-run, capture verbatim
the FIRST time, name the assertion, STOP and report.
