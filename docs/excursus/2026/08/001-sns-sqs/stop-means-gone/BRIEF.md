# BRIEF — stop means gone

**Read `DESIGN.md` beside this first.** It carries the four-layer mechanism (each layer cited on disk),
the contract decision, the correction to my own reported rate, and the one thing this stone deliberately
does **not** fix.

## The work, in one paragraph

`<S>/stop` returns when the service **acks** `Status::Stopped`, but the serve loop sends that ack **and
then** terminates — so `stop` returns while the process is still winding down, and any operation on a
separately-dialed peer races the OS teardown. Make `stop` wait for the lineage socket to **close** — an fd
event, not a sleep — by calling `owner-recv-loop` a second time and requiring `Closed`/`Lost`. `Stopped`
then means the lineage is gone.

## The rooms, in order

| where | why you are going there |
|---|---|
| `wat/service.wat:2287` | the serve loop's `Admin::Stop` arm. Read the comment: *"sends Status::Stopped(state) back up the lineage peer (self), then terminates (returns nil, no recur)"*. **The ack precedes the exit.** This is the race's origin and it does **not** change. |
| `wat/service.wat:2968–2988` | `stop-method-body`. ⭑ **The room.** After the `StopOutcome::Stopped recvd` arm extracts the state, add a second `owner-recv-loop` on the same lineage peer and require `Closed`/`Lost`. |
| `wat/service.wat` `owner-recv-loop` | ⭑ **THE PROVEN SHAPE — read before writing.** Already wall-clock-bounded, already faces `Closed`/`Lost`/`TimedOut`/`Stopped`/`Malformed`, already returns a `StopOutcome`. You are calling it, not writing a loop. |
| `wat/service.wat:3669` | `handle-fields = [handle <- Peer  addr <- Address]` — confirms there is no lineage object to reap. You are using the **peer's socket close** as the signal, which is why no Handle change is needed. |
| the `hibernate` method body | ⭑ **The sibling.** `the-owner-faces-an-outcome` found `stop` and `hibernate` shared a shape. **Measure whether it does here** and say so either way — "fixed one, left the twin" has bitten this campaign five times. |
| `wat-scripts/scratch-pad/probe-crash-surface-try-send-after-stop.wat` | the reproducer. It already exists; do not rewrite it. |
| `docs/excursus/2026/08/001-sns-sqs/the-owner-wait-has-a-deadline/` | the stone that built `recv-by-deadline` and `owner-recv-loop`. Shape to copy for the SCORE. |

## Implementation sketch

Inside `stop-method-body`, the existing `Stopped` arm becomes: extract the state as today, **then** wait
for close before returning it.

```
((:wat::service::StopOutcome::Stopped recvd)
  (:wat::core::match recvd
    ((~status-stopped-kw resp)
      ;; NEW — the lineage socket closing IS the exit event (an fd event, never a sleep).
      (:wat::core::match (:wat::service::owner-recv-loop (~handle-handle-acc h) ~stop-t0-sym 10000 "close")
        ((:wat::service::StopOutcome::Gone _c)      (:wat::service::StopOutcome::Stopped resp))   ;; Closed/Lost → GONE
        ((:wat::service::StopOutcome::GaveUp w l)   (:wat::service::StopOutcome::GaveUp w l))
        ((:wat::service::StopOutcome::Stopped _)    (… still emitting: keep waiting or GaveUp …))))
    (_ (:wat::kernel::assertion-failed! "defservice stop: expected Status::Stopped" …))))
```

⚠ **Read `owner-recv-loop`'s actual arms and return shape before trusting that sketch** — it is a
convenience, not a specification. The declaration on disk is the contract. (A sketch in one of my briefs
was wrong about `EnumVariant::Unit` two stones ago; the executor was right to match the disk instead.)

The same `t0` is reused deliberately so the **10 000 ms budget covers ack + close together** — do not add
a second budget.

## ⭑⭑ VERIFY UNDER LOAD — a quiet box cannot see this bug

I collected **30/30 `Lost` on a quiet box**: silent. The reproducer is load:

```
cd ~/work/holon/wat-rs
for i in 1 2 3 4 5 6 7 8; do (while :; do :; done) & done; LOADPIDS=$(jobs -p); sleep 1
for i in $(seq 1 20); do ./target/release/wat wat-scripts/scratch-pad/probe-crash-surface-try-send-after-stop.wat 2>/dev/null | head -1; done | sort | uniq -c | sort -rn
kill $LOADPIDS
```

**Before this stone** that prints a mix including `try-send-after-stop=Sent` (I measured 3 of 20).
**After**, it must print `Lost` (or `Closed`) on **20 of 20**. Run it **both** before and after, and put
both tables in the SCORE.

## Also verify

- `./scripts/floor.sh`, read the **Summary line** → **5237 passed, 0 FAIL**. ⛔ Never a piped exit code
  (a type-error run this session reported `$?` = 0 through a `| head`; its true exit was **3**).
- `cargo nextest run --release --no-run` — `cargo build --release` does NOT compile tests.
- `cargo clippy --release --workspace --all-targets -- -D warnings` → 0.
- Happy path `circuit.wat 2000 4 3 8192 true 1000` → `distinct=8000;dup=0`, and chaos
  `… 50 2 2 8192 true 1000 0 0 7 0 0 0 0 0 500` → exit 0.
- ⚠ **Watch `stop=` in the happy-path phase line.** It was **149 ms** on my run today for 9 services. An
  extra recv per stop may move it; report the number either way. A large move is a finding, not a failure.
- Everything heavy through `./scripts/capped.sh --limit 8g`.

## STOP triggers

1. **STOP-1 — no sleeps, no retry counts.** If the fix needs a sleep or a bounded attempt count to be
   deterministic, STOP and report: the event is wrong, and a guess that races under heavier load is the
   defect being annihilated.
2. **STOP-2 — if `owner-recv-loop` cannot distinguish "lineage closed" from "still alive"**, STOP and say
   which arms it collapses. A fix that treats `TimedOut` as gone would convert this race into a false
   `Stopped`, which is worse.
3. **STOP-3 — do not add a wat-level `close'` or a lineage field to the Handle.** Both would work and both
   are out of scope (DESIGN §Out of scope). If you conclude the recv-for-close route cannot work, STOP and
   report why rather than reaching for them.
4. **STOP-4 — if a floor test depends on `stop` returning before the lineage exits**, STOP and name it.
   Do not patch a golden; a prior stone reverted `src/freeze.rs` and another restored `wat/core.wat`'s
   exact line count rather than patch one.

## Shape to copy

`the-owner-faces-an-outcome/SCORE.md` — the stone that gave `stop` its outcome and its bounded wait.
And `the-owner-wait-has-a-deadline/SCORE.md` — the one that proved a silent peer returns instead of
hanging, with a probe rather than a floor.
