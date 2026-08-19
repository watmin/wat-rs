# DESIGN STONE — 118.B4-iii · THE WALL. `next` becomes the only way a Stream yields anything.

**Route B, the last stone.** B4-0 (`8f5252a0`) made `nth` native; B4-ii (`8c28ace2`) migrated the
corpus off `(first (drop X n))`. This closes the doors.

**Builder's rulings:** *"we do not do conventions - we do walls - users may not make mistakes in wat"*
(2026-08-18), and **option B** on the `nth`-on-Stream question (2026-08-18) — `nth` closes on Stream
too.

## What it does

Four verbs stop accepting a `Stream<T>`:

```
first    StreamContainer::indexable()      Stream => true  →  FALSE
rest     StreamContainer::has_tail()       Stream => true  →  FALSE
nth      StreamContainer::nth_indexable()  Stream => true  →  FALSE
empty?   NO CAPABILITY — runtime.rs:17337 is a hand-written arm routing AROUND measurable(),
         which is ALREADY false for Stream. That half is a DELETION. The compile-time half
         needs a new infer_list arm consulting measurable().
```

Afterwards `:wat::stream::next` → `NextOutcome<T>` = `Item(value, rest) | Exhausted` is the single
door. Positional access on a lazy sequence is spelled `(drop s i)` then `next` — which is what it
does.

## Why — two measurements, both from probes on disk

**The walk shapes** (`probe-118B4-forces-per-element-by-walk-shape.wat`, 5 elements, no memo):

```
A  next-only                  6 FORCED  = n+1   1x per cell
B  empty? + next             11 FORCED  = 2n+1  2x per cell
C  empty? + first + next     16 FORCED  = 3n+1  3x per cell
```

Walk C uses **no `rest`** and pays the full 3×. That killed "close `rest` only"; closing two of three
leaves 2×. Only all of them leaves `next` alone.

**The quadratic** (n=6, one stream, same answer `sum=15`):

```
index-based  (nth s i) for i=0..5     21 FORCED   = n(n+1)/2   QUADRATIC
next-walk                              7 FORCED   = n+1        linear
```

★ **`nth` is O(1) on a Vector and O(i) on a Stream with identical syntax.** A loop that is linear on
one is quadratic on the other and nothing at the call site says which you hold. That is the ruling's
basis: refusing a lazy sequence a syntax whose complexity class is a lie.

⚠ Both hazards **predate the arc** — `(first (drop s i))` was equally quadratic. B4-ii made it
shorter to write; it did not invent it.

## ★★ THE BLAST RADIUS — MEASURED TODAY, POST-MIGRATION, NOT INHERITED

The three capability bits were flipped on a scratch copy, built, floored, reverted.

```
before B4-ii   4765 run, 1802 passed, 2945 FAILED   ← ONE site cascading (service.wat:468)
after  B4-ii   4765 run, 4727 passed,   38 FAILED   ← a real worklist
```

**The migration collapsed the cascade.** The stdlib now loads under the wall; what remains is 38
nameable failures in three classes:

| class | n | disposition |
|---|---|---|
| **arc 118's own tests, made obsolete by the wall** | 18 | `core-nth` + `core-nth-differential` test `nth` on a Stream; the receiver set shrinks, so the Stream rows come OUT. Rewrite, do not "fix". |
| **`probe_arc118_lazy_seq` / `probe_arc118_2_lazy_map`** | 2 | `lazy_seq_cons_first_rest_traverses` literally tests the three-call walk. **Condemned by the wall** — rewrite onto `next` or retire with a reason. |
| **real corpus violators** | 17 | `probe_arc258_stone3_fix_source` (10) · `probe_arc251_decl_migrator` (8) · `probe_arc209_c1_defmacro_ast_walk` (2) · the loader gate (1) |
| **the loader gate** | 1 | `every_wat_scripts_file_loads_on_the_current_runtime` |

## ⛔ MY CENSUS WAS WRONG, AND ITS AGREEMENT WITH GREP IS WHY I BELIEVED IT

B4-ii reported **44 sites across 13 files** and I wrote into its stone that the form-tree census
"earned" the number because grep independently agreed.

**It is 48 across 16.** Four sites live in `tests/` — `probe_arc258_stone3_fix_source.wat`,
`probe_arc251_decl_migrator.wat`, `probe_arc209_c1_defmacro_ast_walk.wat` — and they are exactly the
files behind 20 of the 38 failures above.

The census was not blind to a *shape*. It was blind to a **directory**. I built the path list with
`grep -rl 'first (:wat::core::drop' wat/ wat-scripts/` and fed that same list to both instruments.
**`tests/` holds ~900 `.wat` files and was never in it.**

★ **Two instruments agreeing is not corroboration when they share an input.** The census and the grep
had one premise between them — my choice of search roots — and it was wrong, so they agreed precisely
where they were both blind. "The census earned the number" was a claim about the instrument that said
nothing about the population.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

**Consequence for this stone: there is no census to run.** The wall IS the census — impose it, read
the screams. The 38 failures are the worklist, and they are complete in a way no survey of mine has
been.
`[[feedback_impose_the_check_and_read_the_screams]]`

## The four questions (option B, as ruled)

- **Obvious? YES.** One rule, said once: *a Stream advances only through `next`.* A reader counts
  `next` calls and knows the force count.
- **Simple? YES.** Three capability flips in the table arc 278 built to be the single source of truth,
  plus one deletion. Nothing new is invented.
- **Honest? YES.** It closes the walk that hid a 3× cost AND the shape that hid an n²/2 cost, rather
  than the first while leaving the second. And the replacement spelling *is* the operation.
- **Good UX? YES.** `NextOutcome::Exhausted` already answers what `empty?` was asked; `(drop s i)` +
  `next` already spells positional access. The refusals can name both in their messages.

## ⚠ THE TRAP — THE COMPILER DOES NOT CATCH THE DEAD ARMS

`cargo build --release` was **clean** with all three bits flipped, twice now. `StreamContainer`'s
exhaustiveness protects against a container being *forgotten*; it does not fire when an arm goes
*dead*, because the arms sit inside a `match` guarded by `if container.<capability>()`.

Four arms must be hand-converted to `unreachable!(…)`, the house pattern already used two lines away
for `Tuple`/`HashSet`:

- `eval_positional_accessor`'s Stream arm (`runtime.rs` ~15456) and its checker mirror
- `eval_rest`'s Stream arm (`collection/eval.rs`, the `realize` branch)
- **`eval_nth`'s Stream walk (`runtime.rs` ~15686) — B4-0 built it one stone ago**
- **`nth-spec`'s `Seqable<T>` arm and `nth-spec-walk` (`wat/core.wat`) — B4-i's four-arm clause loses
  its fourth arm**

Nothing will remind you. `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## The ONE contract decision, pinned

**Every refusal hands the user the door.** `first`/`rest`/`empty?` name `:wat::stream::next` and its
`NextOutcome<T>` shape; `nth` names `(drop s i)` + `next` and says why — a lazy sequence has no O(1)
positional access, and pretending otherwise is what the wall exists to stop.

## ACCEPTANCE

| | assertion | instrument |
|---|---|---|
| 1 | walk C is **refused**, message names `next` | `.bad` fixture beside the probe |
| 2 | walk A still yields **6 FORCED for 5** | `probe-118B4-forces-per-element-by-walk-shape.wat` |
| 3 | `(nth s i)` is **refused**, message names `drop`+`next` | a new `.bad` fixture |
| 4 | the 17 real violators are migrated onto `next` | the floor |
| 5 | arc 118's own Stream-receiver tests are **rewritten, not deleted** | read the diff |

Plus: floor ≥4765/0, clippy 0, ignores 13.

⚠ **Every run capped.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s>`

## Out of scope — affirmative cuts

- **`(do (next s) (next s))` still forces twice.** Nothing short of linear types stops it. The wall
  makes every force a *visible* `next`, countable by reading; it does not make double-forcing
  impossible. Stated so it is never claimed as more.
- **`map`/`filter`/`foldr` over a Stream** — the rest of `mappable()`'s gap. Tracked, unowned.
- **B5** — `into` absorbs the drain; `stream->pvec`/`stream->vec` retire.
