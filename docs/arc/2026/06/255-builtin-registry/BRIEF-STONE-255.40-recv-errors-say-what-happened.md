# BRIEF — STONE 255.40: the Rust recv errors say what happened (prerequisites for the recv split)

**Drawn 2026-09-25 against `main` @ `e9ef4a03d`.** Floor 6127/6127, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 198. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first

`FINDING-INTUERI-naming-the-two-recv-vantages.md`, specifically **C3** and **C4**. The recv vocabulary split
(`PeerRecvOutcome`/`OwnerRecvOutcome`) cannot report `Failed` and `Malformed` honestly until these two
Rust-side facts are separated. This stone does **only** that, at the Rust layer. **No wat-visible outcome
variant changes here.**

## C3 — `RecvError::Failed` carries two facts

`comms::RecvError` (`src/comms/mod.rs` ~:345) has one `Failed(String)` that the process tier builds for
**both**:

- an **io failure**: the transport broke, and the far end's fate is unknown; and
- a **malformed frame or message**: invalid UTF-8, a wire decode failure, `FrameScan::Malformed`. The far
  end is **alive**; this message was bad.

intueri located the builders at `src/comms/process.rs` ~:751/:913/:937/:1129/:1133/:1210. **Verify each.**

**The work:** add `RecvError::Malformed(String)` and route every malformed-message builder to it, leaving
`Failed` for io only. Then follow the new variant through every consumer of `RecvError` (`typed_recv`,
`channel/transfer.rs`, `kernel/peer.rs`, `kernel/spawn.rs` `classify_*`, `kernel/message.rs` recv/select/poll,
`listener.rs`). **Where a consumer builds a wat outcome, keep the wat variant it builds today** (e.g. a
recv still yields `RecvOutcome.Lost` for both), so wat behaviour is unchanged. List every consumer, and say
what each does with `Malformed` now.

## C4 — the two crash classifiers disagree

`classify_peer_death` (`src/kernel/spawn.rs` ~:243) and `classify_peer_error` (~:273) read the same crash
channel after output EOF. intueri: when the crash channel's `recv` returns `Err(Failed)`,
`classify_peer_error` folds it into `Err(_) => PeerDeath::Closed` (**a clean exit**), while
`classify_peer_death` maps it to `Lost`. One fact, two verdicts.

**The work:** measure both, verbatim. Make them **one** classification: a single function, or one calling
the other, so the fold cannot diverge again. A crash-channel io failure is **not** a clean exit, so it
must not be reported as `Closed`. State which verdict it now gets, and why.

## Rows

- A Rust unit test for each new `Malformed` route (a malformed frame yields `RecvError::Malformed`; an io
  error yields `Failed`), asserted with `assert_eq!`/`matches!` on the variant.
- A unit test pinning C4: a crash-channel `Err(Failed)` after EOF gets the same verdict from both paths.
- **The wat-visible floor is unchanged** (no outcome variant moved). The census must show 0 rc flips.

## STOP triggers

1. A malformed-vs-io builder site cannot be told apart at its source (the error is already a string) →
   report each site verbatim; do not guess by string contents.
2. Any wat-visible outcome changes → STOP. That belongs to the recv split, not here.
3. A census file changes rc → report each.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `RecvError::Malformed` | exists; every malformed builder routes to it; `Failed` is io only |
| C4 | one classification; a crash-channel io failure is not a clean `Closed` |
| wat behaviour | unchanged; census 0 flips |
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
- Tests assert exact values (`assert_eq!`, a variant `matches!`), never loose `contains`/`starts_with`
  (the lint refuses them).
- ⭐ **A STOP trigger means STOP.** Report the shape; the builder rules.
- **Declare things where they are logical; a new file is fine** (builder's standing ruling).
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.40-recv-errors-say-what-happened.md` beside this brief.
