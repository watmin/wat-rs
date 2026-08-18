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

## The mechanism, isolated to one variable

Two runs, **identical 8,000-cell chain length**; the only difference is whether the `seen` set grows:

```
8000 elements, ALL IDENTICAL   (seen stays size 1)     rc=0     out=1     survives
8000 elements, ALL DISTINCT    (seen grows to 8000)    rc=137             OOM
```

**The growing `HashSet` is the bomb, not the cell chain.** `distinct` threads
`seen <- HashSet<T>` and `conj`s one element per step. Each `conj` yields a *new* set, and the
`forced: OnceLock` memo (`src/stream/mod.rs:66`/`:124`) retains **every lazy cell** — so all *n*
intermediate sets stay reachable. That is O(n²) memory: at n=8,000 roughly 32M retained element
slots, which is the 2 GB.

⚠ Not yet measured: whether `HashSet/conj` copies eagerly (`hashset_conj_inner`,
`src/collection/eval.rs:599`) or shares structurally. Either way the *retention* of n distinct sets
is sufficient to explain the curve — but the constant differs, and that is a decomposition nobody
has run. `[[feedback_measure_the_decomposition_never_read_it]]`

## Why nothing caught it

The floor exercises `distinct` only on tiny inputs. Nothing in the corpus drains it at a scale where
the quadratic term dominates, so 4714 green tests say nothing about this. `NISI FRANGAS, NIHIL PROBAS.`

## What it predicts about B3

**B3 (delete both memos) should fix this outright**, because the retention — not the algorithm — is
what turns O(n) work into O(n²) memory. That is a **prediction**; B3 measures it with this file's
instrument. It also raises the stakes on B3: this is not only a slope, it is a hard OOM in a verb a
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
