# BRIEF — Finding A RECON: surface WHY the self-scheduling service dies mid-tick

> **Tier:** sonnet shadowdancer. **Kind:** RECONNAISSANCE — surface the death reason + report a located
> root. **You write NO FIX.** **Arc:** 278 item-c (the `VNDE ORTVM` payoff). **HEAD:** `4543ef7a`.

## The work, in one paragraph

The `#[ignore]`'d `self_tick_fires_rearms_and_reactor_serves_thread` fails because the self-scheduling
service **dies mid-tick**. The client only ever sees the generic `"service peer lost (reason on the owner's
crash channel)"` — a signpost, not the reason. The **actual** reason IS captured: when the service thread's
body (its serve loop) crashes, `spawn_thread_peer` catches it and sends the reason over `crash_tx`
(`src/kernel/spawn.rs:678` — a Rust panic → assertion envelope; `:685` — a wat `RuntimeError` →
`re.to_string()`). But `crash_rx` lives in the parent Thread peer (`:706`), which the test's `drive-ticker`
(a client `connect'`) never reads — so the reason is captured and thrown away. **Your job: print that
reason at the capture site, run the thread test, and report the located root — then STOP.**

## Read in order (the rooms)

1. `docs/arc/2026/06/278-rules-engine/DESIGN-self-scheduling-defservices.md` — the ⛔ SCOUT UPDATE (top). The
   two named suspects: **(a)** `poll'` reactor-class/homogeneity of a `{client + armed-timer}` mix
   (`eval_poll_prime`, `src/runtime.rs:27500+`); **(b)** an idx-shift when a fired timer is removed + a new
   one armed (the generated serve loop, `wat/service.wat:948-1015`).
2. `src/kernel/spawn.rs:665-690` — the `catch_unwind` around the service thread body: `Err(payload)` →
   `reason` → `crash_tx.send(reason)` (:678); `Ok(Err(re))` → `crash_tx.send(re.to_string())` (:685). **This
   is where you instrument.**
3. `tests/services/probe_arc278_self_scheduling.{wat,rs}` — the fixture (already cleaned: `drive-ticker`
   faces `_s`, poll-until-count). The thread entrypoint is `:user::self-tick-rearms-thread`; the `.rs` test
   is `self_tick_fires_rearms_and_reactor_serves_thread` (`#[ignore]`'d).

## The strike — a temporary Rust print at the crash-capture site

Add a **temporary** `eprintln!` at BOTH capture arms in `src/kernel/spawn.rs`, so a thread-service death
prints its real reason to stderr (visible under nextest `--no-capture`; Rust `eprintln!` has no wat-stdio
dependency, unlike the earlier `println` recon that hit the `THREAD_IO` gap):

- at `:678` (panic arm): `eprintln!("SCOUT[Finding A] service thread PANIC death: {reason}");`
- at `:685` (RuntimeError arm): `eprintln!("SCOUT[Finding A] service thread RUNTIME death: {re}");`

Build + run the single thread test, ignored, no-capture:

```
cd /home/watmin/work/holon/wat-rs && mkdir -p /tmp/claude-scout
cargo nextest run --release self_tick_fires_rearms_and_reactor_serves_thread --run-ignored all --no-capture 2>&1 | tee /tmp/claude-scout/findingA.log
```

Read the FULL output. Find the `SCOUT[Finding A]` line(s) — the exact death reason (message + any span/
`file:line` it names). That reason IS the located root.

## What each outcome means (the disambiguation — the deliverable)

| the SCOUT reason names… | root |
|---|---|
| a `poll'` error — reactor-class / homogeneity / "expected Peer'" / a downcast/tier mismatch on the peers set | **suspect (a):** the armed `after`-timer isn't accepted in `poll'`'s `{client+timer}` mix (`eval_poll_prime`, `runtime.rs:27500+`). Capture the exact text + span. |
| an index / bounds / `remove-at` / wrong-op / `nth` error, or a wrong count | **suspect (b):** idx-shift on remove-at + re-arm in the serve loop (`wat/service.wat:948-1015`). Capture the last state it reached. |
| something else | report it verbatim — a run beats every guess (the DESIGN root was already stale once). |

## STOP triggers

- **STOP-0:** if you start writing a *fix* to `wat/service.wat`, `src/runtime.rs`, or the serve loop — STOP.
  This is recon. Your output is the located root + the captured reason.
- **STOP-1:** if neither `SCOUT[Finding A]` line fires (the death does not flow through `spawn.rs:665-689`)
  — STOP and report that fact + everything else the run showed (the death may take a different path — itself
  a finding). Do not go hunting other instrumentation sites.
- **STOP-2:** if the reason is empty/opaque (a bare panic with no message) — report it as-is; do not chase
  it deeper.

## Deliverable + cleanup

1. Which suspect (a/b/other) the run confirmed, with the **exact captured reason text + any `file:line`**.
2. The last tick state observed if visible.
3. **Revert the two `eprintln!`s** — `git diff src/kernel/spawn.rs` must be empty when you finish.
4. Do NOT commit. Do NOT un-`#[ignore]` the test. The orchestrator weighs the root and briefs the fix.

## Blast radius

`src/kernel/spawn.rs` only — two temporary `eprintln!`s, **reverted before you report.** No other files. No
fix. Scratch logs → `/tmp/claude-scout/`.
