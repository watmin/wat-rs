# BRIEF — STONE 255.36: `SendOutcome` says what happened (vocabulary 3 of 5)

**Drawn 2026-09-25 against `main` @ `ff53c96c0`.** Floor 6122/6122, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 208. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first

- `FINDING-INTUERI-the-outcome-vocabulary.md`, with the supervisor-pattern RULING.
- The two vocabulary stones before this one: `WEIGH-STONE-255.34-…` (Connect) and `WEIGH-STONE-255.35-…`
  (Accept). Copy their shape. **The lesson both carried: measure the consumers' fields before you fix a
  variant's shape.** 255.34's `Closed` kept a `[cause]` field because its arms bound one; 255.35's arms
  bound nothing.

## The lies, measured (intueri, verified by the orchestrator)

`SendOutcome` (`wat/kernel/outcomes.wat` ~:321–332, builders in `src/kernel/outcome.rs` ~:122–187):

- **`Closed`**, documented *"peer already cleanly closed"*. Its **only** producer is cell `None`, meaning
  **this handle** was closed by `close'` (`src/kernel/message.rs` ~:200/:236/:282). The far end did
  nothing. The corpus already misreads it: `wat/service.wat` ~:1898/:1920/:1941/:1996 comment the arm as
  "client gone → keep serving".
- **`Lost`**, *"disconnected mid-send"*. It is built by `send_outcome_from_error` (`outcome.rs:170`),
  which maps **both** `SendError::Disconnected` (a clean EPIPE: **the far end left**, the same fact recv
  calls `Closed`) **and** `SendError::Failed` (a real io error) to `Lost`, wrapped as `LociDiedError`.
  One is a departure, the other an io failure. Neither is a death.
- **`Stopped` is unreachable on the thread tier.** The thread comms maps every crossbeam send error to
  `SendError::Disconnected` (`src/comms/thread.rs:81`, verified).

## The new vocabulary — `:wat::kernel::SendOutcome`

| variant | fact | loci |
|---|---|---|
| `Sent` | delivered | all |
| `HandleClosed` | **this** handle was already closed (use-after-close) | all (cell `None`) |
| `Closed` | the far end is gone for good; no reason observed | all (`SendError::Disconnected`) |
| `Stopped` | a stop was requested while parked; nothing died | **process only** (the thread never builds it; say so) |
| `Failed [cause]` | an io failure | process (`SendError::Failed`) |

- **`Lost` is removed from `SendOutcome`.** Under the supervisor ruling a sender never receives a death
  reason, so no `LociDiedError` rides a send outcome.
- **`TrySendOutcome` follows the same rule** (2 consumer sites per variant; keep `WouldBlock`).

**Measure `Closed`'s shape before fixing it.** About 56 current `Lost` arms read their cause. If the far-end
`Closed` must keep a field so those arms stay legal, give it `[cause <- Failure]`, a description of the
departure (not a death reason), as 255.34 did. If it can be nullary honestly, do that. **Say which, and why,
with the count.**

## The work

1. The `defenum`s, the Rust builders (`send_outcome_from_error` split: `Disconnected` → `Closed`,
   `Failed` → `Failed`; cell `None` → `HandleClosed`), and the docs, each stating its fact and its loci.
2. **Consumers: 205 arms per variant in 88 files**, by **wat-fix codemod** (dry-run and `diff`, apply with
   `./target/release/wat`, commit with a replay fixture):
   - `SendOutcome.Closed` → `SendOutcome.HandleClosed` (same body);
   - `SendOutcome.Lost` → **two** arms, `Closed` and `Failed`, each with the same body (adjusting the
     binding to the shape you measured);
   - `Sent` and `Stopped` stay;
   - the same for `TrySendOutcome`.
3. **Correct the misreading comments:** `wat/service.wat` ~:1898/:1920/:1941/:1996 and any other comment
   that calls the old `Closed` "client gone". After the rename, the arm that means "client gone" is the
   new `Closed`.
4. Rust tests and `.jsonl`/`.edn` goldens that name the old variants: update them. A line-pinned `.edn` that
   moves: recapture it with `UPDATE_EDN=1`, a line-only diff.
5. **Rows** (`tests/…/probe_arc255_36_*`):
   - a send on a handle after `close'` → `HandleClosed` (pre-stone `Closed`);
   - a send to a far end that left, on both loci → `Closed` (pre-stone `Lost`);
   - a thread send never yields `Stopped`: cite `comms/thread.rs:81` rather than write a row that cannot
     fire.

## STOP triggers

1. A consuming arm's body depends on the old `Closed`'s meaning ("client gone") in a way the rename to
   `HandleClosed` would break → report each site. Do not guess.
2. A `Lost` arm's body reads its cause in a way that differs between the departure and the io failure →
   report it.
3. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `SendOutcome.Lost` / `TrySendOutcome.Lost` in live code | 0 |
| the rows | as above; pre-stone words recorded |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 · ≤ 208 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow).
- ⭐ **Prove every row can say BOTH words** on a pre-stone binary built from this brief's commit. Capture
  `rc=$?` into a variable on the **next** statement.
- A `.wat` corpus migration is a **wat-fix codemod** run with `./target/release/wat`. Never python/sed for
  `.wat`. If the `src/` change and the codemod must ship together, read `wat/fix.wat`'s header
  **STASH-DANCE** note.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.36-send-says-what-happened.md` beside this brief.
