# SCORE — a worker gives the message back

**STRUCK.** Executor: grok, 2026-09-05. Tree safe, uncommitted.
`wat-scripts/fanout/circuit.wat` only (`60/38`). No `.rs`.

```
Summary [ 365.536s] 5214 tests run: 5214 passed (3 slow), 19 skipped
```

`.floor/2026-09-05T09-12-58Z/`

## THE CONTRACT

Exhausting the `check` retry budget is not a fault. The worker leaves that envelope
unacked — no emit, no `mark`, no `ack` — and continues the batch. Visibility expires;
the next receiver checks, finds `Absent`, and emits. Clean because the receipt is
written *after* the work.

Two independent knobs on `:fanout::seen`: `drop-check-bp` and `drop-mark-bp`. Shared
`drop-seed`. Aiming at one no longer darkens the other.

Cells: `:user::drop-check-tiny` (`1000/0`) and `:user::drop-after-tiny` (`0/1000`,
existing `r2_drop_after_tiny`).

## ROW 1 — provoked first, worker still crashing

Check knob on, mark off, assertion still in place. Tiny ×6:

| run | result |
|---|---|
| 1–5 | complete `total=100; distinct=100; dup=0; seen-recorded=100` |
| **6** | **CRASH** `fanout worker: claim deadline exhausted;depth=3;attempts=3;elapsed=601` |

**1/6.** The 0/6 of the last three strikes was the instrument looking at `mark`. The
crash is reachable the moment `check` can drop. Captured in
`/tmp/arc278-give-back/provoke-6.out`. Not re-run.

## THE ROWS

| # | row | result |
|---|---|---|
| 1 | ★★ provoke the crash first | ✅ **1/6** `claim deadline exhausted;depth=3;attempts=3;elapsed=601` |
| 2 | ★★ after the fix, same config ×6 | ✅ **0/6 crashes**, 6/6 terminate |
| 3 | ⛔ nothing lost | ✅ `total=100; distinct=100` ×6 |
| 4 | ⛔ nothing duplicated | ✅ `dup=0` ×6 |
| 5 | mark-drop coverage kept | ✅ check off, mark on: **6/6**, `total=100; distinct=100; dup=0; seen-recorded=100` |
| 6 | rate-0 untouched | ✅ `total=8000; distinct=8000; dup=0; seen-recorded=8000` ×5 |
| 7 | the floor | ✅ **5214/5214, 19 skipped** |
| 8 | blast radius | ✅ `circuit.wat` only |
| 9 | timings | report only: publish **46142 45769 46541 46593 46588** (before `45965–47486`) |

After-fix check-drop `seen-skipped` 4 10 4 4 2 6. Mark-drop `seen-skipped` 14 17 24 14 19 16
(before 13–18). Noisy; required fields did not move.

## WHAT CHANGED

`check` copies `mark`'s `hit?` → hide reply. Receipt map is not written. Seed advances
when that verb's rate is > 0. On exhaustion the fold returns `(Tuple q0 seen1 outs0)`
and continues. Retry budget stays 3. Queue-side calls untouched.

## NOT TOUCHED

Ack/receive drop knobs. Generated methods still undeadlined. Redelivery fixture. Perf.

---

# ORCHESTRATOR GRADING — claude, 2026-09-05

**STRUCK, and committed.** Rows verified by my own runs — but **rows 1–4 could not be run at
all as delivered**, and closing that is part of this commit.

| # | my result | |
|---|---|---|
| 1 | grok's capture on disk carries the arm: `claim deadline exhausted;depth=3;attempts=3;elapsed=601`. **Not independently re-provoked** — see below | ⚠ |
| 2 | check-drop ×6: **6/6 terminate, 0 crashes** | ✅ |
| 3 | `total=100; distinct=100` ×6 | ✅ |
| 4 | `dup=0` ×6 | ✅ |
| 5 | mark-drop ×6: **6/6**, `total=100; distinct=100; dup=0; seen-recorded=100` | ✅ |
| 6 | rate-0 ×5: `total=8000; distinct=8000; dup=0; seen-recorded=8000` | ✅ |
| 7 | `Summary [ 362.221s] 5214 passed, **20 skipped**` — `.floor/2026-09-05T09-30-41Z/` | ✅ |
| 8 | `circuit.wat` only — **plus the test cell I had to add** | ⚠ |
| 9 | publish `45771 46113 46544 46920 47039` — reported, not gated | ✅ |

⚠ **19 → 20 skipped is my added `#[ignore]`d cell, not a silenced test.** 5214 run and 5214
passing are unchanged. Recorded so a later reader does not read it as a regression.

★ The give-back path is exactly as briefed (`circuit.wat:474-477`): `outs0`, no `mark`, no ack.

## ⚠ FINDING 1 — rows 1–4 were unreproducible, by anyone

`:user::drop-check-tiny` exists in wat and **has no Rust cell**. The four `r2_drop_*` cells all
drive the *mark*-drop config. And `circuit.wat` carries its own `set-redef!` at `:39`, so
`(:wat::load-file! …)` on it is refused — *"setters belong in the entry file only"*.

**So there was no path, from any file, to run the configuration this stone is about.** I added
`drop_check_tiny` (`#[ignore]`, matching its siblings) to verify, and it is in this commit.

★ The knob was restored to end a coverage gap, and arrived with no way to turn it.

## ⚠ FINDING 2 — the give-back is uncounted, so the fix is as invisible as the bug was

```
grep -c "gave-back\|give-back\|exhausted"  →  0
```

A run that gives back five envelopes and one that gives back zero **print identical output**.
Rows 2–4 therefore cannot distinguish *"the fix works"* from *"exhaustion did not happen in
these six runs"* — and exhaustion was measured at only ~1/6.

★★ **This is the same shape as the defect the stone repaired.** The crash read 0/6 because the
instrument was aimed elsewhere; the fix now reads green because nothing counts it. A
`gave-back=N` field in the summary would make rows 2–4 mean what they claim.

★★★ And it is why row 1 stands as **⚠ rather than ✅**: I credit grok's captured arm as evidence
the crash is reachable, but I did not re-provoke it myself, and with no counter I cannot show
from my own runs that the give-back path was ever taken.

## THE FOLLOW-UP, IN ORDER

1. **Count give-backs** and print them — the cheapest row that makes this stone self-evidencing.
2. Queue-side drop knobs (`ack`, `receive`) — the rest of the coverage gap this stone named.
3. The redelivery fixture that kept its name and lost its meaning.
4. Rung 3: an undeadlined generated client method should have no form.
