# SCORE — the store injector has teeth

**SCORED.** Executor: grok, 2026-09-13, branch `sns-sqs`, HEAD `7d3401fd2` (DRAWN). Did not commit.

```
     Summary [ 533.903s] 5244 tests run: 5244 passed (9 slow), 22 skipped
```

`.floor/2026-09-13T08-50-43Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5241 → 5244** (+3 tests). Not a shrink.
Clippy: **CLIPPY=0**. `cargo nextest run --release --no-run` → **NORUN=0**.
`git diff --stat -- src/ wat/ wat-scripts/` **EMPTY**. Harness only.

---

## ⭑ THE HEADLINE — `n == real-rows` is on the floor

`grep -rln faulting-store tests/**/*.rs` → **1 file**: `tests/services/probe_store_injector_has_teeth.rs` (was 0).

Three tests, three entry points already on the probe. Probe and proxy untouched.

| test | isolated | on floor (`nice -n 19`) | vs 40 s wall |
|---|---|---|---|
| passthrough | 0.69 s | 1.57 s | **25×** headroom |
| ⭑ drop-reply | 10.72 s | 11.77 s | **3.4×** headroom |
| die | 10.72 s | 11.70 s | **3.4×** headroom |

Not one 20.7 s test. Parallel wall cost is the largest (~12 s), which sits under `every_wat_scripts_file_loads` (~535 s). Flag: nothing under 1.5×.

---

## Relations — 3× identical

```
passthrough  send=Accepted 1  real-rows=1
drop-reply   send=Accepted 1  real-rows=1  drops-fired=1
die          send=TimedOut    real-rows=1
```

elapsed-ms moved (3–4, 10003–10009). Not asserted. `drops-fired` asserted `> 0` only.

Die path: the queue's scan raises `"queue: probe scan failed — peer is dead, not a broken pipe"` (expected; SCORE of `04726854e`). The client still returns `TimedOut` and `real-rows=1`.

First isolated run failed the **harness parser** (`passthrough send=` was read as key `passthrough send`). The printed strings already held the equalities. Parser fixed to take the last token before `=`. Not a flake of the invariant. Did not re-run that log as a green.

---

## Floor-time delta vs 528.833 s

This floor **533.903 s**. Delta **+5.1 s**, inside the ~6 s band.

The guard returned ~30 s. This spends **none of it on the wall clock**: the 11.8 s tests finish while `every_wat_scripts_file_loads` is still running.

---

## WHAT LANDED

`tests/services/probe_store_injector_has_teeth.rs` — `startup_from_source` + `FsLoader`, `apply_function` on `:sf::run-passthrough` / `:sf::run-timedout` / `:sf::run-die`.

Happy: `distinct=8000;dup=0`.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭑⭑ grep drives the proxy | **1 file** |
| 2 | ⭑⭑ `n == real-rows` drop-reply | equality, passing, 3× |
| 3 | same on passthrough | equality, passing, 3× |
| 4 | `drops-fired > 0` | on drop-reply only |
| 5 | die TimedOut and real-rows==1 | both, 3× |
| 6 | three tests | 3; `test(store_injector)` |
| 7 | runtime vs 40 s wall | 25× / 3.4× / 3.4×; none under 1.5× |
| 8 | no count pins | only `drops-fired > 0` |
| 9 | probe/proxy unchanged | wat-scripts **EMPTY** |
| 10 | no src/wat | **EMPTY** |
| 11 | 3× identical relations | above; elapsed-ms moved |
| 12 | floor count moved | **5244** |
| 13 | clippy | 0 |
| 14 | floor-time delta | +5.1 s vs 528.833; spends none of the guard's 30 s |
| 15 | happy | `distinct=8000;dup=0` |

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-13

**Graded against my OWN reads and runs**, from `7d3401fd2` + the working tree.

```
floor   (mine, final)  Summary [ 549.656s] 5244 tests run: 5244 passed, 22 skipped · 0 failure tokens · no ARM.txt
clippy  0        happy  distinct=8000;dup=0        tree: harness only, src/ wat/ wat-scripts/ all EMPTY
```

**STRUCK — and MY FLOOR WENT RED TWICE, both times because of me. Both reds are on disk with their `ARM.txt`.**

## ⛔⛔ RED 1 — I gated an OBSERVATION as an invariant. Again.

`.floor/2026-09-13T09-04-47Z/` — **`ARM.txt` kept, not re-run away:**

```
FAIL (3690/5244) probe_store_injector_has_teeth::store_injector_die_is_timedout_and_write_landed
  die send=Lost;elapsed-ms=20;real-rows=1
  assertion left == right failed    left: "Lost"    right: "TimedOut"
  stderr: "queue: redial failed — peer is dead, not a broken pipe"
```

**My EXPECTATIONS row 5 said `assert_eq!(send, "TimedOut")`.** I took that from D1-c's *one observed* probe
output and pinned it. It is **a load-dependent race**, measured:

```
quiet box, via :user::main      6/6   TimedOut @10000ms
quiet box, this test alone      4/4   TimedOut @10000ms
8 spinners, this test alone     1/4   Lost @5ms
the floor (5244 tests, nice-19)        Lost @20ms
```

★ Both outcomes are correct consequences of the proxy exiting after forwarding: the queue redials, fails,
raises, and dies — so the client either **learns the queue is gone (`Lost`)** or **gives up first (`TimedOut`)**.

⛔ **This is the third time this session I have gated an observation as an invariant** — after `store-calls
"unchanged"` and a two-point `rt-store` band — and the second time a **load-dependent** race caught me, after
`stop-means-gone`. The lesson is in my memory store and I did it anyway.

**The invariant across all 15 observations:** the write landed, and the client never saw a success. That is what
the assertion says now.

## ⛔⛔ RED 2 — MY FIX tripped a lint I had QUOTED EARLIER IN THIS SESSION

`.floor/2026-09-13T09-23-16Z/` — `ARM.txt` kept:

```
FAIL (77/5244) wat::lint no_loose_string_assert::tests_carry_no_loose_string_assert
  → tests/services/probe_store_injector_has_teeth.rs:122      (exactly ONE offender: mine)
```

My correction was `assert!(!send.starts_with("Accepted"))`. **`no_loose_string_assert` bans exactly that** —
and earlier in this very session I quoted its rule from `probe_ex001_fanout.rs`'s header into a BRIEF:
*"Completeness fields are byte-identical (`assert_eq!`, no `.contains(` — `no_loose_string_assert`)."*
**I cited the wall and then walked into it.**

★★ **And the gate made the fix better.** The tight form enumerates the two **measured** branches:

```rust
assert!(send == "Lost" || send == "TimedOut", …)
```

⭑ That is strictly stronger than a prefix test: a **new third outcome would red here**, where
`!starts_with("Accepted")` would have silently accepted it. The lint did not just enforce style — it improved
the assertion.

⚠ **Finding, reported not fixed:** that lint's header says *"Until then this is an expected-red test … a SECOND
red is a real regression."* **The backlog is at zero** — it reported only my line, and every earlier floor this
session was green with it. So the *"expected-red"* sentence is **stale prose** that would tell the next reader
to ignore a real red. One line; the lint owner's to rule.

## ⭑ The correction is verified THREE ways, not one

1. **Against the recorded red string** — `field("send")` of `die send=Lost;elapsed-ms=20;real-rows=1` is
   `"Lost"`: the old assert FAILS it, the new one PASSES. That uses the captured evidence rather than a
   hoped-for race.
2. **On a quiet box** — the `TimedOut` branch, passing.
3. ⭑ **On the final floor** — the `die` test ran in **1.632 s**, i.e. it drew the **`Lost`** branch, and passed.
   Both branches are now demonstrated green.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ a floor test drives the proxy | ✅ 1 file, was **0** |
| 2 | ⭑⭑ `n == real-rows` on drop-reply | ✅ **`assert_eq!(n, rows)`** in source; passing 3× on my runs |
| 3 | same on passthrough | ✅ `assert_eq!(n, rows)` |
| 4 | fault provably fired | ✅ `drops > 0` — the only threshold in the file |
| 5 | die path | ⛔→✅ **row corrected by me.** It pinned one branch of a race; now `real-rows == 1` plus the two measured outcomes |
| 6 | three tests | ✅ my runs: 3 tests, 3 passed, ×3 (10.69/10.70/10.74 s, spread 0.05 s) |
| 7 | runtime vs the 40 s wall | ✅ 1.6 s / 1.7 s / 11.7 s on the floor — **3.4× worst-case headroom**, none under 1.5× |
| 8 | no count pins | ✅ `drops > 0` only; everything else an equality |
| 9 | probe/proxy unchanged | ✅ `wat-scripts/` EMPTY |
| 10 | no `src/` or `wat/` | ✅ EMPTY |
| 11 | 3× identical relations | ✅ mine |
| 12 | floor count moved | ✅ **5244** |
| 13 | clippy | ✅ 0 |
| 14 | floor-time delta | ⚠ **not cleanly measurable from single runs** — grok +5.1 s, mine +20.8 s against a ~6 s band. The honest statement: the largest new test is 11.7 s and runs in **parallel** with the 535 s clock test, so the expected wall impact is ~0, and neither single-run delta separates from noise |
| 15 | happy path | ✅ `distinct=8000;dup=0` |

## What I'd credit above all

**The strike distinguished a parser bug from an invariant failure** — its first isolated run failed because
`field()` read `passthrough send` as the key, and it said so plainly: *"The printed strings already held the
equalities. Not a flake of the invariant. Did not re-run that log as a green."* That is exactly the
distinction that makes a red useful.

## What this hands the builder

Both named items are closed. `Accepted n == real-rows` is now on the floor. ⛔ Two things I found and did not
fix: the **stale "expected-red" sentence** in `no_loose_string_assert`'s header, and **D4** — still undrawn,
now the only one of the five rulings outstanding.
