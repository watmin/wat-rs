# SCORE — stop means gone

**SCORED.** Executor: grok, 2026-09-12, branch `sns-sqs`, HEAD `7724f6ad0` (DRAWN). Did not commit.

```
     Summary [ 526.964s] 5237 tests run: 5237 passed (7 slow), 22 skipped
```

`.floor/2026-09-12T23-22-14Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
**Nothing was re-run to make anything go away.** `5237` unchanged — no deftest added.
Clippy: `cargo clippy --release --workspace --all-targets -- -D warnings` exit **0**.
`cargo nextest run --release --no-run` exit **0**.
`git diff --stat -- src/` empty. `wat/` is `service.wat` only (+55/−4). `wat/core.wat` still 2152 lines.

---

## ⭑ THE HEADLINE — under load, 6/20 Sent became 20/20 Lost

`<S>/stop` now waits for the lineage socket to close after `Status::Stopped`. Same `t0`, same 10 000 ms, no sleep. `Stopped` is a statement about the world.

### Row 1 — BEFORE (8 spinners, 20 runs, this box, 12 CPU)

```
     14 try-send-after-stop=Lost;send-after-stop=Lost
      4 try-send-after-stop=Sent;send-after-stop=Lost
      2 try-send-after-stop=Sent;send-after-stop=Sent
```

6/20 had at least one `Sent`. The DESIGN's common form (`try-send=Sent ; send=Lost`) is 4 of those 6.

### Row 1 — AFTER (same harness, same 8 spinners)

```
     20 try-send-after-stop=Lost;send-after-stop=Lost
```

SENT-LINES=0. 20/20 gone.

A quiet-box after run is also Lost/Lost — that is **not** the proof; the load table is.

---

## WHAT LANDED

`wat/service.wat` only.

- `owner-wait-gone` — after the ack, `owner-recv-loop` until `Gone` (Closed or Lost) or `GaveUp`. A `Stopped` (still emitting) re-recvs on the **same t0 / 10000**. Not a sleep. Not an attempt counter. TimedOut stays GaveUp.
- `stop-method-body` — on `Status::Stopped`, extract `resp`, then `owner-wait-gone`. `Gone` → `Stopped resp`. `GaveUp` → `GaveUp`.
- `hibernate-method-body` — **same**. Measured, not assumed: Admin::Hibernate at the serve loop (`:2320`) sends `Status::Hibernated` then returns nil, no recur, identical to Stop (`:2314`). Grant/revoke recur; they were not touched.

The serve loop's ack-then-exit is unchanged (DESIGN: origin of the race, not this stone).

---

## ROW 3 — Stopped still carries the state

`probe-stop-means-gone-state.wat`: `stop=Stopped`.

---

## ROW 4 — GaveUp is reachable for the new cause

`probe-stop-means-gone-gaveup.wat`: a child prints `"acked"` then parks on `readln`. First `owner-recv-loop` → `Stopped`. `owner-wait-gone` budget 500 ms →

```
first=Stopped
second=GaveUp waited=500 last=TimedOut
```

Not a false `Stopped`. TimedOut is not folded into gone (STOP-2 held).

---

## ROW 5 — hibernate shares the shape

Yes. Serve-loop arms `:2314` (Stop) and `:2320` (Hibernate) both send the status then return nil. Both generated methods now close-wait. Grant/revoke continue serving; out of scope.

---

## ROW 6 — budget shared

One `t0` at send. Both recvs pass `10000`. `owner-recv-loop` computes `remaining = budget - elapsed`. No second budget.

---

## ROW 12 — `stop=` moved: 149 → 404 ms

Happy `2000 4 3 8192 true 1000`: `distinct=8000;dup=0`. Phase `stop=404` (was **149 ms**). 9 services × one extra recv-for-close. **Finding, not a failure.** `stop-busy-ms=2` (CPU); the 255 ms is waiting on the fd.

Chaos `50 2 2 … 500`: `distinct=100;dup=0`, exit 0.

---

## ROW 13 — census cells unchanged

`probe-crash-surface-send-closed-thread.wat`: `thread-recv-after-exit=Closed;thread-send-after-exit=Lost`. This stone does not claim `SendOutcome::Closed`.

---

## NOT TAKEN (correct)

No `close'`. No Handle field. No `src/`. `TrySendOutcome::Sent` still means the local cell accepted it (DESIGN §What this does NOT fix).

---

# ⭑ ORCHESTRATOR'S GRADING — claude, 2026-09-12

**Graded against my OWN reads and runs**, from `7724f6ad0` + the working tree.

```
floor   .floor/2026-09-12T23-34-43Z/  Summary [ 519.375s] 5237 tests run: 5237 passed, 22 skipped
        0 failure tokens · no ARM.txt
clippy  --release --workspace --all-targets -D warnings → exit 0
happy   distinct=8000;dup=0  wall=24748ms  stop=365  stop-busy-ms=1  rt-store=6483
chaos   distinct=100;dup=0  exit 0
blast   wat/service.wat ONLY, +55/−4 · git diff --stat -- src/ EMPTY
```

**STRUCK. 13 of 13 rows pass on my instruments** — and row 1 is the one a floor could never give.

## ⭑⭑ Row 1 — the race is annihilated, and I own both halves of the measurement

My **own** before (unmodified tree, `ec3ea95c8`, 8 spinners × 20 runs) and my **own** after (this tree,
identical harness):

```
BEFORE   17 × Lost                                     AFTER   20 × Lost
          3 × try-send-after-stop=Sent                          0 × Sent
```

grok's before found **6/20** carrying a `Sent` (4 of them the `try-send=Sent ; send=Lost` form the DESIGN
predicted, 2 with both). Mine found 3/20. Different rates, same verdict — and the after is **20/20 on both
our runs, under load.**

★ **A quiet box still cannot see this**, before or after. That is why the load table is the proof and the
green floor is not, and it is why I ran the load harness myself rather than accepting the table.

## Row 2 — an fd event, and I checked the shape, not the claim

`owner-wait-gone` recurses on `Stopped` with the **same `t0` and the same budget**, so the bound is wall
clock via `owner-recv-loop`'s `remaining = budget − elapsed`. **No sleep. No attempt counter. No second
budget.** Termination holds because the budget shrinks on elapsed time.

★ And that makes it consistent with this campaign's own gate: *a give-up path must be bounded by progress
or wall clock, never by an attempt count* (`a-give-up-has-no-form-for-attempts`). The new loop obeys a rule
laid down four stones ago, without being told to.

## Row 4 — the important negative, on my own run

```
first=Stopped
second=GaveUp waited=500 last=TimedOut
```

`TimedOut` is **not** folded into gone. A service that acks and then refuses to close reports `GaveUp`,
not a false `Stopped` — STOP-2 held, and that was the one way this stone could have turned a race into a
**lie**, which is strictly worse than the race.

## Row 5 — the sibling was measured, not assumed

`hibernate` shares the shape and got the same close-wait: serve-loop arms `:2314` (Stop) and `:2320`
(Hibernate) both send their status **then return nil, no recur**. `grant`/`revoke` **recur** — they keep
serving — so they are correctly untouched. ⭑ Sixth time this campaign has asked the sibling question, and
the second time it was asked *before* a floor red rather than after.

## ⚠ Row 12 — the cost is real, named, and not on the critical path

`stop=` **149 → 365 ms** on my run (grok: 404). Nine services × one extra recv-for-close. But
`stop-busy-ms=1`, so it is **fd-wait, not CPU** — and total wall clock is **24748 ms against my own
pre-change 24860 ms**, i.e. unchanged within the run-to-run band. **We bought determinism for ~216 ms of
waiting that the wall clock does not notice.** Reported as a finding either way, per the row.

## Two things I found in the diff that are not rows

★ **The stone creates a structurally unreachable arm.** `owner-wait-gone` returns only `Gone` or `GaveUp`
— its `Stopped` case recurses — so the generated `stop`/`hibernate` match has a `StopOutcome::Stopped` arm
that **cannot fire**, filled with `assertion-failed!`. Exhaustiveness *requires* the arm, and a defensive
raise is the honest filler, so this is not the arc-109 painted brick (the variant is constructable in
general, just not returnable *here*). ⭑ The structural cure would be a return type that can only say
`Gone | GaveUp`, which `StopOutcome` cannot express. Named for the record; not a defect to fix in this
stone.

⚠ **`stop-faced` absorbs the new `GaveUp`.** It maps all three variants to `nil`
(`Stopped`/`Gone`/`GaveUp` → `nil`). That is pre-existing, but this stone makes `GaveUp` reachable *for a
new reason* — "acked but never closed" — and every `stop-faced` caller will therefore ignore it silently.
The information now exists and the default consumer throws it away. Worth a look when the placeholder
sweep happens.

## The rows

| # | row | my verdict |
|---|---|---|
| 1 | ⭑⭑ race gone under load | ✅ **my own** before 3/20 `Sent` → after **20/20 Lost** |
| 2 | fd event, not a sleep | ✅ read the diff — recursion on shared `t0`, no counter |
| 3 | `Stopped` still carries state | ✅ my run: `stop=Stopped` |
| 4 | `GaveUp` for the new cause | ✅ my run: `GaveUp waited=500 last=TimedOut` |
| 5 | sibling measured | ✅ `hibernate` shares it and got it; `grant`/`revoke` recur, untouched |
| 6 | budget shared not doubled | ✅ one `t0`, 10 000 ms total |
| 7 | no new surface | ✅ `service.wat` only; no `close'`, no Handle field, `src/` empty |
| 8 | floor | ✅ my own run, 5237/5237, 0 FAIL |
| 9 | tests compile | ✅ the floor ran them |
| 10 | clippy | ✅ my own run, exit 0 |
| 11 | happy / chaos | ✅ `8000/0` · `100/0` |
| 12 | ⚠ `stop=` cost | ✅ reported: 149 → 365 ms, `stop-busy-ms=1`, total wall unchanged |
| 13 | census cells unchanged | ✅ `thread-send-after-exit=Lost` — no `SendOutcome::Closed` claimed |

## What I'd credit above all

**The strike did not reach for the easy fix.** A sleep would have made row 1 green on the first try and
re-raced under heavier load; `close'` or a Handle field would have worked and blown the scope. It used the
event that was already there, in a loop that already existed, bounded by a budget that already existed —
and it asked the sibling question before the floor did.

## What this hands the next stone

The **store-fault injector** is still ranked #1 and unblocked by nothing here. ⛔ And the four census cells
that need a sanctioned reap — `SendOutcome::Closed`, `TrySendOutcome::Closed`, `CloseOutcome::Signaled`,
`CloseOutcome::Failed` — are **still UNREACHABLE**, deliberately: this stone proved the race closes without
that surface, so the reap stone is now worth exactly its four cells and no longer carries the race.
