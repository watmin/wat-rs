# BRIEF — STONE 255.42: drive the re-poll path from Rust; the text-count guard goes

**Drawn 2026-09-25 against `main` @ `7e0f6c8da`.** Floor 6135/6135, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 198. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Why

255.41 fixed the re-poll bug (both index-0 arms now call `process_lineage_event`), but pinned it with
`include_str!("../../src/kernel/message.rs").matches("process_lineage_event(").count() == 4`
(`tests/kernel/probe_arc255_41_both_poll_arms_call_the_lineage_helper.rs`). **That is a substring count over
source text.** A comment moves it, and a moved call site can keep the count while the bug returns. The
builder: *"all of wat is strongly structural.... these kind of matches are incredibly shitty"*, and:

> *"can we fabricate the necessary trigger conditions via rust orchestration?... we can do whatever awful
> situation building to manifest the triggers via rust in our tests?"*

**Yes.** The row must **drive the real re-poll path** and observe the outcome.

## The trigger

In `eval_poll_prime` (process tier, `src/kernel/message.rs`, the 255.41 ranges): `select_raw` returns the
**listener** readable → the non-blocking `accept` returns **`EAGAIN`/`WouldBlock`** → the loop selects again
(the re-poll) → index 0 (the owner's lineage) carries a frame. Readiness is a snapshot, so the connection
must vanish **after** the first select observes it and **before** `poll`'s accept.

## The work

1. **Build the situation in a Rust test**, with real sockets wherever possible: a real `SocketListener`, the
   lineage self-peer, a client. Find a **deterministic** way to hit `accept` → `EAGAIN` after readiness was
   observed:
   - **(a) an fd-level trick**, if one is deterministic (not a race);
   - **(b) otherwise a minimal test seam on the accept step**, e.g. an injectable accept or a
     `#[cfg(test)]` hook that makes exactly one accept return `WouldBlock`, with **everything else real**.
     Keep the seam as narrow as the one kernel event it simulates, and say in the SCORE why it is honest.
   - **No sleeps as synchronisation, and no retry-until-it-happens.** A test that hits the path by luck
     proves nothing.
2. **The row:**
   - the owner sends an admin message; the re-poll path is driven; `poll` returns `ServiceEvent.Admin`
     carrying it;
   - the twin: the owner drops the lineage, and the re-poll returns `ServiceEvent.Shutdown`.

   **Both words on the pre-255.41 code:** build the binary or library at `3e2db3b6c` (before 255.41's fix),
   or revert the re-poll arm in a scratch copy, and show the Admin row **fails** there. That is the proof the
   row can see the bug.
3. **Delete the text-count test**
   (`tests/kernel/probe_arc255_41_both_poll_arms_call_the_lineage_helper.rs`). The driven row replaces it.
   Keep 255.41's unit test of the helper's two verdicts.

## STOP triggers

1. Neither (a) nor a narrow (b) can hit the path deterministically → STOP, and report what you tried.
2. The seam would have to change production behaviour (not test-only) → STOP, report the shape.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the re-poll path | reached deterministically, the method named |
| the Admin row | pass on HEAD; **fail** on the pre-255.41 code |
| the text-count test | deleted |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 198 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so. **This row touches a race by design:** run it at least 20 times and
  report the count, all identical.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger.
- Tests assert exact values, never loose `contains`/`starts_with`, **and never a count of source text**.
- ⭐ **A STOP trigger means STOP.**
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.42-drive-the-re-poll-from-rust.md` beside this brief. Record any `message.rs` lines
  touched, for the `sns-sqs` merge.
