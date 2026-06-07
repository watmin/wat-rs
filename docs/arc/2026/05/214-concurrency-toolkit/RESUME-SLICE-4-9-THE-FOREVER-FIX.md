# 214 RESUME — Slices 4–9: the forever-fix (2026-06-07)

**The commitment (builder):** *"let's get 214 done done — prove we're done fucking
around with these things forever."* 214's engine is built; the wiring was never
finished, so the live path is still the hand-wired `typed_channel`/`thread_io`/`fork`
stack — which deadlocks (the ProcessPeer ambient-stdio round-trip) and which we keep
re-touching. Finishing 214 retires that stack for good.

## Where 214 actually stands (grounded 2026-06-07)

- **Slices 1–3 ✅ built + warded:** `comms::thread` + `comms::process` — full
  Sender/Receiver/Select/pair, io_uring, HolonRepresentable, cascade-aware,
  persistent rings. SCORE'd, WARD-PASS'd.
- **Slice 4 ⚠️ partial:** only `program-env` (Stones 4.1–4.3) shipped. The **peer
  types, the unified spawn, the polymorphic verbs are OPEN.**
- **comms is UNWIRED:** `comms::` appears nowhere in `runtime/check/freeze/fork/
  thread_io/spawn_process`. The wat surface (`spawn-thread`, `spawn-process`,
  `Process/readln`, the stdio services) all run the OLD stack.
- The make-channel collapse (254.0) was a **Slice-5/6 fragment done early** (one
  channel constructor; the migration spine starts here).

## The path (each "big" = ~4–7 stones; ~25–35 total — a campaign, not a strike)

- **Slice 4 — kernel layer** (`src/kernel/{mod,peer,spawn}.rs`, NEW):
  - **4.4 peer types** — `Thread<I,O>` (comms::thread Sender<I> + Receiver<O> +
    join + cascade), `Process<I,O>` (comms::process Sender<I> + Receiver<O> +
    Pidfd + cascade); wat-level `:wat::kernel::Thread<I,O>` / `Process<I,O>`.
  - **4.5 spawn dispatcher** — `spawn-program'` dispatches on `:tier` →
    `comms::thread::spawn_program` / `comms::process::spawn_program`; sandbox-walker
    validates `:process` captures (HolonRepresentable-only).
  - **4.6 polymorphic verbs** — `send'`/`recv'`/`try-recv'`/`select'`/`close'`
    multimethod-dispatch on peer type (arc 146 multimethod).
  - **4.7 smoke probes** — thread + process peer round-trips via the kernel verbs.
- **Slice 5 — migration sweep** — flip caller sites to the primed verbs;
  `typed_send`/`typed_recv` become thin shims over `comms::*`; unify spawn;
  retire legacy regs; rename primes → canonical.
- **Slice 6 — structural wall** — **retire `typed_channel.rs` + the dead
  `thread_io`/`fork`/`spawn` paths** via module privacy. *typed_channel dies here.*
- **Slice 7 — brackets** (`parallel-for-each`).
- **Slice 8 — services universe-resident** — actors over comms; **kills
  handle-passing** → the ambient-stdio-ProcessPeer deadlock class becomes
  unrepresentable.
- **Slice 9 — INSCRIPTION** — closes **214 + 253 + 254** (the unwind).

## Discipline for the campaign

- Per-stone cadence: sub-DESIGN → FM-2-bis probe (RED) → brief sonnet
  (`model:"sonnet"`) → SCORE vs own re-run → commit + push. Monotonic: every stone
  committed, never regresses.
- The comms tiers are DONE — Slice 4 *wraps* them, does not rebuild them.
- Process tests run via `integration-run.sh` (setsid+timeout per-binary), NEVER
  the raw `cargo test --test test` run-tier (it deadlocks on the old stack until
  Slice 8).
- The deadlock consumes no more debugging attention — it dies as a *consequence*
  of Slice 6 (retire) + Slice 8 (universe-resident services), like the leak died
  via RAII and the constructor zoo died via make-channel.

## Resume point: **Slice 4 Stone 4.4 — peer types.**
Everything downstream (5 migrate → 6 retire → 8 services) hangs off the peer
types existing. That's the first strike.
