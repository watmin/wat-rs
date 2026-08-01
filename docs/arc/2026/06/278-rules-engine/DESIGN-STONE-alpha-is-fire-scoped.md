# DESIGN-STONE — a natively-fired Session carries no alpha-memory

> **Origin (builder's ruling, 2026-07-31):** *"the wat-forms needs to be correct — the rust-native needs
> to be correct and fast … the wat side is an oracle to measure correctness against — the rust side is
> where we get all the perf we need while remaining correct … **if the wat side is naive and wasteful,
> so be it — the rust side is what users actually use.**"*
>
> This settles a class, not just a field: **the oracle is never optimized.** Its waste is acceptable by
> design. All performance lives in the kernel, and the kernel is licensed to **diverge in SHAPE while
> matching in RESULT** (R22 `OCVLI NOVI, ORACVLVM IMMOTVM`).

## The measurement — alpha is 31.3% of fire, on its own

`9d9a4e77` split `to_persistent` by field. At `G=200 W=200` (40,200 facts), mean of 3:

```
OUT: to_persistent   52.99 ms  31.5%
  out:alpha          52.66 ms  31.3%   ← 99.4% of the phase
  out:beta            0.00 ms   0.0%   ← already cleared before freeze
  out:production      0.33 ms   0.2%
```

The prize is **~31% of fire**, and it is alpha alone. (The seam had this as an *attribution*; it is now
a measurement. It did not have to survive — production could have been the bulk. It wasn't.)

## Why it is free — alpha is WRITE-ONLY in both engines

Grounded in both directions this session:

- **Native** clears it at the top of every fire — `kernel.rs:1735` (`fire_fixpoint_delta`),
  `kernel.rs:1005` (`fire_once_session`). The incoming alpha is discarded before a single pass runs.
- **The oracle** re-seeds the alpha fold with an **empty** `PersistentMap` — `rete.wat:1409-1411`
  (`fire-once`) — and never reads `Session/alpha-memory` as an input. The alpha threaded through
  `insert-spec` (`:841`), `fire-fixpoint` (`:1525`) and `retract` (`:1887`) is carried, never consumed.

So the frozen alpha is not state. It is **fire-scoped scratch that the record type presents as state** —
and the kernel spends a third of every fire serializing it for nobody.

**Beta already got this treatment** (`kernel.rs:1017`, `:2462`), which is why `out:beta` reads 0.00 ms.
The oracle still returns beta populated. That is not a defect — it is exactly the licensed
shape-divergence, and this stone extends it to alpha rather than inventing it.

## The three consumer checks (answered before the strike, not after)

| check | verdict | ground |
|---|---|---|
| **a — any reader?** | ONE, and it inspects internals | `probe_arc278_2b_insert_alpha.wat` (3 sites) |
| **b — whole-Session differentials?** | **none** | all ~24 oracle-vs-native probes compare COUNTS/SUMS; a grep for Session equality returns nothing |
| **c — EXPLAIN / snapshot / query?** | **none** | `Session/alpha-memory` appears at exactly **7 sites tree-wide**: 3 in that probe, 4 in `rete.wat` (all oracle-internal carry) |

## ★ THE ONE CONTRACT DECISION

**The clear goes at the two FIRE SITES, never inside `to_persistent`.**

`to_persistent` must stay a **pure converter**: `round_trip_fired_session` (`kernel.rs:3150`) asserts
`to_persistent(to_transient(fired)) == fired`. Putting the clear inside it makes the converter lossy and
that identity false — and it would break a test whose subject is *conversion*, for a reason that has
nothing to do with conversion. Mirror what beta already does: clear at the fire path's own freeze
boundary, one line, with the reason attached.

Two sites, both ending `wm.beta.clear(); … to_persistent(wm)`:
- `fire_once_session` — `kernel.rs:1017`
- `fire_fixpoint_delta` — `kernel.rs:2462`

⚠ **A trap the rider must not "clean up":** `kernel.rs:3206` runs the four passes inline *specifically
to avoid* `fire_once_session`, because that fn clears beta before freeze. It is deliberate. Leave it.

## The RED gate — and what would turn it red

A new probe, `probe_arc278_alpha_is_fire_scoped`, asserting **four** things — the last two are what stop
it being a gate that cannot notice (R59 `NISI FRANGAS, NIHIL PROBAS`):

1. `native-alpha-key-count` (fired via `fire-rules`) **== 0** — the clear happened.
2. `oracle-alpha-key-count` (fired via `fire-rules-spec`) **> 0** — **the workload really does populate
   alpha, so assertion 1 is not vacuously true**, and the oracle is provably UNMOVED.
3. + 4. `native-derived-count` **==** `oracle-derived-count`, both **> 0** — the RESULT is untouched.

Red today on (1). A gate that only asserted (1) would pass just as green over a workload with no
matching facts at all; (2) is what makes it a measurement.

## Blast radius

`src/rete/kernel.rs` (2 clears + 1 stale doc comment on `round_trip_fired_session`, which claims
"populated alpha/beta"), `wat/rete.wat` (the `Session` doc comment — mark alpha/beta fire-scoped, using
the vocabulary already established by `Support`'s *"EPHEMERAL — carried only in Explained"*),
`tests/rete/probe_arc278_2b_insert_alpha.{wat,rs}` (re-point to the oracle), and the new gate.
**No `.wat` corpus migration. No codemod. The oracle's LOGIC is untouched** — only its record's comment.

## Re-pointing 2b — and why that is not a weakening

Its three entries fire via native `fire-rules` and then read `Session/alpha-memory`; natively that
becomes empty. Re-point them to `fire-rules-spec`, where alpha population is a real, observable
property of the spec.

Native alpha-match correctness is **not** lost with it: if native's alpha pass or its binding-flow
broke, no facts would match, no joins would seed, and **every one of the ~24 count differentials would
go red.** 2b's unique contribution is the *shape* of alpha population — which is now an oracle-only
property, and that is exactly where the assertion belongs.

## Out of scope = REJECTED (affirmative cuts)

- **Deleting the `alpha-memory` / `beta-memory` fields from the `Session` record.** The oracle uses both
  internally and returns them; the record is shared. Removing a public field is a different, larger
  stone with a corpus surface.
- **Making the oracle drop alpha too.** Explicitly refused by the ruling — the oracle is never
  optimized, and symmetry here would buy nothing (the oracle's cost is not a cost we pay).
- **Clearing `production`.** It IS the result. Untouchable.
- **Re-populating beta natively to match the oracle.** The inverse trade: costs time to restore data
  nobody reads.
