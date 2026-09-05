# DESIGN-STONE — `fire-rules` must not return an alpha-memory the oracle does not

> **v2 (2026-07-31) — REDRAWN after the rider's STOP-4 falsified v1's premise.** v1 said "the oracle
> populates alpha in its output, so re-point the probe there." **False.** `fire-stratified`
> (`rete.wat:1817-1820`) returns `:alpha-memory (:wat::core::PersistentMap)` — EMPTY — and
> `fire-rules-spec` carries that through. The line was in the first grep of this session and was read
> past; four *other* alpha sites were read in detail and a claim was built on their pattern. **A grep
> that returns hits is not an enumeration.** The STOP trigger caught it before anything shipped.
>
> The correction makes the stone **stronger**, and changes what it is: not a perf optimization that
> costs a divergence, but a **divergence being closed that happens to buy 31% of fire.**

> **The governing ruling (builder, 2026-07-31):** *"the wat-forms needs to be correct — the rust-native
> needs to be correct and fast … **if the wat side is naive and wasteful, so be it — the rust side is
> what users actually use.**"* The oracle is never optimized; all perf lives in the kernel, which may
> diverge in SHAPE while matching in RESULT (R22 `OCVLI NOVI, ORACVLVM IMMOTVM`).

## The actual state of the disk — native and oracle DISAGREE on alpha, today

| verb | alpha-memory | beta-memory | ground |
|---|---|---|---|
| oracle `fire-rules-spec` | **empty** | **empty** | `rete.wat:1817-1820` (`fire-stratified` returns both empty); `:1839-1840` carries them |
| native `fire-rules` | **POPULATED** | empty | `kernel.rs:2462` clears beta before freeze; alpha is not cleared |
| oracle `fire-once` | populated | populated | `rete.wat:1462` — the single-pass verb genuinely fills both |
| native `fire-once'` | populated | empty | `kernel.rs:1017` clears beta |

Two corrections to the record, both mine:

1. **There is no standing beta divergence.** I reported one. Both fixpoint verbs return beta empty —
   they **agree**. Retracted.
2. **The alpha divergence is real and runs the other way.** Native returns alpha the oracle does not.
   Clearing it natively brings the kernel **into** alignment.

## The measurement — the divergent field costs 31.3% of fire

`9d9a4e77` split `to_persistent` by field. At `G=200 W=200` (40,200 facts), mean of 3:

```
OUT: to_persistent   52.99 ms  31.5%
  out:alpha          52.66 ms  31.3%   ← 99.4% of the phase
  out:beta            0.00 ms   0.0%   ← already cleared before freeze
  out:production      0.33 ms   0.2%
```

So the kernel spends a third of every fire serializing the one field it does not share with its oracle.

## Why it is free — alpha is WRITE-ONLY in both engines

- **Native** discards the incoming alpha at the top of every fire — `kernel.rs:1735`
  (`fire_fixpoint_delta`), `kernel.rs:1005` (`fire_once_session`).
- **The oracle's `fire-once`** seeds its alpha fold with an **empty** `PersistentMap`
  (`rete.wat:1409-1411`) and never reads `Session/alpha-memory` as an input. The alpha threaded through
  `insert-spec` (`:841`), `fire-fixpoint` (`:1525`) and `retract` (`:1887`) is carried, never consumed.

Alpha is fire-scoped scratch that the record type presents as state.

## ★ THE ONE CONTRACT DECISION — clear it in `fire_fixpoint_delta` ONLY

Not in `fire_once_session`, and **never** inside `to_persistent`.

- **`to_persistent` is a pure converter** — `round_trip_fired_session` (`kernel.rs:3150`) asserts
  `to_persistent(to_transient(fired)) == fired`. A clear inside it makes that identity false and breaks a
  *conversion* test for a reason that has nothing to do with conversion.
- **`fire_fixpoint_delta` is where both the cost and the divergence are.** It is what `fire-rules` runs
  (`fire_fixpoint`, the naive loop at `kernel.rs:1174`, is `#[allow(dead_code)]` — a documentation
  reference, explicitly "do NOT delete").
- **Leaving `fire_once_session` alone is deliberate, not laziness.** Native `fire-once'` keeps populated
  alpha, which is exactly what the oracle's `fire-once` does — so the single-pass pair stays *aligned*,
  and the 2b probe keeps a truthful home. Narrowing the cut is what makes the alignment total instead of
  trading one mismatch for another.

⚠ **A trap:** `kernel.rs:3206` runs the four passes inline *specifically to avoid* `fire_once_session`,
because that fn clears beta before freeze. Deliberate. Leave it.

## The consumer checks (re-verified, v2)

| check | verdict | ground |
|---|---|---|
| **a — any reader?** | ONE | `probe_arc278_2b_insert_alpha.wat`, 3 sites, fires via `fire-rules` |
| **b — whole-Session differentials?** | **none** | all ~24 oracle-vs-native probes compare COUNTS/SUMS; a grep for Session equality returns nothing |
| **c — EXPLAIN / snapshot / query?** | **none** | `Session/alpha-memory` at exactly **7 sites tree-wide**: 3 in that probe, 4 in `rete.wat` |

## Re-pointing 2b — to the single-pass verb, where alpha is real

Its three entries fire via `fire-rules` then read alpha. Re-point to **`fire-once'`** (native single-pass,
`runtime.rs:4700`), which retains populated alpha under this stone's narrowed cut.

This is a *better* home than `fire-rules` was: 2b is stone **2b — insert/alpha-activation**, a
**single-pass** property. Firing to fixpoint to assert a single-pass fact was always more machinery than
the claim needed. The rule under test has an empty RHS, so nothing cascades and single-pass ≡ fixpoint
for these three assertions.

## The RED gate — a differential, with a non-vacuity anchor

`probe_arc278_alpha_is_fire_scoped`, five assertions:

1. `native-alpha-key-count` (via `fire-rules`) **== 0**
2. `oracle-alpha-key-count` (via `fire-rules-spec`) **== 0**
3. **1 == 2** — the divergence is closed; native and oracle now agree on alpha
4. `single-pass-alpha-key-count` (via `fire-once'`) **> 0** — **the anchor**: this workload genuinely
   populates alpha, so assertions 1–3 are not vacuously true over a no-match workload
5. `native-derived-count` **==** `oracle-derived-count`, both **> 0** — the RESULT is untouched

Red today on (1). Without (4), the whole gate passes just as green over a workload that matches nothing —
that is the difference between a measurement and a claim (R59 `NISI FRANGAS, NIHIL PROBAS`).

## Blast radius

`src/rete/kernel/` (**one** clear + the stale `round_trip_fired_session` comment, now `kernel/tests/pass_semantics.rs`), `wat/rete.wat` (the
`Session` doc comment only — mark alpha/beta fire-scoped, mirroring `Support`'s *"EPHEMERAL — carried
only in Explained"*), `tests/rete/probe_arc278_2b_insert_alpha.{wat,rs}`, and the new gate pair.
**No corpus migration. No codemod. No oracle logic touched.**

## Out of scope = REJECTED (affirmative cuts)

- **Clearing alpha in `fire_once_session`.** Would break the single-pass alignment and orphan 2b.
- **Deleting `alpha-memory`/`beta-memory` from the `Session` record.** The oracle's `fire-once` fills
  both; removing a public field is a larger stone with a corpus surface.
- **"Fixing" `fire-stratified` to carry alpha.** It is the oracle. The oracle is never optimized *and
  never adjusted to suit the kernel* — the kernel matches it, not the reverse.
- **Clearing `production`.** It IS the result.
