# BRIEF — self-scheduling item-(c): RECON (surface the mid-tick death)

> **Tier:** sonnet shadowdancer. **Kind:** RECONNAISSANCE — surface + report a root, **do NOT fix.**
> **Arc:** 278, item (c), self-scheduling defservices. **HEAD:** `25e316f0` (the no-hidden-failures
> LAW is committed at `1212c9ae` — do not touch it). **Ground:**
> `DESIGN-self-scheduling-defservices.md` (read the ⛔ SCOUT UPDATE at the top FIRST — the old "after
> builds the wrong thing" diagnosis is STALE; `after` is migrated both tiers).

## The work, in one paragraph

Two `#[ignore]`'d tests — `self_tick_fires_rearms_and_reactor_serves_thread` and `…_process`
(`tests/services/probe_arc278_self_scheduling.rs`) — are RED because the self-scheduling service
**dies mid-tick**. The generated serve loop `poll'`s over a mixed set of `{client connection + armed
`after`-timer}`; something in that path kills the service, and the only visible symptom is
*downstream*: the client's `poll` send hits `send': channel disconnected`
(`probe_arc278_self_scheduling.wat:87`) because the service is already dead. The service's OWN death
reason is invisible (it dies in its spawned thread). **Your job is to make that death VISIBLE and
report exactly where and why it happens — then STOP.** You write no fix.

## Read in order (the rooms)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-self-scheduling-defservices.md` — the ⛔ SCOUT UPDATE
   (top) + "The serve-loop change" + "How internal ops join the homogeneous `select'`" sections. This
   is the map; the two named suspects are (a) `poll'`'s reactor-class/homogeneity of a `{client +
   timer}` mix, (b) an idx-shift when a fired timer is removed + a new one armed.
2. `tests/services/probe_arc278_self_scheduling.wat` — the RED fixture. The Ticker service: `start`
   returns `ReplyAndArm` (arms the first `-tick`), `poll` returns `Reply` (the current count), `-tick`
   returns `NoReplyAndArm` (re-arm) until `target`, else `NoReply`. `drive-ticker` (:81) connects,
   `start`s, `nap`s 100ms, `poll`s → expects count == target (3).
3. `tests/services/probe_arc278_self_scheduling.rs` — the two `#[ignore]`'d entrypoints; thread-tier
   is `self_tick_fires_rearms_and_reactor_serves_thread` (:26), process-tier `…_process` (:45).
4. `wat/service.wat:948-974` — the generated serve loop's internal `-tick` arm (remove-at the fired
   timer's idx, re-arm via `arm-fn`); `:978-1015` the surface-op arm. **Read only — do not edit.**
5. `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` — the GREEN hand-rolled exemplar. Note it
   uses `select'` (one arg = the peers vec), NOT `poll'`. The delta between this green code and the
   crashing serve loop is the `select'` → `poll'` multiplexer (which also carries a self-peer +
   listener + a reactor-class homogeneity dispatch).

## The strike — instrument the real serve loop, run the THREAD test, read how far it gets

Thread-tier shares the test's stdout, so `(:wat::kernel::println …)` inside the running service is
visible under `--no-capture`. Focus the whole recon on the **thread** test first (the process test's
service is a separate process — stdout won't surface the same way; leave it for last).

**Sketch — add temporary `println` markers to the FIXTURE `tests/services/probe_arc278_self_scheduling.wat`:**

- top of the `start` handler body (:41): `(:wat::kernel::println "SCOUT start fired")`
- top of the `-tick` handler body (:52): `(:wat::kernel::println (:probe::ticker'::Record/count (:probe::ticker'::State/durable s)))` — prints the count each tick so you see *how many* ticks fire.
- inside `drive-ticker` (:81), right before the `poll` (:87): `(:wat::kernel::println "SCOUT client about to poll")`.

Keep the markers minimal and syntactically valid (`--check` the file after editing:
`./target/release/wat --check tests/services/probe_arc278_self_scheduling.wat`).

**Run the single thread test, ignored, no-capture** (from `wat-rs/`):

```
cargo nextest run --release self_tick_fires_rearms_and_reactor_serves_thread --run-ignored all --no-capture 2>&1 | tee /tmp/claude-scout/self_sched_thread.log
```

(If the package needs naming, discover it from `cargo nextest list | grep self_tick`.) Read the FULL
output — the wat printlns, AND any Rust panic / assertion / `RuntimeError` the service emits as it
dies (it may surface as a panic in the spawned thread, or a broadcast + a downstream error).

## What each outcome means (the disambiguation — this IS the deliverable)

| you observe | the root is |
|---|---|
| `start fired` prints, `-tick` count **never** prints, service dies | **suspect (a):** `poll'` rejected/mishandled the armed `after`-timer when it was `conj`'d into the mixed `{client+timer}` set — a reactor-class/homogeneity failure in `poll'` (`eval_poll_prime`, `runtime.rs:27499+`). Capture the exact error text. |
| `-tick` count prints once/twice then dies, or counts are wrong (skips/repeats) | **suspect (b):** idx-shift on remove-at + re-arm in the serve loop (`service.wat:948-974`) — the fired timer's idx and the re-armed timer's insertion desync. Capture the last count printed. |
| `-tick` reaches target (3) fine but `client about to poll` → `send': channel disconnected` | a THIRD root: the service stops serving the client after ticking (a serve-loop exit / selectables mismanagement) — capture whether the service exited its loop. |
| something else | report it verbatim — a run beats every guess. |

## STOP triggers (rejection criteria — surface the gap, ship no fix)

- **STOP-0:** if you find yourself writing a *fix* to `wat/service.wat`, `src/`, or the fixture's
  logic — STOP. This brief is recon only. Your output is a located root + the captured error text.
- **STOP-1:** if the `println` markers do not compile or the test cannot be invoked — STOP, report
  the exact `--check` / nextest error; do not restructure the fixture to make it run.
- **STOP-2:** if the thread-tier death does not surface even under `--no-capture` (the service dies
  truly silently) — STOP and report that fact plus everything you *did* see; do NOT escalate to the
  process tier or invent an inline `poll'` probe on your own.

## Deliverable (report back, then leave the tree clean)

1. Which suspect (a/b/third) the run confirmed, with the **exact captured error text / panic** and the
   **`file:line`** it points at.
2. The last `-tick` count observed (how far the ticking got).
3. **Revert every `println` marker** so the fixture is byte-identical to HEAD (`git diff` must be
   empty on `tests/services/probe_arc278_self_scheduling.wat` when you finish).
4. Do not commit. Do not un-`#[ignore]` the tests. The orchestrator weighs your report and briefs the
   fix.

## Blast radius

`tests/services/probe_arc278_self_scheduling.wat` only, temporary printlns, **reverted before you
report.** No `src/`, no `wat/service.wat`, no other `.wat`. No new files. Scratch logs → `/tmp/claude-scout/`.
