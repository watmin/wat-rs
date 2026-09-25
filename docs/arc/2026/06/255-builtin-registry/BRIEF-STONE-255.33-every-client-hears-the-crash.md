# BRIEF — STONE 255.33: every connected client hears the crash

**Drawn 2026-09-25 against `main` @ `3ed90abeb`.** Floor 6118/6118, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 208. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Ruling (builder, 2026-09-25) — the supervisor pattern

*"a dead service should optimistically notify all connected clients that its crashing without providing a
reason, the reason may only be provided to the admin handle who owns the service."* Recorded in
`FINDING-INTUERI-the-outcome-vocabulary.md` (final section).

## The violation — measured by 255.32 and re-run by the orchestrator

`tests/services/probe_arc255_32_two_clients_see_the_crash.wat` (committed):

```
"thread client-a Lost peer crashed (abnormal far-side crash — no reason; …)"
"thread client-b Lost peer crashed (…)"
"thread owner Lost P32-SERVICE-PANIC-REASON"
"process client-a Lost peer crashed (…)"
"process client-b Lost io_uring read failed"        ⛔ the idle client; not the notice
"process owner Lost P32-SERVICE-PANIC-REASON"
```

On the thread tier both clients hear the reasonless notice. **On the process tier only the client whose op
triggered the crash does.** The idle client reads a raw transport failure (`src/comms/process.rs` ~:751/:931,
then `src/kernel/message.rs` ~:603).

The mechanism: `serve-dispatch-op'` (`src/kernel/serve.rs` ~:154–209), on a handler panic, calls
`broadcast_peer_crashed_best_effort(&clients)` (`src/kernel/peer.rs` ~:479). That walks **every** peer in
`clients` and `try_send`s the `PEER_CRASHED_SENTINEL` (`peer.rs` ~:417–427; socket tier:
`tx.try_send(PEER_CRASHED_SENTINEL.to_string())`).

## The work

1. **Find the cause. Measure; do not guess.** Candidates, **all unverified**:
   - the idle client is not in `clients` at the moment of the panic;
   - the socket `try_send` enqueues, but the process exits before the frame is flushed;
   - **the dying process closes a socket that still has unread data, the kernel sends a RST, and the
     client's pending read of the queued sentinel is discarded as `ECONNRESET`**;
   - something else.

   Instrument it and say which one, with evidence (strace or an env-gated probe; remove it before
   committing).
2. **Cure it at the cause** so that **every** connected client of a crashing service, on **both** loci,
   gets the reasonless notice. Keep the broadcast **best-effort and non-blocking**: a dying process must
   never wait on a peer that is not draining. If the honest cure needs a guarantee the kernel will not give
   (e.g. RST discards unread data regardless), **STOP** and report the shape, with the measurement.
3. **Update 255.32's probe row** so the process client-b line is the notice. The owner lines stay the
   reason.
4. **Do not rename variants.** Clients still see `RecvOutcome.Lost`; the vocabulary is its own stones.

## STOP triggers

1. The cure needs a blocking wait, or a delivery guarantee the transport cannot give → STOP, report.
2. The fix leaks the reason to any client → STOP.
3. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the cause | named, with evidence |
| the 255.32 probe | all four client lines are the reasonless notice; both owner lines are the reason; pre-stone the process client-b line is `io_uring read failed` |
| repeated runs | the probe run ×10, identical every time (this is a race until proven otherwise) |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 208 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so. **This stone touches a race:** a row that passes once proves nothing,
  so run it repeatedly and report the count.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow).
- ⭐ **Prove every row can say BOTH words** on a pre-stone binary built from this brief's commit. Capture
  `rc=$?` into a variable on the **next** statement.
- **No sleeps as synchronisation**: a wait arrives as an fd event or it does not arrive honestly (`mora`).
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.33-every-client-hears-the-crash.md` beside this brief.
