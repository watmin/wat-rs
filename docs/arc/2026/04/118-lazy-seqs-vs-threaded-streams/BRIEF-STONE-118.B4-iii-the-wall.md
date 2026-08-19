# BRIEF — STONE 118.B4-iii · the wall, in three phases

`first`, `rest`, `empty?`, and `nth` stop accepting a `Stream<T>`. `:wat::stream::next` becomes the
only way a Stream yields anything. Read `DESIGN-STONE-118.B4-iii-the-wall.md` first — it carries the
measurements, the ruling, and the 38-failure worklist this brief works from.

## ⛔ YOU DO NOT RUN THE FLOOR. That is new, and it is deliberate.

Four riders on this arc backgrounded `scripts/floor.sh`, ended their turns, and delivered nothing —
including one warned explicitly with the count. The affordance is the defect, not the riders, so the
tier has moved: **you edit and report; the orchestrator builds the floor and clippy centrally, once,
after your tree is quiescent.** (`docs/COMPACTION-AMNESIA-RECOVERY.md` FM 19, amended 2026-08-18.)

**You MAY run**, in the FOREGROUND: `cargo build --release`, `./target/release/wat --check <file>`,
a single `.wat` probe, and a SCOPED `cargo nextest run --release -E 'test(<pattern>)'`.
**You may NOT run** `scripts/floor.sh` or an unscoped `cargo nextest`. If you want the full picture,
say so in your report and I will run it.

## Phase 1 — finish B4-ii over the population my census missed (4 sites, mechanical)

B4-ii reported 44 sites across 13 files. **It is 48 across 16.** I built the path list with
`grep -rl … wat/ wat-scripts/` and never included `tests/`, which holds ~900 `.wat` files. The four
survivors are the same `(first (drop X n))` shape the committed codemod already rewrites:

```
tests/resolve/probe_arc258_stone3_fix_source.wat:17,54
tests/resolve/probe_arc251_decl_migrator.wat:50
tests/macros/probe_arc209_c1_defmacro_ast_walk.wat:10
```

Run the committed codemod over exactly those three paths:

```
printf '["tests/resolve/probe_arc258_stone3_fix_source.wat" "tests/resolve/probe_arc251_decl_migrator.wat" "tests/macros/probe_arc209_c1_defmacro_ast_walk.wat"]\n' \
  | ./target/release/wat ./wat-scripts/fixes/first-of-drop-to-nth.wat
```

No new code. Dry-run on a `/tmp` copy and `diff` first. Then confirm with
`wat-scripts/scratch-pad/census-first-of-drop.wat` over **all 16 files** — expect **0**.

★ Phase 1 is green on its own and does not need the wall. Verify it with a scoped
`nextest -E 'test(arc258_stone3_fix_source) + test(arc251_decl_migrator) + test(arc209_c1)'`
before moving on.

## Phase 2 — the wall

```
first    StreamContainer::indexable()      Stream => true  →  FALSE
rest     StreamContainer::has_tail()       Stream => true  →  FALSE
nth      StreamContainer::nth_indexable()  Stream => true  →  FALSE
empty?   delete the hand-written Stream arm at runtime.rs:17337 — it routes AROUND
         measurable(), which is ALREADY false for Stream. Then add an infer_list arm
         consulting measurable(), mirroring `rest`'s at check.rs:4487.
```

★ **Then hand-convert FOUR now-dead arms to `unreachable!(…)`.** The compiler will NOT tell you —
measured twice: `cargo build --release` was clean with all three bits flipped, both times. The
exhaustiveness guarantee catches a *forgotten* container, not an arm that went *dead* behind an
`if capability()` guard. The house pattern sits two lines away for `Tuple`/`HashSet`:

- `eval_positional_accessor`'s Stream arm (`runtime.rs` ~15456) + its checker mirror
- `eval_rest`'s Stream arm (`collection/eval.rs`, the `realize` branch)
- **`eval_nth`'s Stream walk (`runtime.rs` ~15686)** — B4-0 built it one stone ago
- **`nth-spec`'s `Seqable<T>` arm and `nth-spec-walk` (`wat/core.wat`)** — B4-i's clause drops to
  three arms

**Every refusal names the door.** `first`/`rest`/`empty?` name `:wat::stream::next` and its
`NextOutcome<T>` = `Item(value, rest) | Exhausted` shape. `nth` names `(drop s i)` then `next`, and
says why: a lazy sequence has no O(1) positional access, and pretending otherwise is what this wall
exists to stop.

## Phase 3 — rewrite arc 118's own tests, which the wall makes obsolete (20)

These are **not** failures to repair. They assert behaviour the wall deliberately removes.

- **`wat-tests/core/core-nth.wat` and `core-nth-differential.wat` (18)** — the Stream rows come OUT.
  `nth`'s receiver set is now Vector / PersistentVector / List, so the differential covers three, not
  four. Keep every eager row. **Do not delete the files.**
- **`tests/types/probe_arc118_lazy_seq.wat:13-14`** — `(first s)` and `(first (rest s))`, the
  three-call walk in its purest form. Rewrite onto `next`, preserving what the test *measures*
  (traversal order and laziness), not how it spells it.
- **`tests/types/probe_arc118_2_lazy_map.wat:16`** — `(first mapped)`. Same treatment.

★ Ask what each test MEASURES before changing how it measures. A test rewritten to pass while
asserting something weaker is worse than one deleted with a reason.
`[[feedback_ask_what_a_test_measures_before_fixing_how_it_measures]]`

## Blast radius

`src/collection/seq_container.rs`, `src/runtime.rs`, `src/check.rs`, `src/collection/eval.rs`,
`wat/core.wat`, the 3 phase-1 test files, and the 4 phase-3 test files. **No other `wat/` changes** —
B4-ii already cleared the stdlib.

## STOP triggers — each is "ship nothing further, report the gap, stop"

**STOP-1** — a violator you cannot migrate without changing what its test measures. Report the test,
the site, and what it asserts. Do not weaken an assertion to clear the wall.

**STOP-2** — the census over all 16 files does not return 0 after phase 1. Report what remains.

**STOP-3** — a scoped `nextest` shows a failure OUTSIDE the 38 the design stone enumerates. That is a
new finding, not a cascade. Copy the block verbatim, name the assertion, stop.

**STOP-4** — you cannot flip a capability bit without changing which containers a DIFFERENT capability
accepts. Report what forced it.

## Your report

1. Phase 1: sites migrated, census over all 16, and the scoped test result.
2. Phase 2: the four `unreachable!()` conversions, each with its file:line, and the text of each of
   the four refusal messages.
3. Phase 3: for each rewritten test, what it measures and how you preserved that.
4. What you ran, and its output. Say plainly that you did not run the floor.
5. Honest deltas.
6. Wall-clock.
