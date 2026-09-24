# WEIGH — STONE 255.18 (C-b1a): the locus names its transport — ACCEPTED AS PARTIAL

**Executor commit `bca52b0e4`.** Weighed against disk by the orchestrator on 2026-09-24.

## Re-measured

| row | measured | result |
|---|---|---|
| tree | `git status --porcelain` | clean |
| floor | `.floor/2026-09-24T05-00-07Z/clean.log` | `6032 tests run: 6032 passed (9 slow), 22 skipped` |
| a thread launch claimed as Wire | `…_thread_yields_wire.wat.bad` on the snapshot `58d7e0ad3` vs the new binary | **old rc=0 → new rc=1**, a real hole closed |
| a process launch as Wire | `…_process_yields_wire.wat` | old 0, new 0 |
| the narrowing-gap pin | `…_generic_narrowing.wat.bad` | new rc=1 (`NoMatchingClauseAtCallSite`); a pin of the gap, which cannot say the other word until the gap closes |
| erasure witness | `wat-scripts/scratch-pad/255-18-with-label-erases-the-transport.wat` | rc=0 on both binaries; **the finding** |

Taken from the report without re-running: clippy 0, census `no STOP-8` (213 → 213), delta NEW 3 /
RECOVERY 0, ledger 220. The codemod `wat-scripts/fixes/locus-names-its-transport.wat` was dry-run and
diffed (7/7), and its replay fixture and ORACLE are committed.

## What landed

`Locus :- [T]`; `launch` returns the full `(Launched :- [S R Sh Lu T])`; `ThreadOpts` binds `Shared` and
`ProcessOpts` binds `Wire`. In `bracket.wat` the pair is the second half of the same single binding:
`spawn.wat` provides `launch` and `bracket.wat` provides `spawn-runner`, and 255.16's identical-binding
row admits it. Ten consumers in other files take `(Locus :- [T])`.

## What stopped — the waist is wider than the surface

`:wat::spawn::runner-count` (`spawn.wat:181`) and `:wat::spawn::with-label` (`spawn.wat:471`) are
**defclauses keyed on the concrete loci**. Their own comment says *"a new locus type joins as one more
clause here."* A generic `(Locus :- [T])` cannot narrow into them, so defservice's `start$impl`/`resume$impl`,
`bracket.wat` `map-worker`, and a probe keep the bare `Locus`. The executor stopped here rather than
choose, correctly: this was not a named STOP, but it is the builder's ground.

**A second hole was found:** `with-label` returns the bare `Locus`, and a bare `Locus` is accepted
wherever any `(Locus :- [T])` is expected. So a process locus passed through `with-label` type-checks as
a `Shared` launch.

## The four questions on the blocker

| option | Obvious | Simple | Honest | Good UX |
|---|---|---|---|---|
| N1 — the checker narrows a generic `(Locus :- [T])` into a defclause whose clauses cover every implementor | **N**: coverage depends on an implementor set the reader cannot see, and every such defclause elsewhere is one more place a new locus must be added (N×M) | — | — | — |
| **N2 — `runner-count` and `with-label` become `Locus` surface methods, each implemented in the locus's `extend-type`; `with-label` returns `(Locus :- [T])`** | Y | Y | Y | Y |

N2 puts per-locus behaviour **on the waist**, so adding a locus is one `extend-type` that satisfies the
whole contract. It also closes the erasure. The last per-locus defclause (`wat/test.wat:355`) and
defservice's `start-impl-thread`/`-process-params` copies are the same shape, the first to be measured
in the next brief and the second being step 3.

**Refusing a bare `Locus` as any instantiation** is recorded for C-b5, beside the "uninstantiated
aggregate `Handle` accepts any instantiation" arm S5 left standing. It becomes refusable once no bare
annotations remain.

## Brief errors, recorded

- It named 43 mentions in 20 files. The per-site census found 23 type-position lines; the 43 counted
  comments and `Locus/` calls.
- It assumed every consumer could take `(Locus :- [T])` by declaration alone. The defclauses refuse.
- `cargo wat` in `~/.cargo/bin` is a stale Aug-29 binary; the codemod ran under `./target/release/wat`.
  CLAUDE.md's codemod instruction names `cargo wat`. **Recorded here, not corrected there.**
