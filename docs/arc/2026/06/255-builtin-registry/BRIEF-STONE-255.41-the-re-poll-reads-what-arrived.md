# BRIEF — STONE 255.41: the process `poll` re-poll reads what arrived

**Drawn 2026-09-25 against `main` @ `b39cbf61a`.** Floor 6132/6132, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 198. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## The bug (intueri found it; the orchestrator verified it)

`src/kernel/message.rs` ~:2015–2025, the process-tier `poll` **re-poll** path (the spurious-`POLLIN` retry):

```rust
crate::comms::SelectOutcome::Recv { index: idx2, result: res2 } => {
    if idx2.0 == 0 {
        Value::Enum(Arc::new(EnumValue { … variant_name: "Shutdown".into(), … }))
```

On index 0 (the owner's lineage self-peer) it builds `ServiceEvent.Shutdown` **without reading `res2`**.
When the owner actually **sent an admin message** (e.g. `Admin.AllowPeer`), that message is **dropped**: the
serve loop takes `Shutdown` and exits, and the owner, blocked on its ack, sees `Closed`. The main (non-re-poll)
arm at ~:1794/:1801 reads the result correctly: `Ok(msg)` → `Admin`, `Err(_)` → `Shutdown`. **The re-poll
arm must do the same.**

## The work

1. **Mirror the main arm exactly** on the re-poll path: `Ok(msg)` → `ServiceEvent.Admin` (decoded the same
   way the main arm decodes it, through the same helper), and `Err(_)` → `ServiceEvent.Shutdown`. **Share
   the helper** the main arm uses rather than copying its body, so the two arms cannot drift again. If
   they cannot share one, say why.
2. **Do not rename anything.** `ServiceEvent`/`Shutdown` stay; the vocabulary split is the next stone.
   **Do not convert raises**; that is the split too.
3. **A row that reaches the re-poll path.** It is the spurious-`POLLIN` retry, so it may be hard to drive.
   Measure how to reach it deterministically (e.g. the conditions under which the first select returns
   no complete frame). If it **cannot** be driven from a test honestly, say so, and pin the fix with a
   Rust unit test of the shared helper plus a code-level assertion that both arms call it. **No sleeps as
   synchronisation.**
4. **Record for the coming merge:** the branch `sns-sqs` is converting service-to-service and bracket
   raises in the same code. In the SCORE, list the exact functions and line ranges this stone touched in
   `src/kernel/message.rs`, so the merge can be planned.

## STOP triggers

1. The main arm's admin decode and the re-poll path cannot share one helper without changing the main
   arm's behaviour → STOP, and report both verbatim.
2. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the re-poll index-0 arm | reads `res2`: `Ok` → `Admin`, `Err` → `Shutdown`, through the main arm's helper |
| a row pinning it | driven, or a unit test plus the shared-helper assertion, with the reason stated |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 198 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow).
- Tests assert exact values, never loose `contains`/`starts_with`.
- ⭐ **A STOP trigger means STOP.**
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.41-the-re-poll-reads-what-arrived.md` beside this brief.
