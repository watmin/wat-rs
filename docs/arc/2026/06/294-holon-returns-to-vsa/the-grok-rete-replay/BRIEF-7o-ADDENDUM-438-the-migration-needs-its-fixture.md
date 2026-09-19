# BRIEF 7o ADDENDUM — #438: a recorded migration must carry its replay fixture

**Read `BRIEF-7o-replay-batch-4o.md` first.** This addendum resolves a composition defect found at the
orchestrator's verification floor, after batch 4o reported complete. It supersedes only what it names.

## The defect

The floor at `e8f4d08a8` is **RED, 1 of 5856** (the count matched the locked prediction exactly; clippy 0;
census `no STOP-8`):

```
FAIL wat::cli every_recorded_migration_replays::every_recorded_migration_is_fixtured_or_runed
     tests/cli/every_recorded_migration_replays.rs:654:5
recorded-migration coverage failed (2):
wrap-session-facts-in-factbag: no fixture, no rune:replay(unreadable-preimage)
wrap-session-facts-in-factbag: expected exactly one `;; SCOPE:` line, found 0
```

**One cause.** #438 lands a new recorded migration, `wat-scripts/fixes/wrap-session-facts-in-factbag.wat`.
**This tree gates every recorded migration; grok's does not.** Our gate demands, for each
`wat-scripts/fixes/*.wat`: a replay fixture XOR a `rune:replay(unreadable-preimage)` header rune, and
exactly one `;; SCOPE:` line. In its own words: *"Nothing is exempt by silence."*

**This is finding 38's family**: a main-only artifact — here a main-only GATE — meets a file a replayed
step legitimately adds. It is not grok's defect and not a landing error; it is the seam.

## Why a fixture and not the rune

Measured: **100 of the 106 recorded migrations carry a fixture**; the rune is the rare escape for a
pre-image that cannot be read. This migration's pre-image is perfectly readable — we ran it over 15 files
this batch. **Write the fixture.**

## The cure — fold into #438

1. **Add the `;; SCOPE:` line** to `wrap-session-facts-in-factbag.wat`'s header, in the shape every other
   migration uses (`;; SCOPE: corpus`, unless the true scope is narrower — say what is true).
2. **Add `wat-scripts/fixes/replay/wrap-session-facts-in-factbag/`** with:
   - `before.pre` — a real pre-image. Take it from a file this migration actually rewrote, at its
     pre-image: either grok's own `09e3d912c^` or this tree's `#438^`. Keep it small and representative,
     and make sure it exercises **both** uniform wraps the header documents (the `Session/facts` read AND
     the `:facts` value inside a `Session` constructor) — otherwise the fixture is vacuous on half the
     tool, and the gate checks non-vacuity.
   - `after.post` — the tool's output for that input, byte-exact.
   - `ORACLE` — provenance, in the established shape. Read
     `wat-scripts/fixes/replay/assertion-failed-to-kwargs/ORACLE` as the model. ⛔ **The oracle is HISTORY
     or the codemod's header spec — never the tool under test.** A `history` commit that does not resolve,
     or a cited path absent at both `<commit>` and `<commit>^`, is RED.
3. **Run the gate** — `-E 'test(every_recorded_migration_is_fixtured_or_runed)'` and the whole
   `every_recorded_migration_replays` binary (N > 0, green). It asserts byte-exactness, idempotence,
   non-vacuity and the provenance, so a fixture that merely exists will not pass.
4. **Fold into #438** (`aa313d357`) and rebuild **#439, #440 and the SCORE commit** on top.
   **Never `git filter-branch`** — detach, amend, rebuild descendants.
5. **Prove the rebuild inert**: `git diff e8f4d08a8 <new tip>` names only the fixture files, the `;; SCOPE:`
   line, and the docs you add. Per step, each commit's own delta must equal its old one except #438's.
   Then: gate exit 0, `refs/original/` empty, `git replace -l` empty, origin still an ancestor, tree clean.

## What #438's body must say

That this tree gates recorded migrations and grok's does not; that the fixture was written here for that
reason; which pre-image it was taken from; and that both uniform wraps are exercised.

## Rows this affects

`EXPECTATIONS-7o` is **not amended** (finding 34 — a contract is not edited after its results are seen).

- **E10** (the checkpoint) is the row that caught this; it stays the orchestrator's.
- **E3/E15** gain the disclosure: the migration's fixture, its oracle, and what it proves.
- **E20** (no knowingly-red commit) is why this is a fold and not a follow-up.
- A new row the orchestrator will check: **`every_recorded_migration_replays` is green at the tip, and the
  fixture is non-vacuous on BOTH wraps.**

## Then the orchestrator re-runs the floor

Prediction unchanged: **5856 run, 24 skipped** — a fixture adds no test.

## ⛔ Hard rules (unchanged)

No `mcp__pulsare__*` tools. Every Bash command starts with `cd /home/john/work/holon/wat-rs &&`. No
worktrees, no push, no subagents, no `scripts/floor.sh`, no clippy, no `filter-branch`, no hand-typed SHA
or subject. Quote your heredocs and read the message back. On any further red: do not re-run, capture
verbatim, name the assertion, STOP and report.
