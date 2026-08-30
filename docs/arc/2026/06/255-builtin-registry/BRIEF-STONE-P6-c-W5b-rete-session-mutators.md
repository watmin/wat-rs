# STONE P6-c-W5b — the rete session mutators, and the widening W5a refused

> Wave 5b. Read `BRIEF-STONE-P6-c-W5a-rete-predicates.md` first — this is its mirror image, and its
> forbidden act is this wave's required one.

## The six

```
:wat::rete::arm-session       :wat::rete::export      :wat::rete::eval-insert
:wat::rete::release-session   :wat::rete::import      :wat::rete::eval-test
```

**Pre-checked — verify, don't redo:** all six SHAPE=fits; **all six on `KNOWN_UNREVIEWED`**; **none on
`FROZEN_CHECKER_DEBT_LEDGER`**, and none was in the debt-adder set I measured earlier (that set's rete
members are `fire-rules`, `fire-once`, `fire-rules-explain`, `insert-all` — **all deferred**).

⛔ **The nine firing verbs are NOT in this wave** — `fire-*`, `insert-*` and their `$native` twins are
a coherent family where a public verb and its native half likely want coordinated treatment, and four
of them ADD checker debt. Separate stone. **And the four unverified readers** (`lower`,
`collect-rules`, `step-payload`, `axis-violation`) still wait on their purity. Affirmative cuts.

## ★ THE INVERTED PREDICTION — W5a's STOP is this wave's DELIVERABLE

W5a forbade widening `effectful_by_prefix`: its nine were read-only, so a purity-census red there
meant a mis-cut wave. **Here the opposite holds.** These six mutate a rete session — arm it, release
it, serialize it in and out. They should declare `@Purity Effectful`, and
`declared_purity_vs_effectful_by_prefix_census` asserts **`Effectful ⇒ effectful_by_prefix`**
(`src/intrinsic/mod.rs:987` — "one direction survives as a real assertion"). `:wat::rete::` is not on
that list.

**So this wave is expected to add `:wat::rete::` to `effectful_by_prefix`, and that is the honest
fix, not a way to quiet a gate.** W2 set the precedent widening `:wat::stream::` for `next`, and its
rider explicitly named and rejected the dishonest alternative — reclassifying the verb as Pure.

⚠ **STOP-1 is therefore inverted too: if NO widening turns out to be needed — if all six are genuinely
pure — the wave is MIS-CUT.** Report that; do not declare a verb Effectful to justify the widening.

★ **And predict the side effect.** Widening the prefix puts W5a's NINE PURE rete verbs under an
"effectful" prefix. That is legal — the reverse direction (`prefix ⇒ Effectful`) is a **counted
census, not an assertion** — but the reported disagreement count WILL rise by about nine (it sits at
108 today, all `:wat::config::`/`:wat::holon::`, zero rete). **Report the before and after number.**
A count that moves unexplained is how a census stops being read.

## Predictions

```
effectful_by_prefix    GAINS `:wat::rete::`   (the deliverable, not a workaround)
purity-census disagreements   108 → ~117      (report the real number)
KNOWN_UNREVIEWED       231 → 225              (−6, by the GATE'S OWN line)
FROZEN_CHECKER_DEBT    53 → 53                (unchanged)
population             116 → 110              registry 412 → 418
```

## Row 2 carries over

W3 found seven doc lies in ten verbs; W4 one in three plus five stale sites; W5a found none but
reported **two near-misses** rather than a bare "clean" — an arm comment true of a pure core and
false of its wrapper, and a `LAW A` comment showing **pre-fix** behaviour a reader could copy as
current. Read every doc against its body; report matched, corrected (with before/after), or
near-miss.

★ **Purity decides the example**: `Pure` AND `Deterministic` obliges a RUNNABLE `@example`.
`Effectful` does not — and an `@example` that arms or mutates a session had better be `-norun` or
genuinely self-contained.

## ⛔ The registry command, FIXED — my briefed one was broken

W5a's rider found it: `$(git ls-files '*.rs')` **cannot see an untracked file**, so a wave creating a
new module undercounts by that whole file. Use the git-independent form:

```bash
grep -rhcE '^[ \t]*#\[wat_intrinsic' --include=*.rs src crates | awk '{s+=$1} END {print s}'
cargo nextest run --release -E 'test(every_dispatched_verb_is_classified_or_disposed)' --no-capture 2>&1 | grep -aE "UNREVIEWED"
cargo nextest run --release -E 'test(declared_purity_vs_effectful_by_prefix_census)' --no-capture 2>&1 | grep -aiE "disagree|effectful"
python3 wat-scripts/hunt/p6c-disposition-census.py --no-multisite 2>&1 | grep -aE "HOMEABLE|AWAITING|RULED OUT"
```

`all_entries().count()` is that grep **+ 2**, always (`if`/`let` via `#[wat_special_form]`).

## STOP triggers — each REJECTS.

1. **No widening is needed** — all six are pure. The wave is mis-cut; report it. Never declare a verb
   Effectful to justify the widening.
2. **A doc contradicts its body and you cannot tell which is right.**
3. **A purity you would have to guess.**
4. **Either frozen ledger moves other than `KNOWN_UNREVIEWED` −6.** My pre-check was wrong — a finding.
5. **A verb turns out to be one of the firing family's halves in disguise** (a `$native` twin, or
   something `fire-*` calls). Report; it belongs to the deferred wave.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK of all six: shape · return type · arity · dispatch sites · both ledgers.
      Disagreements reported BEFORE any edit.
 1. ★ SIX RULINGS with disk citations. Instrument: AWAITING 81 → 75, population 116 → 110.
 2. ★ EVERY DOC READ AGAINST ITS BODY — matched / corrected (before+after) / near-miss.
 3. ★ REAL ARITY PUBLISHED — `metadata-of` for all six, before and after; each matches its guard.
 4. ★ THE ARITY ERROR SURVIVES for each — same op/expected/got, now from the shim.
 5. ★ DIRECT CALLS BYTE-IDENTICAL, before and after. `git show HEAD:<path>` — never `git stash`.
 6. ★ SIX ARITY GUARDS DELETED. Name any helper that goes dead.
 7. ★ `effectful_by_prefix` GAINS `:wat::rete::` — show the diff, and say in one sentence why it is
      the honest fix here and was forbidden in W5a.
 8. ★ THE DISAGREEMENT COUNT, before and after, with the ~9 rise explained by name.
 9. ★ `KNOWN_UNREVIEWED` 231 → 225 by the gate's own line. Debt ledger unchanged at 53.
10. ★ PURITY GROUNDED — one sentence per verb on what it MUTATES.
11. cargo build --release --all-targets — clean; warnings VERBATIM.
12. cargo nextest run --release -E 'test(rete) + test(intrinsic) + test(purity) + test(reflection)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Your pre-check table. The six rulings. Row 2's verdicts. Arity and error quotes. The
`effectful_by_prefix` diff with its one-sentence justification. The disagreement count before and
after. The gate's `UNREVIEWED` line. Six purity groundings naming what each mutates. Then the honest
deltas — especially any verb that was not what its name suggested.
