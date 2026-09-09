# SCORE — the drain gives up on a stall, not a budget

**SCORED.** Executor: claude, 2026-09-09, branch `sns-sqs`, HEAD `2e04ee9ef`. Did not commit.
**No STOP fired.** **One functional file modified — `wat-scripts/fanout/circuit.wat` — and nothing
else.** No `wat/`, no `src/`, no `sqs.wat`, no `sns-fanout.wat`, no `.config/nextest.toml`, no test
file. `require!` untouched.

```
     Summary [ 482.912s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-09T21-54-25Z/` — `scripts/floor.sh` exit `0`, **no `ARM.txt`**, **zero**
`FAIL`/`TRY`/`TIMEOUT`/`ABORT`/`SIGSEGV` lines in `clean.log`. Run **once**, on the final tree, after
every timing run, with the box otherwise quiet. **Nothing was re-run.** `5237` unchanged — no test
added, removed or renamed.

---

## ⭑ THE HEADLINE — the drain is BIMODAL, and the old budget died in the second mode

The scenario the stone was drawn around does not have one drain time. It has two, and they differ by
**70×**:

| mode | what the poller sees | before this stone | after |
|---|---|---|---|
| **fast** | drains in ≤3 polls | `drain` 71–186 ms, PASS | `drain` 68–224 ms, PASS |
| **slow** | ~5.2 s in which **no ack lands anywhere in the system**, then it drains | budget exhausted at 100 attempts / 1.76–2.45 s → **`drained-never`, exit 2** | `drain` 5142–5311 ms, **PASS**, `distinct=100 dup=0` |

The slow mode's incidence is **unchanged by the change**: 4 of 11 tiny runs before (all 4 failed),
4 of 11 tiny runs after (all 4 passed). The stone did not make the system faster or the mode rarer —
it stopped the *observer* from quitting 2.7 s into a 5.2 s wait and calling it a system failure.

★ And the instrument that says so is the one the DESIGN said was already in hand: `drain-stale-max`,
the longest run of consecutive polls with no Σacks movement, now on the report line.
**191 / 197 idle, 284 / 289 under 8-way load, 0 on every fast-mode and every write run.**

---

## ⭑⭑ THE CONSTANTS, AND WHY MY REASONING WAS WORTH LESS THAN ONE RUN

### `K = :fanout::drain-stale-polls = 600` polls

I first reasoned it: worst legitimate ack-free gap = visibility expiry (200 ms) + the inbox's 250 ms
receive wait + the sub's 250 ms receive wait ≈ 1.5 s ≈ 70 polls; picked **200** for ~3× margin.

**The first instrumented run reported `drain-stale-max=197` — on a run that PASSED.** My chosen
constant would have red-flagged a correct drain by three polls. The measured worst legitimate streak
is **289** (8-way contention), i.e. **4× my estimate**. `K = 600` is **2.1×** the largest streak yet
observed and ~11–15 s of wall silence at the measured 18.5–25.7 ms/poll.

⚠ **A second guess died in the same data.** I wrote in the code that a wall-clock gap "spans FEWER
polls under load, so a poll-counted K is self-widening". Measured, the opposite: the gap stayed
~5.2 s while the poll got *cheaper* under load (25.7 ms idle → 18.5 ms at 8-way), so the streak grew
**197 → 289**. The comment now says that, with the numbers. **K in polls is not self-widening; it is
a number that has to be checked against `drain-stale-max=`, which is why that field exists.**

What actually makes the loop load-independent is not K at all — it is that the loop no longer gives
up *while deliveries are still landing*.

### `CEILING = :fanout::drain-ceiling-ms(pairs) = 30000 + 12 × pairs` ms

The unconditional backstop for the one world the stall arm cannot catch: a system that keeps acking
(redelivery churn) and never drains. Bounded on **both** sides, which is the part worth stating:

- **below, by the work** — 30 s is ~5.7× the longest legitimate no-progress gap measured (5.3 s);
  12 ms/pair is ~40× the healthy write drain's measured per-pair cost (110 polls × 22 ms / 8000
  pairs ≈ 0.3 ms/pair). Healthy drains measured: 68–224 ms (100 pairs), 2465–2478 ms (8000 pairs).
- **above, by the runner** — a ceiling-hitting drain must still **print its verdict** inside
  nextest's kill for that scenario, or the arm is destroyed and we are back to arc 278's empty
  TIMEOUT. 8000 pairs → 126 s, inside `r2_drop_*`'s 90/180 s override with ~25 s of the rest around
  it.
- **above K's wall window at every scale**, or a genuine stall would be mislabelled a timeout.
  Measured, not projected: a total stall reaches the stall arm at **600 polls / 13172 ms**, under the
  30 s floor.

⚠ **The tightest constraint in the system is the tiny drop cells' 30 s nextest kill, and it is now
close.** An honest stall window for this scenario cannot be shorter than ~5.3 s of silence, and with
3× margin that is ~15 s; those cells carry ~11 s of setup/collect. They fit today (measured below,
16.9 s worst) but a wedge would land at ~28 s. **I did not touch `.config/nextest.toml`** — that is
outside the stated radius and it belongs with the next stone's ruling on those cells' ignore status.
Named, not fixed.

---

## THE ROWS

| # | what | result |
|---|---|---|
| 1 | ⛔ **the happy path is untouched** | ✅ **PASS ×3.** `circuit.wat 2000 4 3 8192 true 1000`: `distinct=8000`, `dup=0`, inbox `accepted=2000`, no raise, `rc=0` on all three. `drain` 2478/2468/2465 ms vs before 2463/2453/2484 (**+0.15 % on the mean — unreadable**, see row 13). `poll-calls` 345/330/330 vs 330/330/330. `drain-stale-max=0` on all three: the write path never comes close to K |
| 2 | ⛔ **THE ASYMMETRY IS GONE** | ✅ **PASS — and the before side was worse than banked.** Alone ×3: **1 FAIL / 2 PASS before → 3 PASS after.** 8-concurrent: **3 FAIL / 5 PASS before → 8 PASS after.** Same verdict (empty) in both conditions, `distinct=100 dup=0` on all 11 post-change runs. Numbers below |
| 3 | ⛔ **a real stall is still caught** | ✅ **PASS — EXECUTED, not reasoned.** `circuit.wat 5 1 0 32 false 0` (j=0: no consumers exist, nothing can ever ack) → exit **2**, `drained-stalled: no delivery progress in 600 polls; last=[0/0] outbox=5 acks=0 polls=601 elapsed=13172`, raised at `circuit.wat:2450 :fanout::require!`. Argument below |
| 4 | ★ **the two worlds are distinguishable** | ✅ **PASS, both arms executed.** `drained-stalled …` and `drained-timeout …` are distinct strings and each names `acks`, `outbox`, `elapsed` (plus `polls`, and `stale`/`stale-max` on the timeout). The timeout arm was witnessed by temporarily shrinking the ceiling — verbatim below — because an unexecuted `format` is an unverified `format` |
| 5 | ★ **zero extra round-trips** | ✅ **PASS.** `:fanout::sweep-acks` folds `third` over the sweep **already taken**; it makes no `Queue/stats` call of its own. The per-poll round-trip count is unchanged and so is its accounting: `rts' = rts + (count qclients) + 1`, byte-identical to before. Live: fast-mode `poll-calls` 6–9 before and after for the same 2–3 polls |
| 6 | ★ **`acks` for stall, never completion** | ✅ **PASS.** The drained test is still `(and (:fanout::sweep-drained? sweep) (= box 0))`, unchanged. `acks` appears in the loop only as `(:wat::i64::> acks acks-prev)` and inside format strings. **No equality test on acks anywhere** — `grep` for `=`-on-acks in `circuit.wat` returns nothing |
| 7 | **the corpus loads** | ✅ **PASS.** `wat::lint wat_scripts_fixes_load::every_wat_scripts_file_loads_on_the_current_runtime` **PASS [482.905s]**, inside the green floor. No file was added to `wat-scripts/` (see "the scratch probe that could not exist", below) |
| 8 | **the floor holds** | ✅ **PASS.** `Summary [ 482.912s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — **0 FAIL, 0 TIMEOUT**, exit 0, no `ARM.txt`. The 7 SLOW warnings are the same seven as the two pre-change floors today (`480.935s` and `488.615s`); the two circuit tests inside them sat at **19.4/20.2 s after vs 19.6/21.0 s and 19.6/19.6 s before** — no shift |
| 9 | **the five passing chaos scenarios still pass** | ✅ **PASS, and the two failing ones now pass too.** `Summary [ 41.883s] 7 tests run: 7 passed (2 slow), 5252 skipped` — was `5 passed, 2 failed`. ⚠ They got slower and two crossed the 15 s SLOW warning; durations and the caveat below |
| 10 | **blast radius** | ✅ **PASS.** `git status --porcelain` → `M wat-scripts/fanout/circuit.wat` plus this SCORE. Nothing was forced elsewhere: the compiler and the corpus gate were the census and neither named a second file |
| 11 | ★★ **`acks` is monotone and already in the reply** | ✅ **CONFIRMED FROM SOURCE.** STOP-1 did not fire. Citations below |
| 12 | ⚠ **the 9–19 stuck entries are NOT explained** | ✅ **honoured.** This SCORE makes no claim that nothing is lost. What the runs show, and its limits, below |
| 13 | ⚠ **`drain` timings move** | ✅ **re-baselined on this box today.** Write path: **no readable change** (+0.15 %, under the ~1 % floor). Tiny path: the fast mode is unchanged; a **new slow mode** appears where a failure used to be. Not a regression — a different instrument |

---

## ⭑⭑ ROW 2 — THE ASYMMETRY, MEASURED BOTH WAYS ON THIS BOX TODAY

`./target/release/wat wat-scripts/fanout/circuit.wat 50 2 2 32 false 0 0 1000 42`, byte-identical
command on both sides, each run under `./scripts/capped.sh --limit 2g`. Before-side taken minutes
before the edit, tree clean at `2e04ee9ef`.

### BEFORE — alone ×3

| run | rc | `drain` | `poll-calls` | verdict |
|---|---|---|---|---|
| 1 | **2** | — | — | `drained-never: last=[0/0][0/0] outbox=9 attempts=100 elapsed=2449` |
| 2 | 0 | 71 ms | 6 | *(empty)* |
| 3 | 0 | 76 ms | 6 | *(empty)* |

⚠ **The BRIEF's premise — "passes alone" — is FALSE on this box today.** 1 of 3 alone runs failed
before the change. The banked asymmetry understated the problem; the scenario was already failing on
a quiet box roughly a third of the time.

### BEFORE — 8 concurrent

| run | rc | `drain` | verdict |
|---|---|---|---|
| 1 | **2** | — | `drained-never: … outbox=9 attempts=100 elapsed=1790` |
| 2 | **2** | — | `drained-never: … outbox=19 attempts=100 elapsed=1900` |
| 5 | **2** | — | `drained-never: … outbox=10 attempts=100 elapsed=1760` |
| 3, 4, 6, 7, 8 | 0 | 160 / 108 / 140 / 128 / 186 ms | *(empty)* |

**3 of 8 FAIL** — consistent with the banked 3–4 of 8. `outbox` 9 / 10 / 19 at the same seed,
reproducing the FINDING's refutation of the seed hypothesis.

### AFTER — alone ×3, then 8 concurrent

| condition | run | rc | `drain` | `poll-calls` | `drain-stale-max` | `distinct` | `dup` | `seen-skipped` |
|---|---|---|---|---|---|---|---|---|
| alone | 1 | 0 | **5142** | 600 | **197** | 100 | 0 | 9 |
| alone | 2 | 0 | 68 | 6 | 0 | 100 | 0 | 0 |
| alone | 3 | 0 | **5166** | 582 | **191** | 100 | 0 | 0 |
| 8-way | 1 | 0 | **5299** | 861 | **284** | 100 | 0 | 9 |
| 8-way | 2 | 0 | 97 | 6 | 0 | 100 | 0 | 0 |
| 8-way | 3 | 0 | 121 | 6 | 0 | 100 | 0 | 0 |
| 8-way | 4 | 0 | 197 | 6 | 0 | 100 | 0 | 0 |
| 8-way | 5 | 0 | **5311** | 879 | **289** | 100 | 0 | 9 |
| 8-way | 6 | 0 | 140 | 9 | 0 | 100 | 0 | 0 |
| 8-way | 7 | 0 | 128 | 6 | 0 | 100 | 0 | 0 |
| 8-way | 8 | 0 | 224 | 9 | 0 | 100 | 0 | 0 |

**11 of 11 pass. `distinct=100`, `dup=0`, `empty=1` on every one.** The same scenario reaches the
same verdict — the empty one — whether the box is idle or carrying eight copies of itself.
**STOP-2 did not fire.**

★ The two conditions are now separated by *how long the poller waits*, not by *whether it gives up*:
the slow mode costs 5.14–5.17 s alone and 5.30–5.31 s at 8-way. **The wall gap barely moves under
8× load** — it is a wall-clock mechanism in the system, not a CPU-starvation artifact.

---

## ⛔ ROW 3 — HOW I CONVINCED MYSELF THE CHECK CAN STILL GO RED

Making a check load-tolerant is exactly how a check becomes unfailable, so I did not want an
argument alone. There are three, in increasing order of weight.

**1. The structure.** Exactly one path in `poll-until-drained*` returns `""`, and it is the
*unchanged* completion test, `(and (sweep-drained? sweep) (= box 0))`. Every other path returns a
non-empty string. There is no arm that says "give up and call it fine".

**2. Both give-ups are unconditional, and the second does not depend on progress at all.** The stall
arm fires at K consecutive no-progress polls. The ceiling arm fires at `elapsed >= ceiling-ms`
**regardless of progress** — so the one world the stall arm cannot see (a system that keeps acking
forever via redelivery churn and never drains) is still bounded, and still red. A run cannot loop
forever, and cannot exit clean without draining.

**3. I built the failure and watched it fail.** `circuit.wat 5 1 0 32 false 0` — `j=0`, so **no
consumer worker exists at all**; nothing can ever receive or ack, Σacks is pinned at 0, and
`sweep-drained?` can never hold. Verbatim:

```
#wat.kernel/AssertionFailure :message "drained-stalled: no delivery progress in 600 polls;
  last=[0/0] outbox=5 acks=0 polls=601 elapsed=13172;check-exhausted=0;mark-exhausted=0;
  ack-retries=0;ack-exhausted=0"
  :location circuit.wat:2450  :fanout::require!
  :frames   circuit.wat:2702  :fanout::run-with   ← :user::main
```

`rc=2`. `acks=0` is the whole story in one field — the thing the old `drained-never` could never say.

⚠ **And the third arm, witnessed too, because an unexecuted `format` is an unverified `format`.** No
natural run reached the ceiling, so I temporarily set `drain-ceiling-ms` to `1000 + 0×pairs`, ran the
same j=0 stall, and restored the file (verified byte-identical by `md5sum`,
`e48b416754985c7090cd54762bda24b8`, before the floor's log was even read):

```
drained-timeout: ceiling 1000ms reached with no 600-poll stall; last=[0/0] outbox=5 acks=0
  polls=55 elapsed=1014 stale=54 stale-max=54
```

★ That output is also why the DESIGN's wording for this verdict is *not* in the code. The design
called it *"still progressing"*; here it fired on a system that was **totally stalled** and would
have printed a false sentence. The arm now says only what it knows — the ceiling was reached and no
K-poll stall was seen — and prints `stale`/`stale-max` so the reader can judge which world it is
rather than trusting a label. **`stale=54` in that line tells the truth the label could not.**

---

## ★★ ROW 11 — THE BUILDER'S PREMISE HOLDS. BOTH HALVES.

**Present in the reply `depth-of` already receives.** `:queue::Stats` is a 19-field flat record
(`sqs.wat:135–145`) with `acks` among them, and it is constructed at **exactly one site**
(`sqs.wat:1282–1302`, inside `:queue::Queue::StatsResponse::Ok`) — the same reply `depth-of` matches
and was discarding. `:fanout::tier-line` already read `(:queue::Stats/acks qst)` off it. **Zero extra
round-trips is a fact about the wire, not an estimate.**

**Monotone.** `Stats/acks` is `(:queue::Counters/acks cold)` at that one site. `:queue::Counters` is
constructed at **7** sites in `sqs.wat`:

| site | what it does to `acks` |
|---|---|
| `456` | `:acks 0` — process init, once |
| `1069`, `1129` | `(+ (:queue::Counters/acks cold) (:wat::core::count ids))` — the two ack arms |
| `510`, `569`, `910`, `1409` | `(:queue::Counters/acks cold)` — pass-through, unchanged |

`(count ids)` is a vector length, so it is ≥ 0. **`acks` never decreases for the life of a queue
process.** The only conceivable decrease is a process restart resetting to 0, and the loop treats a
decrease as *no progress* (conservative: it surfaces as a red, never as silence) — stated in the
code at the `stale'` binding.

**STOP-1 did not fire.**

---

## ⚠ ROW 12 — WHAT I AM NOT CLAIMING

The 9–19 entries are **not explained by this stone and this SCORE does not claim delivery is proven.**

What the runs *do* show, stated at exactly its own strength: on this box today, in the identical
command, the mode that used to fail at ~2 s **completed** — `distinct=100`, `dup=0` — in **4 of 4**
runs that entered it (2 alone, 2 under 8-way load), after ~5.2 s. That is evidence about **those four
runs**, in **that one scenario**, at **one seed**. It is not proof that the entries in the earlier
failures would have delivered, and it is not proof that no message is ever lost under 10 % ack-reply
loss. A `drained-timeout` verdict would have been weaker still; none was produced.

⚠ Two observations that are **not** a mechanism, recorded so the next stone does not have to
rediscover them:

- `seen-skipped=9` accompanies **three of the four** slow-mode runs (a duplicate delivery absorbed by
  `seen` — the redelivery path), and the fourth (`alone-3`, `drain-stale-max=191`) has
  `seen-skipped=0`. So "the stuck entries are the redelivered ones" **fits three runs and fails on
  the fourth.** I did not chase it.
- The gap is suspiciously **constant**: 5142 / 5166 / 5299 / 5311 ms across four independent runs at
  two very different load levels. That looks like a fixed timeout in the system, not queueing delay.
  **Nothing in this stone identifies it.** It is the obvious next question, and `drain-stale-max` is
  now the instrument for asking it.

---

## ⚠ ROW 13 — `drain` RE-BASELINED, NOT COMPARED

Fresh before/after on this box, this session. **The banked 1194 / 2506 / 5305 are not used.**

### Write path — `2000 4 3 8192 true 1000`, ×3 each side

| side | run 1 | run 2 | run 3 | mean |
|---|---|---|---|---|
| before | 2463 | 2453 | 2484 | 2466.7 ms |
| after | 2478 | 2468 | 2465 | 2470.3 ms |

**+0.15 %.** The instrument's own session-to-session floor is ~1 %, so this is **not readable** and
no claim is made in either direction. `setup` 12393–12486 both sides; `fill` 2598–2665 both sides.

### Tiny path — the mode split is the whole story

| mode | before | after |
|---|---|---|
| fast (≤3 polls) | 71 / 76 / 108 / 128 / 140 / 160 / 186 ms | 68 / 97 / 121 / 128 / 140 / 197 / 224 ms |
| slow | *did not exist — it was a failure at 1760–2449 ms* | 5142 / 5166 / 5299 / 5311 ms |

**The fast mode is unchanged** (bands overlap almost exactly). The slow mode is not a regression and
not a slowdown: **it is time that used to be spent failing.** Any future comparison of `drain` on
this scenario must state which mode it is quoting, or it is comparing two different things.

⚠ **Consequence for the chaos cells (row 9), named because it will bite someone later.** The seven:

| cell | before (FINDING, `109.817s` set) | after (`41.883s` set) |
|---|---|---|
| `drop_ack_tiny` | **FAIL** 10.027s | **PASS** 10.975s |
| `r2_drop_before_tiny` | **FAIL** 10.694s | **PASS** 16.871s ⚠ SLOW |
| `drop_check_tiny` | PASS ~8–10s | PASS 16.612s ⚠ SLOW |
| `drop_recv_tiny` | PASS ~8–10s | PASS 16.277s ⚠ SLOW |
| `r2_drop_after_tiny` | PASS ~8–10s | PASS 10.766s |
| `r2_drop_before_write` | PASS ~31s | PASS 40.696s |
| `r2_drop_after_write` | PASS ~31s | PASS 41.872s |

Per-cell durations across the two runs are **not** cleanly comparable — the whole set went 109.8 s →
41.9 s, so the contention each cell saw was different. What is comparable and matters: **two tiny
cells now cross nextest's 15 s slow warning against a 30 s kill**, because a run entering the slow
mode now waits it out instead of quitting at 2 s. Margin today is 13 s. Nothing timed out.

⚠ These seven still have **no assertions** (the FINDING's second finding): a PASS here means "the wat
did not raise", not "`distinct = n×m`". That is out of this stone's scope and unchanged.

---

## STOPS

| STOP | fired? | |
|---|---|---|
| **1** — `acks` not monotone / not in the reply | **NO** | Confirmed from source at 7 construction sites and 1 Stats site. Row 11 |
| **2** — the contended asymmetry survives | **NO** | 3 PASS alone, 8 PASS at 8-way, same empty verdict. Row 2 |
| **3** — `require!` or a second file forced | **NO** | `require!` untouched; `git status` shows one modified file. The compiler and the corpus gate were the census and named nothing else |
| **4** — a red in the floor or the corpus gate | **NO** | `5237 passed`, no `ARM.txt`, 0 `FAIL`/`TIMEOUT`. Nothing was re-run |

---

## THE EDIT SITES — the real count

**One file, 29 diff hunks, +162 / −45 lines, 12 definitions touched of which 3 are new.**

| # | definition | what changed |
|---|---|---|
| 1 | `:fanout::depth-of` | return `Tuple[i64 i64]` → `Tuple[i64 i64 i64]`, third = `(:queue::Stats/acks qst)`; sentinel `(-1 -1)` → `(-1 -1 -1)` |
| 2 | `:fanout::sweep-of` | element type widened (**4** occurrences) |
| 3 | `:fanout::snapshot-str` | element type widened (**2**); body unchanged, still prints `[v/u]` |
| 4 | `:fanout::sweep-acks` | **NEW** — Σ `third` over a sweep already taken. Shape copied from `:fanout::sum-store-calls`, minus its `Queue/stats` call |
| 5 | `:fanout::sweep-unread?` | element type widened (**2**) |
| 6 | `:fanout::sweep-drained?` | element type widened (**2**); predicate unchanged |
| 7 | `:fanout::sweep-filled?` | element type widened (**2**) |
| 8 | `:fanout::drain-stale-polls` | **NEW** — `600` |
| 9 | `:fanout::drain-ceiling-ms` | **NEW** — `30000 + 12 × pairs` |
| 10 | `:fanout::poll-until-drained*` | rewritten: `left`/`total` out, `ceiling-ms`/`acks-prev`/`stale`/`stale-max`/`polls` in; returns a 3-tuple; `drained-never` → `drained-stalled` + `drained-timeout` |
| 11 | `:fanout::poll-until-drained` | `attempts` → `pairs` (same `n×m` argument, spent as a ceiling); returns a 3-tuple |
| 12 | `:fanout::run-with` | 3 sites: the drain comment, the new `drain-stale-max` binding, and `drain-stale-max={dsm}` on the `phases` report line |

**13 type-widening occurrences** in total (1 in `depth-of`'s return + 12 across the five sweep
functions), plus 2 inside the new `sweep-acks`.

**Two callers the compiler did NOT force**, worth naming because the trap-door list warned they might
be: `:fanout::poll-until-visible-zero*` destructures `depth-of` with `first`/`second` only, and
`:fanout::poll-until-filled*` + the `fill-sweep` binding in `run-with` consume the sweep through the
predicates. All three compile unchanged against the wider tuple. **`poll-until-filled*` is still
attempt-bounded** — deliberately out of scope; this stone is about the drain.

⚠ **The scratch probe that could not exist.** I first wrote row 3's witness as
`wat-scripts/scratch-pad/probe-a-real-stall-still-goes-red.wat`, which `load-file!`d `circuit.wat` to
drive `poll-until-drained` directly against a queue with no consumer. It cannot work:
`circuit.wat:39` is `(:wat::config::set-redef! true)`, and the loader rejects a config setter in a
loaded file (`#wat.load/SetterInLoadedFile … setters belong in the entry file only`). **The file was
deleted, not left to rot** — the corpus gate type-checks everything under `wat-scripts/` and a probe
that cannot load is a red. The `j=0` route through the real CLI is strictly better anyway: it
exercises the production path rather than a scratch re-wiring of it.

---

## THE BOX

12 CPUs, load average 0.02 before the first measurement. `ps -eo args | grep -E 'cargo|nextest|
release/wat'` showed only two `wat --mcp` servers before every timing run and before the floor. Every
heavy command went through `./scripts/capped.sh` (`--limit 2g` for tiny runs and each of the 8
concurrent copies, `--limit 4g` for the n=2000 runs, `scripts/floor.sh`'s own cap for the floor).
Nothing was run beside the floor. Exit codes were read directly, never through a pipe.

## BLAST RADIUS

```
 M wat-scripts/fanout/circuit.wat        (+162 / −45, 29 hunks)
?? docs/excursus/2026/08/001-sns-sqs/the-drain-gives-up-on-a-stall-not-a-budget/SCORE.md
```

Nothing else. **Everything left uncommitted.**

---

## WHAT THE NEXT STONE INHERITS

1. **The ~5.2 s gap is unidentified**, and it is now the most interesting number in the system: four
   runs, two load regimes, 5142–5311 ms. `drain-stale-max` is the instrument for finding it.
2. **The tiny chaos cells' 30 s kill is the binding constraint** on how honest a stall verdict can
   be at that scale (an honest window is ~15 s; the cells carry ~11 s of other work). Their ignore
   status, their missing assertions, and possibly a nextest override are one ruling.
3. **`poll-until-filled*` is still attempt-bounded** — the same defect, in the fill phase, with the
   same free instrument (`sends-accepted` rides the same reply). Not touched.
4. **The inbox is still hardcoded to zero drops** (`circuit.wat`'s tier-1 queue record). Still the
   highest-value item on the list, and still untouched by this stone.

---

# ⭑ THE ORCHESTRATOR'S GRADING — my own runs, my own reads

**Floor from `.floor/2026-09-09T21-54-25Z/clean.log`:**
`Summary [ 482.912s] 5237 tests run: 5237 passed (7 slow), 22 skipped` — 0 lines matching
`FAIL|TRY|TIMEOUT|ABORT|SIGSEGV`, no `ARM.txt`.

**Row 3 — the row I was most afraid of — verified by running it myself.** A load-tolerant check that
can no longer fail would be worse than the ambiguous one it replaced:

```
circuit.wat 5 1 0 32 false 0     →  rc=2
drained-stalled: no delivery progress in 600 polls; last=[0/0] outbox=5 acks=0 polls=601 elapsed=11576
```

**The gate still goes red on a genuine stall.** Row 6 verified too: no equality test on `acks`
anywhere, and `circuit.wat:1100` states the contract in a comment.

**Row 2 — the asymmetry, my own runs: 11/11 pass.**

```
alone ×3   rc 0 0 0    distinct=100 dup=0
8-way      rc 0 0 0 0 0 0 0 0     failures 0/8, no drained-* verdict anywhere
```

## ⭑⭑⭑ THE ~5.2 s GAP IS IDENTIFIED — it is one hardcoded constant

The SCORE above records the bimodality and says the gap is *"unidentified… looks like a fixed
timeout, not queueing."* It is a fixed timeout, and my runs make the case unmissable:

```
alone   drain =   78   5190     71
8-way   drain =  149  159  165  5255  5283  5300  5305  5313
```

Two clusters with **nothing between**. Six slow-mode samples spanning both load regimes span only
**123 ms** (5190–5313). Queueing spreads; a constant does not.

**The constant:**

```
circuit.wat:2323   (:demo::mk-tw 5000000000 …)      5,000,000,000 ns = exactly 5 s
                   → :demo::topic-worker :durable [vis-ns …]   (sns-fanout.wat:249)
                   → used as the inbox receive's :visibility-ns (sns-fanout.wat:412)
```

★★ **There are TWO visibility timeouts and only one is reachable:**

| | value | set where | used where |
|---|---|---|---|
| **sub queues** (consumers) | `vis-ms`, 200 ms under drops | `circuit.wat:2255-2260` | `circuit.wat:471`, `:943` |
| **inbox** (topic-worker) | **hardcoded 5 s** | `circuit.wat:2323` | `sns-fanout.wat:412` |

## ⛔ Which corrects MY OWN refutation of H3

In `the-chaos-gate-does-not-exist/FINDING.md` I recorded:

> *"⛔ Hypothesis 3 — the budget cannot cover redelivery latency — REFUTED. `vis-ms` 200/50/20 gives
> 4/8, 4/8, 3/8. Redelivery latency is not the variable."*

**The measurement was right and the conclusion was wrong.** Redelivery latency *is* the variable — I
varied the **sub queues'** visibility while the gap lives in the **inbox's**, which `vis-ms` does not
touch. H3 was never tested. ★ Fifth instance today of the same family: the instrument answered a
different question than the one I asked, and the output looked identical either way.

## ⭑ So the whole chain is now closed, and it answers the builder's question

1. 10 % of sub-queue ack replies are dropped (applied server-side, reply lost).
2. Some inbox entry's fan-out does not fully complete, so the worker's `ok = min over subs` falls and
   that entry is **not acked** — the safety property working exactly as designed.
3. That entry is invisible for **5 s**, hardcoded.
4. During those 5 s **no ack lands anywhere**, so the old attempt-budgeted poller expired at ~2 s and
   printed `drained-never`.
5. The stall-bounded poller waits it out; the entry redelivers; the run drains and passes at ~5.2 s.

★★★ **The system was always correct. It is slow by exactly one hardcoded 5-second constant, and the
harness's budget was shorter than that constant.** That is the direct answer to *"are we correct but
slow?"* — **yes, and now the number has a name.**

## Two of my specifications were wrong and the executor fixed both

- ⛔ **My `drained-timeout` wording asserted something the code cannot know.** I specified *"still
  progressing"*; the executor witnessed it firing on a **totally stalled** system and rewrote it to
  print only `stale`/`stale-max`. I wrote a message that lies in a reachable state — the same class
  as a catch-all arm that reports nothing.
- ⛔ **My BRIEF's premise "passes alone" was already false.** The executor measured 1 FAIL in 3
  pre-change runs alone. My probe-B claim came from one sample per `sub-cap`; it was a lucky draw
  reported as a property.
- ★ And my constant would have red-flagged correct work: I reasoned ~70 polls, the executor picked
  200, then measured **`drain-stale-max=197` on a PASSING run** — three polls from a false red.
  Measured max 289; shipped 600.

## Grade

`1 ✅ · 2 ✅ · 3 ✅ (executed, verified by me) · 4 ✅ (both arms executed, ceiling temporarily shrunk
to witness, file restored byte-identical) · 5 ✅ · 6 ✅ · 7 ✅ · 8 ✅ · 9 ✅ (7/7 pass, was 5 pass 2
fail) · 10 ✅ · 11 ✅ my premise held · 12 ✅ honoured · 13 ✅ re-baselined`

No STOP fired. **The two chaos reds are resolved without widening a rate, a cap, or a timeout** —
which was the standing prohibition, and it held.

## Named for the builder's ruling, not acted on

⚠ Two tiny chaos cells now cross nextest's **15 s SLOW warning** (16.3 / 16.6 s) against a 30 s kill,
because a slow-mode run waits out the 5 s. Margin 13 s, nothing timed out, and `.config/nextest.toml`
was correctly left untouched.

★ **And the 5 s constant itself is now the obvious next question:** it is hardcoded at one site, it is
50× the fast path, and nothing documents why it is 5 s. Whether it should be a parameter — or smaller
— is a ruling, not a cleanup.
