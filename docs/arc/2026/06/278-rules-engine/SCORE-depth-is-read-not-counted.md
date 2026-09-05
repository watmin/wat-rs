# SCORE — depth is read, not counted

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted. `wat-scripts/queue/sqs.wat` only.

```
Summary [ 360.915s] 5214 tests run: 5214 passed (3 slow), 19 skipped
FLOOR=0        my own run · circuit total=8000; distinct=8000; dup=0 ×5
```

The 19 skipped are the same 15 + four R2 placement cells. No existing test was silenced.

## ★ THE PROBE FILE DID NOT MOVE

`probe-depth-derived-from-the-index.wat` is the commit before the brief (`c9ce0f9ce`). After the strike, same file, no edit:

```
took=1;LEASE-LIVE derived=[2/1] counters=[2/1] agree=yes;LEASE-EXPIRED derived=[3/0] counters=[3/0] agree=yes
```

Before: `LEASE-EXPIRED … counters=[2/1] agree=NO`. Derived stayed `[3/0]`. The `agree=NO` became `yes` because the stats surface now reads the index.

Leak probe, same discipline:

```
sent=[1/0];held=[0/1];EXPIRED-NO-RECEIVER=[1/0];came-back=same-id;after-receive=[0/1];AFTER-ACK=[0/0]
```

Before: `EXPIRED-NO-RECEIVER=[0/1]` — the counter still holding a lease the clock had already ended.

## THE SHAPE

`depth` is a typed local `fn` inside `:init` at `sqs.wat:224`, immediately after `take` (ends `:219`). Carried as `:ephemeral`, called through `State/depth`. Two range scans, `limit = cap + 1`, `(visible, unacked) = (|isk in [0, now]|, total − visible)`. `+inf` is the probe's `4000000000000000000`. Lost/Closed/Stopped on a scan assertion-fail — a 0 would open the cap.

A top-level `defn` was not reached for. Circuit queues are process locus; the circuit completed, which is the position proof the thread-locus probes cannot give.

`StatsRequest` is still empty. The arm uses `Invocation/start-ns` and an ephemeral `q-name` remembered from send/receive/ack (one name per process-locus instance). Durable did not move. Circuit did not move.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ defect probe flips | ✅ `LEASE-EXPIRED … agree=yes`; derived `[3/0]`; counters `[2/1]`→`[3/0]` |
| 2 | ★★ leak probe closes | ✅ `EXPIRED-NO-RECEIVER=[1/0]`; `after-receive=[0/1]`; `AFTER-ACK=[0/0]` |
| 3 | rate-0 circuit ×5 | ✅ `total=8000;distinct=8000;dup=0` ×5. ⚠ `seen-dups` is **not** 0 — see findings |
| 4 | the floor | ✅ **5214/5214, 19 skipped, my run** (`360.915s`) |
| 5 | ★ publish band | ⚠ **FINDING** — 45821 45945 44935 45372 45285 ms vs baseline 26599–27832 |
| 6 | ★ stranding survives | ✅ completed runs `total ∈ {90,91,91,91,92}` of 100. ⚠ 1/6 died on a different arm — see findings |
| 7 | `drained-never` is gone | ✅ none of the six printed `drained-never` |
| 8 | the two fields are gone | ✅ `grep -c "State/visible\|State/unacked"` → **0** (before 31) |

## ⚠ FINDING — publish is 67 % slower, and that is not a reason to restore the counters

Row 5, recorded:

| | ms |
|---|---|
| baseline (5-run, 4.6 % spread) | 26599 26742 27527 26950 27832 |
| after | 45821 45945 44935 45372 45285 |

Cap gate moved from two field reads to two `scan-index` round-trips on every send, 8000 of them, `limit = cap + 1`. ~18 s extra ≈ 2 µs-scale store RTTs × 16 000 scans. The DESIGN named this cost before the strike: *a fast wrong number is not a substitute for a slow right one.* Counters stay deleted.

## ⚠ FINDING — rate-0 `seen-dups` is no longer 0

Same five circuit runs: `seen-dups = 10 8 5 3 2`. Outcomes are still unique (`dup=0; distinct=8000`). Seen is the server's claim tracker; T1's timeout → discard → redial → retry on a fresh peer is live at rate 0. Slower publish makes that path fire. Cousin of the row-5 finding, not a second mechanism, and not a reason to restore the counters.

## ⚠ FINDING — tiny drop, 1/6, a different red than `drained-never`

Five of six `r2_drop_after_tiny` completed in ~8 s with `total ∈ {90..92}` — stranding survived, attribution holds. Run 5 died at `circuit.wat:1022`:

```
fanout worker: claim deadline exhausted;depth=3;attempts=3;elapsed=600
```

Not `drained-never`. Honest depth at stop is 3 leftover rows; drop-after answers those claims with `None`; three 200 ms deadlines exhaust. Previously the inflated `unacked` hid the leftovers from this path and stranded the drain instead. Out of scope to repair (circuit.wat, stranding). Reported, not patched.

## THE FIRST FLOOR WAS RED — and that was the fixture clock, captured

`.floor/2026-09-05T04-32-00Z/`. Do not re-run; the arm is on disk.

```
Summary [ 357.163s] 5214 tests run: 5213 passed (3 slow), 1 failed, 19 skipped
FAIL  wat::services probe_queue_depth::queue_depth_counters_are_accurate
got:    send=p=3,f=0;recv=p=3,f=0;ack=p=2,f=0
wanted: send=p=3,f=0;recv=p=1,f=2;ack=p=1,f=1
```

`:user::depth` in `sqs.wat` sent at fixture `T0=1e9` with `vis-ns=1e9` (hide-at `2e9`). Stats uses `Invocation/start-ns` — wall clock, ~1.7e18. Every lease looked expired. The probes and the circuit never hit this: they drive send/receive from `now` too.

Fix stayed in `sqs.wat`: `:user::depth` now takes `epoch-nanos(now)` and `vis-ns=1e12`, same clock stats uses. The Rust assertion did not move. Second floor (new tree, not a flake re-run) is the green Summary above.

This is not STOP-3. Stats has a moment. The in-file fixture was on a different one.

## WHAT DID NOT MOVE

- `circuit.wat` — drain condition untouched. `visible == 0 AND unacked == 0` now sees true numbers.
- `wat/` — no Store count verb, no `mem.wat` change.
- Durable record — still `cap` + `store-addr`.
- StatsResponse surface fields — still `visible` / `unacked`; only the State counters went.
- Consumer stranding (`circuit.wat:491`) — still there, `total < 100` on the runs that completed.
- nextest override — not touched.

## DELTA FROM THE SKETCH, NOT A STOP

The BRIEF sketched `depth` as the one new ephemeral. Stats has no queue name and `scan-index` needs `ipk`, so State also carries `q-name <- String` (init `""`, set from send/receive/ack). Empty name scans `ipk=""` and reports `[0/0]`. Cap gate uses the SendRequest's `q`, not the stored name. Circuit unchanged — not STOP-4.

## SCOPE

`wat-scripts/queue/sqs.wat` only (`104/77`). SCORE uncommitted. Counters not restored.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK, and committed.** Every row re-run by me on the executor's tree. Nothing below is
credited to the report; each line is a run of mine.

| # | my result | agrees |
|---|---|---|
| 1 | `LEASE-EXPIRED derived=[3/0] counters=[3/0] agree=yes` | ✅ |
| 2 | `EXPIRED-NO-RECEIVER=[1/0];after-receive=[0/1];AFTER-ACK=[0/0]` | ✅ |
| 3 | `total=8000;distinct=8000;dup=0` ×5 | ✅ |
| 4 | `Summary [ 360.461s] 5214 tests run: 5214 passed, 19 skipped` — `.floor/2026-09-05T05-08-25Z/` | ✅ |
| 5 | publish `46033 46520 45550 45356 45401` vs baseline `26599–27832` → **1.68×** | ✅ |
| 6 | 5/6 complete, `total ∈ {89,90,90,91,89}`, all **< 100** | ✅ |
| 7 | **no `drained-never`** in any of six | ✅ |
| 8 | `grep -c` → **0** | ✅ |
| — | 1/6 died `claim deadline exhausted;depth=3;attempts=3;elapsed=601` | ✅ (grok saw 600) |

## THE RED WAS HONESTLY DISPOSED — checked two ways

`.floor/2026-09-05T04-32-00Z/` is on disk with its arm, reported rather than re-run away.

**The fixture was corrected, not weakened.** `probe_queue_depth.rs:26` still asserts
`"send=p=3,f=0;recv=p=1,f=2;ack=p=1,f=1"` — verified by reading it, and independently by
`git status` showing **one** modified file. Only the fixture's clock moved (`T0 = 1e9` →
`epoch-nanos(now)`).

★ **The old fixture passed *because* the counters never consulted a clock.** `T0=1e9` was a
fantasy timestamp that worked only while nothing compared it to now. Making depth time-relative
made the fake clock visible. A test that was green for a compensating reason.

## ⚠ MY FINDING — the hot path buys a split it discards

Three of the four `depth` call sites immediately sum the pair:

```
sqs.wat:294   depth (:wat::i64::+ (first vu) (second vu))     ← the cap gate, SEND path
sqs.wat:472   (:wat::i64::+ (first vu2) (second vu2))         ← Full response
sqs.wat:502   (:wat::i64::+ (first vu2) (second vu2))         ← Full response
sqs.wat:741   (first vu) / (second vu)                        ← stats — the ONLY split consumer
```

`visible + unacked` **is** `total`, which is the `[0, +inf)` scan alone. **The send path runs two
scans to compute a number one scan gives** — 8000 sends × one wasted round-trip, on the exact
path row 5 measures.

This does not un-strike the stone: the contract is met, and the DESIGN named the cost in advance
as a finding rather than a blocker. It is the follow-up.

★ **Falsifiable prediction, stated before the work:** halving the send-path scans should recover
roughly half the 18 s, *and* pull rate-0 `seen-dups` back toward 0 — because finding 2 is
downstream of finding 1 (slower publish makes T1's deadline fire). If `seen-dups` does **not**
move when publish recovers, finding 2 has a second mechanism and my reading of it is wrong.

## ON FINDING 3 — the failure did not get worse, it got legible

`claim deadline exhausted;depth=3;attempts=3;elapsed=601` replaced a 63 s `drained-never`. The
inflated counter used to hide the leftover rows from this path and strand the drain instead.
**A named 600 ms arm in place of an unfalsifiable minute-long hang is this campaign's whole
thesis**, not a regression.

## ON FINDING 2 — the clean baseline is gone, and that is worth naming

Rate 0 was the arc's deterministic control. `seen-dups = 7 7 10 7 7` (mine) means the T1
deadline path now fires in the floor's own configuration. The invariant holds
(`distinct=8000; dup=0`), so nothing is broken — but **"rate 0" no longer means "nothing
happens"**, and any future row that reads `seen-dups` as a chaos signal must account for that.
