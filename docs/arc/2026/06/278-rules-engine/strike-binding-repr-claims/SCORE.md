# SCORE — C5, weighed against the orchestrator's own re-run

> Re-run at `bd5d268c7` + the rider's tree. **The strike landed. The sharpest finding is that my
> DESIGN asserted a reachability claim in bold that is FALSE — and acting on it would have proposed
> reaping live code.**

## The scorecard

| # | required | result, MY re-run |
|---|---|---|
| 1 | ★ the floor assertion can fail | ✅ `> 0` + a row-count check + three orderings, every message carrying the table |
| 2 | tests name the live repr | ✅ both named tests head with `BindSpan` / `session.rs:64` (⚠ the count was wrong — see C) |
| 3 | purpose stated | ✅ named as evidence for `fire/delta.rs:725-726` |
| 4 | the `163 ns` anchor gone | ✅ 4 → **1** mention, and the survivor at `:38` records *why it was dropped* |
| 5 | the collapse recorded | ✅ sign + samples recorded, no number pinned |
| 6 | arms survive | ✅ nothing deleted |
| 7 | matcher path untouched | ✅ |
| 8 | radius | ✅ `binding_repr_bench.rs` only, +117 −9 |
| 9 | lints | ✅ 210/210 |
| 10 | clippy | ✅ rc=0 |
| — | floor | ✅ **`5327 tests run: 5327 passed, 21 skipped`**, exit=0 |

## ⛔⛔ A — MY DESIGN'S BOLD CLAIM IS FALSE, AND IT WOULD HAVE LICENSED REAPING LIVE CODE

DESIGN.md states, in bold: *"**The whole matcher evaluation path is reached only from tests.**"* I
reached that by counting Rust callers of `alpha_match_inner{,_local,_seeded}` under `src/`, finding
0/0/1, and confirming the one was inside `#[cfg(test)] mod tests`.

**I never grepped the `.wat` corpus.** Verified myself:

```
src/rete/matcher.rs:278   const OP: &str = ":wat::rete::alpha-match";
src/rete/matcher.rs:357   const OP: &str = ":wat::rete::alpha-match-under";

wat/rete/oracle/pass.wat:21,22,193,305,551,563,661   alpha-match / -local / -under
wat/rete/oracle/accum-pass.wat:247                   alpha-match-under
```

Those functions are the **bodies of registered wat primitives**, and the wat oracle calls them. In a
repo whose entire premise is that wat calls into Rust primitives, **a Rust-caller count is not a
reachability proof.** The true statement is narrower and still supports the strike: the path is off
the *native round loop* (`compiled_cond.rs:912`), and it is the *wat oracle's interpreter*.

⛔ **C13, as I wrote it, was dangerous.** It called the matcher path *"a genuine `purgare` target — or
a deliberately kept differential oracle"*, framing a live, wat-reachable interpreter as possibly
reapable on the strength of a count that could not see its callers. **C13 is corrected to a
non-question.**

## ⭐ B — THE RIDER REFUSED TO SHIP A FLAKE MY SKETCH WOULD HAVE PRODUCED

My brief said *"plus whatever orderings must hold"*. The natural set is four — both operations at both
ends. One of them does not clear the noise: **EXTEND at the smallest cardinality ran 1.19–2.02× over
twelve drives**, and a 19% floor sits inside the ~16% my own EXPECTATIONS declares absolutes
reproduce to. A rider following the sketch literally would have manufactured a flake **into a repo
that bans known flakes absolutely**, in the very file whose sibling `binding_key_cost` is `#[ignore]`d
for exactly that reason. It dropped that ordering, kept the three that measured ≥2.6×, and recorded
the drives and the reasoning in the code.

## ⛔ C — TWO MORE BRIEF DEFECTS, BOTH MINE

1. **"−22 ns/fact" does not reproduce.** Three drives: **−2, −4, −10 ns/fact**; the one-bind arm alone
   spans 14.89–15.29 ms, so every delta is smaller than that spread. The **sign** held, so the
   conclusion survives — but I quoted a single sample as a fact in a brief and a work-list row, one
   day after promoting *the noise floor is measured, not assumed*. The rider recorded the sign and the
   samples rather than pinning a number, which is the only honest form.
2. **"Both floor tests in this file" is off by one.** There are **three** unignored tests:
   `bind_key_construction_vs_map_operation`, `binding_cardinality_distribution` (`:390`), and
   `token_bindings_representation_dominance`. The two I named were the right two to head; the count
   in the brief and in EXPECTATIONS row 2 was wrong.

## ⭐ D — MUTATION 1 DRIVEN BY ME, BY A DIFFERENT METHOD

The rider zeroed the counters *by measurement* (slowing the array twin 8× so they legitimately reach
zero). I zeroed them **directly**, to test a different thing — that the assertion is wired to those
variables at all:

```
panicked at src/rete/kernel/tests/binding_repr_bench.rs:726:5:
the probe measured NOTHING — across 8 cardinalities and both operations the array representation
did not come out ahead in a single cell...
Summary [0.105s] 1 test run: 0 passed, 1 failed, 5347 skipped
```

Both methods red. Restored; `diff` against the saved copy is empty.

## ⚠ E — AND MUTATION 2 SHOWS THE OLD ASSERTION WOULD HAVE PASSED THE VERDICT INVERTING

The rider's second mutation slowed the trie extend arm, and the table printed
**`DOMINANCE (array wins EVERY cardinality on extend): YES`** — the opposite of R60's recorded
conclusion — while the *old* `< usize::MAX` assertion passed green. The new ordering caught it. That
is the clearest statement of what this class costs: the verdict line can invert and the gate does not
notice.

## ⚠ F — A DISCLOSURE THE RIDER VOLUNTEERED

Assertion (1) (`> 0`) is **logically implied** by assertion (2) (small-end GET): if the array wins GET
at card 1 the sum is necessarily > 0, so (1) cannot fail while (2) passes. It fires first and states
the whole-table condition — it is the check my scorecard required — but it is **not independently
falsifiable**, and the code comment says so instead of implying otherwise. Disclosed, not discovered.

## Per-arm status

| arm | status |
|---|---|
| apportionment (a)/(b)/(c) | **proven** — all three timed every run, non-zero |
| dominance: 4 timed cells × 8 cardinalities | **proven** — small-end GET and both large-end cells asserted |
| dominance: EXTEND at smallest cardinality | **driven, deliberately NOT asserted** — 1.19–2.02× over 12 drives, inside the noise |
| `binding_key_cost`, `binding_repr_microbench` | **reachable, not driven** — `#[ignore]`, untouched |
| the live `BindSpan` path | **not reachable from this file** — it builds `Value`/`Arc`/rpds directly and names zero symbols from its host module; naming it in the header is the most this file can do |
| retired matcher path | **untouched** — and NOT test-only, see A |
