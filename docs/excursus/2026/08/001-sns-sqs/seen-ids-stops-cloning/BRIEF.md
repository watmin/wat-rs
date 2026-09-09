# BRIEF — `seen-ids` stops cloning

## The work, in one paragraph

The queue's `seen-ids` set uses `:wat::core::HashSet`, whose `conj` **clones the entire set on every
insert** (`src/collection/eval.rs:333`). Eight thousand messages means eight thousand clones of a set
growing to eight thousand — a quadratic term inside the instrument built to measure scaling. Swap it to
`:wat::core::PersistentSet` and `:wat::set::conj`, which share structure at `O(log n)`. Three real edits.
Nothing else changes.

## Read in order

1. **`wat-scripts/queue/sqs.wat`** — search `seen-ids`. **32 mentions, 3 that matter**: the `:ephemeral`
   declaration, the initial empty value in `:init`, and the `(:wat::hashset::conj seen id)` at the
   redelivery-detection site. The other 28 are `:seen-ids (:queue::queue::State/seen-ids s)` threading
   through State constructors and **must keep threading unchanged**.
2. **`src/collection/eval.rs:322-335`** — the cloning insert you are moving away from. `:333` is
   `let mut out: HashSet<Value> = (**s).clone();`.
3. **`src/collection/eval.rs:388-410`** — `persistentset_conj_inner`, the replacement:
   `let out = (**s).insert(item.clone());` — a new trie sharing structure, no whole-set clone.
4. **`src/intrinsic/set.rs`** — the five verbs and their doc comments. `conj disj contains? empty?
   length`, nothing else.
5. **`wat-scripts/scratch-pad/probe-the-sets-get-a-persistent-variant.wat`** — a worked example of the
   surface in wat, including the constructor form.

## Implementation sketch

```wat
;; the :ephemeral declaration
seen-ids <- (:wat::core::PersistentSet :- [:wat::core::String])

;; the initial value in :init
:seen-ids (:wat::core::PersistentSet :- [:wat::core::String])

;; the one insert site
(:wat::set::conj seen id)
```

The 28 `:seen-ids (:queue::queue::State/seen-ids s)` threading sites are untouched.

## Blast radius

`wat-scripts/queue/sqs.wat` **only**. No `src/`. No other `.wat`.

## STOP triggers

**STOP-1** — if the fix requires touching anything outside `sqs.wat`, **STOP and say what**. The
persistent set already exists and is already registered; needing a substrate change means something about
it is incomplete, and that is worth more than this stone.

**STOP-2** — do **not** delete or weaken the `redeliveries` counter to make it cheap. It must still read
**0 at the inbox and nonzero at a subscriber tier** (20 on both graded runs). A counter that reads zero
everywhere has been broken, not optimised.

**STOP-3** — do **not** bound, window, or clear the set. Memory stays O(N); a window size is a constant
nobody chose.

**STOP-4** — do **not** touch the other four counters, admission, visibility, the cap, `StoredRow`, or
`Envelope`.

**STOP-5** — if `:wat::set::` turns out to be missing a verb you need, **STOP and name it.** This is the
type's first production caller; a gap it exposes is a finding about the type, not an obstacle to work
around with `:wat::hashset::`.

**STOP-6** — on any red floor arm: capture it whole, name the exact arm, **do not re-run it.**

## What "done" looks like

`grep` finds no `:wat::hashset::` on any `seen-ids` line. One n=2000 `vis-ms=1000` run on a quiet box
(state the load) reporting **wall clock and every phase time beside the 41 s / fill≈16–17 s baseline** —
this stone's purpose is that the instrument stops changing what it measures, and the axis is time, not
store calls. Inbox `redeliveries=0`, at least one subscriber tier nonzero, `distinct=8000`, `dup=0`, and
inbox `refused` still equal to the publisher's `full-retries`. `every_wat_scripts_file_loads` PASS. Floor
Summary reads 5237 / 22 skipped / 0 FAIL / 0 TIMEOUT.

**Write the SCORE to `SCORE.md`** in the shape of
the neighbouring SCORE files: the floor's Summary line verbatim, a row-by-row table, the measured numbers
beside the baseline, and the blast radius. Do not commit.
