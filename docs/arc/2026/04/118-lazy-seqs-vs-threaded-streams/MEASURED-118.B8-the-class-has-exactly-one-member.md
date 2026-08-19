# MEASURED — 118.B8 Part 3 · the class B3 named has EXACTLY ONE MEMBER, and here is why

Stone 118.B3 named a shape — **"a growing collection threaded through a lazy walk"** — said
*"Owed, tracked, not this stone,"* and it was tracked nowhere. B8 discharges it.

**Instrument:** `wat-scripts/scratch-pad/census-growing-collection-in-a-lazy-walk.wat` — a form-tree
walk (`read-string` → `ast->children`), committed, reproducible, usable by anyone.
`[[feedback_an_instrument_must_outlive_the_number_it_produced]]`

## The numbers

```
population          44 .wat files (ALL of wat/, via `find` — see the population note below)
defns scanned      373
lazy-cell walkers   12   (body constructs a deferred cell: stream::lazy / stream::cons)
growth hits          4   (a self-call carrying a conj/assoc-grown accumulator)
★ THE CLASS         1   (in BOTH sets)
```

## The four growth hits, and why three are not the class

| site | verdict | why |
|---|---|---|
| **`:wat::core::distinct-walk`** `wat/seq.wat:566` | ⛔ **THE CLASS** | wraps in `stream::lazy`, emits `stream::cons`, and grows `seen` by `conj` per emitted element. n deferred cells each pin an independent copy. |
| `:wat::core::stream->pvec-spec` `wat/seq.wat:168` | eager | the drain. Tail-recursive, no cell construction; the accumulator **is** its return value. One copy alive. |
| `:wat::bracket::collect-loop` `wat/bracket.wat` | eager | ordinary accumulating loop, no deferred cell. |
| `:wat::kernel::recv-all-loop` `wat/spawn.wat` | eager | ordinary accumulating loop, no deferred cell. |

## ★ WHY THE COUNT IS ONE — the reason, not just the number

The 12 lazy walkers are `remove` · `take-while` · `drop-while` · `take-nth-walk` ·
`interpose-walk` · `interpose` · `keep` · `keep-indexed-walk` · `map-indexed-walk` · `dedupe-walk` ·
**`distinct-walk`** · `reductions-walk`.

**Only one of them needs unbounded history.** `distinct` must remember *everything it has already
seen* to answer "is this new?". Every other lazy walker either carries no state at all, or carries a
fixed amount — and the closest sibling proves the point by construction:

```wat
(:wat::core::defn :wat::core::dedupe-walk<T>
  [prev <- :wat::core::Option<T> s <- :wat::stream::Stream<T>] -> ...
```

`dedupe` removes *consecutive* duplicates, so it carries **one element**, not a set. O(1) per cell.

**The class is not "we happened to write one such verb." It is "exactly one verb in the surface
requires unbounded history, and that is what makes it the only member."** A future verb needing
all-history — `frequencies`, a lazy `group-by`, `distinct-by` — would join it, and the census is on
disk to catch that.

## What this settles, and what it does NOT

**Settles:** 109's parked `NOTE-hashset-conj-full-clones-per-insert.md` covers the whole class.
There is no hidden sibling. B3's owed census is discharged with a count *and* a mechanism.

**Does NOT settle — and these are two DIFFERENT costs that must not be conflated:**

1. **O(n²) LIVE MEMORY** — n deferred cells each pinning an independent copy. This is B3's shape,
   affects **only** the lazy hit, and B3 already removed the memo half of it.
2. **O(n²) TIME** — `conj` full-cloning the container on every insert. This is independent of
   laziness and affects **all four** hits. It is 109's separate, parked question, and this census
   incidentally hands it a four-site worklist. ⚠ Measured only for `HashSet/conj` (12/53/223/875ms at
   n=2k/4k/8k/16k). Whether `Vector/conj` and `PersistentVector/conj` clone the same way is
   **unmeasured** — do not assume it from the shared verb name.

## ⚠ TWO THINGS THE INSTRUMENT GOT WRONG FIRST — both caught, both worth carrying

**1 — my first discriminator did not discriminate.** It asked whether the body mentions
`stream::next`, and labelled BOTH `distinct-walk` and `stream->pvec-spec` **LAZY**. But `next` is how
you *consume* a stream and is common to the harmful walker and the benign drain alike. The property
that actually distinguishes them is *building a deferred cell that captures the accumulator* —
`stream::lazy` / `stream::cons`. **I found this only because the census returned a hit I had not
predicted and I had to explain it.** A census that returns exactly what you expect teaches nothing.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

**2 — my population was one file short and I nearly reported it as complete.** I built the path list
with `ls wat/*.wat wat/**/*.wat`, which without `globstar` descends **one** level: it silently
excluded `wat/kernel/services/stdio.wat`. Caught by cross-checking against `find wat -name '*.wat'` —
43 vs 44.

★ **This is the same defect as yesterday's 44-site census, and it did NOT repeat only because the
second instrument answers the same question by a DIFFERENT mechanism.** Yesterday a form-tree census
and a grep "agreed" at 44 because I had fed both the same path list. Here `find` walks the tree and
the glob expands a pattern — they cannot share a blind spot. **Agreement is worth something only
when the instruments are independent.**
`[[feedback_two_instruments_agreeing_is_not_corroboration]]`

(The missed file proved innocent — 361 → 373 defns, hits unchanged at 4. That it changed nothing is
a fact I now *know* rather than one I assumed.)

## Reproduce

```bash
printf '[%s]\n' "$(find wat -name '*.wat' | sort | awk '{printf "\"%s\" ", $0}')" \
  | ./target/release/wat wat-scripts/scratch-pad/census-growing-collection-in-a-lazy-walk.wat
```
