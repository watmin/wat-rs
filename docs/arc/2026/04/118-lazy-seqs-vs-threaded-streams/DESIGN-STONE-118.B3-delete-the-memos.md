# DESIGN STONE — 118.B3 · delete BOTH memos. The stone the whole arc was aimed at.

**Route B, stone 3 of 5.** B1 (`488eacd0`) minted `Seqable<T>`; B1a (`eab12e05`) made a concrete
instantiation satisfiable; B2 (`b4a8f86b`) collapsed six verbs and deleted the seven twins, leaving
**zero three-call Stream walkers in the corpus.** That was the precondition. It is now met.

## What it does

Delete `forced: Arc<OnceLock<Arc<Stream>>>` from **both** cell kinds and the caching logic in
`realize`. Eight sites, **one file**:

```
src/stream/mod.rs:66,73     LazyCell.forced        (a wat closure per cell)
src/stream/mod.rs:124,131   NativeLazyCell.forced  (a Rust closure per cell)
src/stream/mod.rs:168,192   realize's Thunk arm       — the get / get_or_init pair
src/stream/mod.rs:195,200   realize's NativeThunk arm — the same pair
```

`grep -rn '\.forced\|forced:' src/ crates/ tests/ --include=*.rs` returns those eight and **nothing
else** — every other hit is prose (`compiler-forced`, doc comments). The blast radius is one file.

## Why — this is no longer a prediction

The memo was never an optimization. Measured this session:

```
population B (native map chain), n=400k    memo ON 3.99s → memo OFF 1.70s   2.35× FASTER without it
population C (wat generator), 100k…800k    3,188 B/elem  → FLAT (−0.15 B/elem over an 8× range)
distinct, n=8000                           OOM >2 GB     → completes
distinct, n=16000                          —             → completes
```

Its only function was masking the three-call walk, and there is no longer a three-call walk to mask.

★ **It also fixes a hard OOM in a core verb.** `(into [] (distinct (range 0 8000)))` currently
exceeds 2 GB. The mechanism is a two-part interaction — `HashSet/conj` full-clones
(`collection/eval.rs:613`) *and* the memo retains every cell, so n live cells each hold an
independent full copy: O(n²) live memory. Neither half is fatal alone (an eager `foldl`+`conj` at
n=8000 completes in 471 ms). **B3 removes the half that turns transient allocation into permanent
retention.** Full evidence: `MEASURED-distinct-ooms-at-8000.md`.

## The four questions

- **Obvious? YES.** A cache whose only job was hiding a protocol that no longer exists.
- **Simple? YES.** Deletion. Two struct fields, four `realize` lines, one import. Nothing replaces it.
- **Honest? YES.** It stops the substrate depending on a cache to keep user code from running three
  times, and it removes a live OOM rather than a slope.
- **Good UX? YES.** A lazy pipeline stops being O(n) in memory, which is the entire point of
  laziness and the thing wat did not have.

## ⚠ THE TRAP — an instrument whose MEANING changes under this stone

`wat-scripts/scratch-pad/probe-118B-memo-state-detector.wat` prints `"FORCED"` once per user-code
call over 5 elements. Its readings:

```
before B2   memo ON  + three-call walk   →  5    (the memo is hiding 15)
before B2   memo OFF + three-call walk   → 15    (the defect, visible)
AFTER  B3   memo OFF + next-based walk   →  5    ← SAME NUMBER, OPPOSITE REASON
```

**A `5` no longer proves the memo is present.** After B3 it proves the *walk* is single-force, which
is what we want — but anyone reading the old header will mis-conclude. **The stone must rewrite that
probe's header.** The instruments that actually witness the memo's death are the retention slope and
the `distinct` OOM, not the force count.

This is the third instrument this session whose reading meant something other than it appeared to.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

## ⚠ The residual hazard, named — not deferred

The corpus has zero three-call Stream walkers (measured, with a corrected balanced-bracket parse —
my first census used a naive `->` split and silently missed half the forms, including
`stream->pvec`, because `->` appears in its own name). **But `first`/`rest`/`empty?` still ACCEPT a
Stream**, so *user* code can still write the three-call walk — and after B3 that runs their function
**3× per element**, silently, for effectful `f`.

**B4 closes this permanently** by making the three-call walk unrepresentable, and it is a dialect
ruling the builder owns. B3 does not create the hazard — it exists today, masked. B3 unmasks it for
user code while fixing it for the stdlib. **That trade is the stone's honest cost and it is stated
in the record, not discovered later.**

## ACCEPTANCE — three numbers, all required, all with instruments already on disk

| | assertion | instrument |
|---|---|---|
| 1 | **`f` runs exactly N times for N elements** | `probe-118B-memo-state-detector.wat` → `"FORCED"` ×5 for 5 |
| 2 | **per-element retention is FLAT** | `probe-118B-dorun-retention-slope.wat` at 100k/200k/400k/800k → slope ≈ 0 |
| 3 | **`distinct` at n=16000 completes** | capped run, rc=0, out=16000 |

Plus: floor ≥4714/0, clippy 0, ignores 13.

⚠ **Every run capped.** `systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s>`
— verified both ways (normal work completes; a 2.5 GB run is SIGKILLed at 512M). `MemorySwapMax=0`
is the load-bearing half: without it a runaway swaps instead of dying, and a swapping box cannot be
killed interactively.

## Rooms beyond the eight sites

Three doc comments describe the memo as live and go stale on this stone —
`src/types.rs:1713`, `src/runtime.rs:12264`, and `src/stream/mod.rs`'s module header (which still
says *"without the cache `f` ran three times per element"*). Two test doc comments name it as well
(`tests/types/probe_stone_118_11a_next.rs:4`, `tests/types/probe_arc118_lazy_seq.rs:4`). **Updating
them is part of the stone** — a comment that survives its subject is the rot this project keeps
paying for.

`use std::sync::{Arc, OnceLock}` (`:19`) loses its second import; clippy will say so.

## Out of scope — affirmative cuts

- **Closing `first`/`rest`/`empty?` on Stream** — B4, the builder's dialect ruling.
- **`HashSet/conj`'s full clone** — a separate finding with its own blast radius: it is O(n²) *time*
  for every HashSet accumulation in the language, streams or not. B3 removes the memory half only.
- **The class census** — "a growing collection threaded through a lazy walk" is a shape, not just
  `distinct`. Owed, tracked, not this stone.
- **`dorun` building a Vector and binning it**, `length` on a Stream raising, `first` on an exhausted
  Stream returning bare `nil` — B5.
