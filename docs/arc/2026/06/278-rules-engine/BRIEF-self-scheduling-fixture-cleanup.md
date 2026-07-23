# BRIEF — self-scheduling item-(c): FIXTURE CLEANUP (make the vantage honest)

> **Tier:** sonnet shadowdancer. **Kind:** fixture cleanup — improve the test instrument. NOT a
> runtime fix. **Arc:** 278, item (c). **HEAD:** `25e316f0`. The self-scheduling service still crashes
> mid-tick — **you are NOT fixing that.** You are making the DRIVER an honest instrument so that when
> it fails, it fails *legibly*.

## Why (one paragraph)

The fixture `tests/services/probe_arc278_self_scheduling.wat`'s driver `drive-ticker` is a blind
instrument: it (1) **discards `_s`**, the `start` `RecvOutcome`, so a start-time death vanishes; (2)
**waits with a fixed `nap 100`** — a sleep-guess (`mora`: sleep is a guess, guesses race) — then polls
**once**; (3) because it polls only *after* the sleep, it never exercises the property this gate exists
to prove: *the reactor keeps serving a client BETWEEN ticks*. Clean all three. The tests stay
`#[ignore]`'d and stay RED (the runtime crash is unfixed) — but after this, the failure they show is
legible, and what remains blind is precisely the substrate seam (a `send'` raise on a dead peer),
which is the next strike, not yours.

## Read in order (the rooms)

1. `tests/services/probe_arc278_self_scheduling.wat` — the fixture. `drive-ticker` (:81) is what you
   rewrite; `nap` (:64) is a wait helper you keep (repurposed as a small bounded backoff); the service
   (:33) and entrypoints (:93, :99) you do **not** touch.
2. `tests/services/probe_arc278_self_scheduling.rs` — the two `#[ignore]`'d tests assert the returned
   `i64` == 3. Your rewritten `drive-ticker` must still return `i64` and still return **3 on success**
   (so the .rs assertion is unchanged). Leave the `#[ignore]` attributes in place.
3. The existing `match r` in `drive-ticker` (the `RecvOutcome::{Message,Lost,Closed}` shape, :88-90) —
   copy it as the shape for facing every recv outcome.

## The strike — rewrite `drive-ticker` (3 changes, one function)

**(1) Face `_s`.** After `start`, `match` its `RecvOutcome`:
- `Message[StartResponse::Ok]` → proceed to the poll loop.
- `Message[StartResponse::RequestTooLarge …]` → return a distinct sentinel (e.g. `-3`).
- `Lost[cause]` / `Closed` → `assertion-failed!` carrying the cause (a start-time death now speaks).

**(2) + (3) Replace `nap 100`-then-poll-once with a wire-synced poll-until-count loop.** Write a TCO
recursive helper (wat is TCO-proper) that polls, faces the outcome, and terminates on the *observed
value* — not on elapsed time:

```clojure
(:wat::core::defn :probe::poll-until
  [c <- <the peer type>  target <- :wat::core::i64  attempts <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::i64::<= attempts 0)
    -2                                              ;; bound exhausted without reaching target
    (:wat::core::match (:probe::Ticker/poll c (:probe::Ticker::PollRequest))
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:probe::Ticker::PollResponse::Count n)
            (:wat::core::if (:wat::core::i64::>= n target)
              n                                     ;; observed the target — done, no timing guess
              (:wat::core::let [_ (:probe::nap 5)]  ;; bounded backoff, NOT a correctness-bearing sleep
                (:probe::poll-until c target (:wat::core::i64::- attempts 1)))))
          ((:probe::Ticker::PollResponse::RequestTooLarge _b _cp) -1)))
      ((:wat::kernel::RecvOutcome::Lost __cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message __cause) :wat::core::None :wat::core::None))
      ((:wat::kernel::RecvOutcome::Closed)
        (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))))
```

`drive-ticker` then: `connect'` → `start` → face `_s` → `(:probe::poll-until c 3 <bound>)`. Pick a
generous `attempts` bound (e.g. 40 — 40 × ~5ms backoff ≫ the 15ms of ticks; the loop exits early on
observation, so the bound is only a failsafe, never the timing). Ground the exact peer type of `c` from
the current `connect'` binding — do not guess it.

## Ground the `RecvOutcome::Closed` arm shape

Match how the existing fixture writes the bodiless `Closed` arm (a bare-keyword arm vs `((Closed) …)`)
and stay consistent with it — do not "fix" that arm's shape; it is settled. `--check` the file after
every edit: `./target/release/wat --check tests/services/probe_arc278_self_scheduling.wat`.

## Run it (report what the cleaned instrument now shows)

```
cd /home/watmin/work/holon/wat-rs
cargo nextest run --release self_tick_fires_rearms_and_reactor_serves_thread --run-ignored all --no-capture 2>&1 | tee /tmp/claude-scout/self_sched_clean.log
```

The test will still FAIL (the runtime crash is unfixed). **The deliverable is the shape of that
failure now:** does the cleaned driver surface the death as a faced value (a `Lost`/`Closed`
`assertion-failed!` with a cause — from `_s` or from a poll), or does a **`send'` raise
(`channel disconnected`)** still fire *before* any `match` can run? That distinction is the finding.

## STOP triggers (rejection criteria)

- **STOP-0:** you touch `wat/service.wat`, `src/`, the service handlers, or the `.rs` assertions — STOP.
  Scope is `drive-ticker` + one new helper `defn`, in the fixture `.wat`, only.
- **STOP-1:** do NOT attempt to catch/handle the `send': channel disconnected` raise — that is item 5
  (a substrate change, out of scope). If it fires, that IS the finding; report it verbatim.
- **STOP-2:** do NOT un-`#[ignore]` the tests and do NOT change what the .rs expects on success (still
  `i64` == 3).

## Deliverable

1. The rewritten `drive-ticker` + `poll-until` helper, `--check` clean.
2. The run output: what the cleaned instrument surfaces (faced value with cause, vs. the `send'` raise).
3. Do NOT commit — leave the edit in the working tree for the orchestrator to weigh. Report the exact
   failure text and where it originates (recv-side faced value, or send-side raise).

## Blast radius

`tests/services/probe_arc278_self_scheduling.wat` only — `drive-ticker` rewritten + one new `defn`. No
`src/`, no `wat/`, no `.rs`, no other `.wat`. Scratch logs → `/tmp/claude-scout/`.
