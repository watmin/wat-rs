# ⛔ MEASURED — `:wat::core::distinct` OOMs at 8,000 elements. PRE-EXISTING, not B2.

**2026-08-18, found while scoring stone 118.B2.** The builder's machine filled RAM and swap and had
to be hard-rebooted. No kernel OOM record survived the reboot, so this is **not** proven to be the
cause — but it is a live bomb of exactly the right shape, and it is in a core verb.

## The measurement

Every run under `systemd-run --user --scope -p MemoryMax=2G -p MemorySwapMax=0`, per-cell exit code
captured (no pipe — a piped exit code reports the pipe's last stage, not the run's):

```
(into [] (distinct (range 0 n)))          n=2000   rc=0     320ms   out=2000
                                          n=4000   rc=0     541ms   out=4000
                                          n=8000   rc=137   963ms   out=          ← SIGKILL, >2 GB
                                          n=16000  rc=137   982ms   out=
```

**It exceeds 2 GB somewhere between 4,000 and 8,000 elements, in under a second.**

## It is PRE-EXISTING — proven against HEAD, not assumed

`wat/seq.wat` was reverted to `git show HEAD:wat/seq.wat` (the original `distinct-stream` twin,
untouched by any B2 work), rebuilt, and re-run:

```
HEAD's original distinct    n=4000   rc=0     out=4000
                            n=8000   rc=137   out=          ← same death
```

**B2 did not cause this.** The corrected B2 form restores the original algorithm, so it inherits the
same defect — it does not introduce it.

## ★★ THE MECHANISM — TWO designs, each harmless ALONE, catastrophic TOGETHER

⚠ **This section replaces an earlier draft that named retention as "the" mechanism. That was a story
told before the discriminating runs existed.** Both halves are now measured.

**Half 1 — `HashSet/conj` FULL-CLONES the set.** `src/collection/eval.rs:613`:

```rust
let mut out: HashSet<Value> = (**s).clone();   // std::collections::HashSet, behind an Arc
out.insert(item.clone());
```

`Value::wat__std__HashSet(Arc<HashSet<Value>>)` (`value.rs:114`) is **not** a persistent
structurally-shared set. Its own comment says the strategy outright: *"clone-then-new-Arc."* So
building an n-element set by `conj` allocates O(n²) entries in total.

**Half 2 — the `forced: OnceLock` memo retains every forced cell** (`src/stream/mod.rs:66`, `:124`).

### Each one alone is survivable — measured

```
EAGER foldl+conj, n=8000, no stream at all        rc=0   471ms    ← half 1 alone: FINE
```

Because in a fold each intermediate set is freed the moment the next replaces it. Peak is **one**
set: O(n). The clone costs time, not peak memory.

```
LAZY distinct, n=8000,  memo OFF (throwaway build)  rc=0   out=8000
LAZY distinct, n=16000, memo OFF                    rc=0   out=16000
LAZY distinct, n=8000,  memo ON  (shipping)         rc=137 OOM >2 GB
```

Half 2 alone is the familiar ~297 B/element overhead.

### Together

The lazy walk creates n cells; the memo keeps **all n alive**; each holds a **fully independent
copy** of the `seen` set. n live cells × an average n/2-entry copy = **O(n²) LIVE memory.** At
n=8,000 that is ~32M retained entries — the 2 GB.

**Neither file is wrong when you read it alone.** A structure-sharing set would make the retention
cheap; a non-retaining walk would make the cloning transient. The defect exists only in the product,
which is why code review of either side finds nothing.

## Why nothing caught it

The floor exercises `distinct` only on tiny inputs. Nothing in the corpus drains it at a scale where
the quadratic term dominates, so 4714 green tests say nothing about this. `NISI FRANGAS, NIHIL PROBAS.`

## What it predicts about B3

**B3 (delete both memos) fixes this — MEASURED, not predicted.** The memo-off rows above show
n=8,000 and n=16,000 both completing. Deleting the memo returns peak memory to the eager fold's
O(n); the clone remains an O(n²) *time* cost, which is a separate and much less urgent question. It also raises the stakes on B3: this is not only a slope, it is a hard OOM in a verb a
user can reach with eight thousand items.

## The standing instrument, and it is now mandatory

Every measurement in this tier runs capped:

```
systemd-run --user --scope -q -p MemoryMax=<N> -p MemorySwapMax=0 timeout <s> <cmd>
```

Verified both ways this session: a normal run completes under the cap, and a run needing ~2.5 GB is
SIGKILLed (rc=137) at 512M rather than taking the machine with it. **`MemorySwapMax=0` is the
load-bearing half** — without it the process swaps instead of dying, and a swapping box cannot be
killed interactively. That is what "couldn't kill it before swap filled" looks like from the outside.

## ★ THE CLASS, and it is bigger than `distinct`

The shape is **"a growing collection threaded through a lazy walk."** `distinct` threads a
`HashSet`. Anything else that accumulates a container across stream cells has the same product of
the same two halves. **Nobody has censused for it.** That census is owed, and it is not part of B2.

Also unresolved and now sharper: `HashSet/conj`'s full clone is O(n²) *time* for every HashSet
accumulation in the language, lazy or not — the eager fold above survives on memory but still pays
32M inserts to build 8,000 entries. That is its own finding, independent of this tier.
