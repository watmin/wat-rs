# STONE P6-c-W5a — the rete predicate surface (nine), and a ledger that must SHRINK by nine

> Wave 5a. `:wat::rete::` has 28 verbs in the giant match; this is the **read-only** half's core.
> Read `BRIEF-STONE-P6-c-W3-runtime-reflection.md` (row 2's doc discipline) and
> `BRIEF-STONE-P6-c-W2-stream-program-stdlib.md` (the purity red) first.

## The nine

```
:wat::rete::deterministic?                  :wat::rete::alpha-match
:wat::rete::primitive?                      :wat::rete::alpha-match-local
:wat::rete::pure?                           :wat::rete::alpha-match-under
:wat::rete::total?
:wat::rete::vocabulary-admitted?
:wat::rete::cond-has-deferred-constraint?
```

**Pre-checked by the orchestrator — verify, don't redo:** all nine are SHAPE=fits; **all nine are on
`KNOWN_UNREVIEWED`**; **none is on `FROZEN_CHECKER_DEBT_LEDGER`**. Derive each arity yourself from
the guard you delete.

⛔ **The other 19 rete verbs are NOT in this wave**, and the cut is drawn on the axis that generates
the work — purity. The session-mutating half (`fire-*`, `insert-*`, `arm-session`, `release-session`,
`import`, `export`, and the `$native` twins) is Effectful and needs the widening below; and
`lower`, `collect-rules`, `step-payload`, `axis-violation`, `eval-test`, `eval-insert` I have **not**
verified as read-only, so they wait rather than ride on a guess. Affirmative cut.

## ★ THE PREDICTION, AND THE RED IT IS GUARDING AGAINST

```
KNOWN_UNREVIEWED         240 → 231   (−9, by the GATE'S OWN line, never a grep)
FROZEN_CHECKER_DEBT      53 → 53     (unchanged)
population               125 → 116
registry (anchored)      403 → 412
effectful_by_prefix      UNCHANGED — no widening
```

⚠ **`:wat::rete::` is NOT in `effectful_by_prefix`** (`src/runtime.rs`, `pub(crate) fn
effectful_by_prefix` — the list is `kernel · io · holon · config · stream`). W2 hit exactly this: it
declared `stream::next` Effectful and `declared_purity_vs_effectful_by_prefix_census` went red
because `:wat::stream::` was missing.

**So that census going red here means one of your nine is NOT read-only.** That is a **STOP, not a
chore**: report which verb and what its body does. **Do not widen the prefix to quiet the gate** —
widening is correct only for a namespace whose verbs are genuinely effectful, and this wave's whole
premise is that these nine are not.

## ★ A pleasing recursion worth getting right

`pure?`, `total?` and `deterministic?` are the verbs that **report** purity. Homing them makes the
purity system declare its own reflection surface's purity. Reading a purity fact is not the same as
having one — ground each on what its body does, not on what it is about.

## Row 2 carries over, and it has earned it

W3 found **seven** doc lies in ten verbs; W4 found one real lie in three plus five more stale sites.
Read every handler's prose against its body before carrying it into a `///` block. The known stale
family is `:wat::holon::HolonAST` in a return position — arcs 201/251/294.f retired it and never
swept the comments. **Report every correction with before/after text.**

★ And W1's coupling, still binding: **decide purity first, because `Pure` AND `Deterministic`
obliges at least one RUNNABLE `@example`.**

## The commands — use these, do not invent your own

```bash
grep -rhcE '^[ \t]*#\[wat_intrinsic' $(git ls-files '*.rs') | awk '{s+=$1} END {print s}'   # anchored
cargo nextest run --release -E 'test(every_dispatched_verb_is_classified_or_disposed)' --no-capture 2>&1 | grep -aE "UNREVIEWED"
python3 wat-scripts/hunt/p6c-disposition-census.py --no-multisite 2>&1 | grep -aE "HOMEABLE|AWAITING|RULED OUT"
```

⚠ `registry().all_entries().count()` is **anchored-grep + 2, always** — `:wat::core::if` and
`:wat::core::let` register via `#[wat_special_form]`. Three riders have reported that 2 as drift.
It is now in `FROZEN_CHECKER_DEBT_LEDGER`'s header. **Report the DELTA and the command.**

## STOP triggers — each REJECTS.

1. **The purity census goes red.** One of the nine is effectful. Report which; do not widen the prefix.
2. **A doc contradicts its body and you cannot tell which is right.**
3. **A purity you would have to guess.**
4. **A verb is not on `KNOWN_UNREVIEWED`, or IS on the debt ledger.** My pre-check was wrong — a finding.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: shape · return type · arity · dispatch sites · both ledger memberships.
      Disagreements reported BEFORE any edit.
 1. ★ NINE RULINGS with disk citations. Instrument: AWAITING 90 → 81, population 125 → 116.
 2. ★ EVERY DOC READ AGAINST ITS BODY — matched or corrected, with before/after text.
 3. ★ REAL ARITY PUBLISHED — `metadata-of` for all nine, before and after; each matches its
      deleted guard. (`metadata-of` is itself homed as of W4 — it reports on these from the registry.)
 4. ★ THE ARITY ERROR SURVIVES for each — same op/expected/got, now from the shim.
 5. ★ DIRECT CALLS BYTE-IDENTICAL, before and after. `git show HEAD:<path>` — never `git stash`.
 6. ★ NINE ARITY GUARDS DELETED. Name any helper that goes dead.
 7. ★ `KNOWN_UNREVIEWED` SHRINKS BY EXACTLY 9 — 240 → 231, by the gate's own line.
      `FROZEN_CHECKER_DEBT_LEDGER` unchanged at 53.
 8. ★ `effectful_by_prefix` UNTOUCHED — `git diff` on that fn is empty. Say it.
 9. ★ PURITY GROUNDED — one sentence per verb, and `pure?`/`total?`/`deterministic?` must each say
      what the BODY does, not what the verb is about.
10. cargo build --release --all-targets — clean; warnings VERBATIM.
11. cargo nextest run --release -E 'test(rete) + test(intrinsic) + test(purity) + test(reflection)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Your pre-check table. The nine rulings. Row 2's verdicts with before/after text. Arity and error
quotes. The gate's `UNREVIEWED` line before and after. `git diff` on `effectful_by_prefix`. Nine
purity justifications. Then the honest deltas — especially any verb that turned out not to be
read-only.
