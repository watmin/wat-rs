# BRIEF — STONE 255.35: `AcceptOutcome` says what happened (vocabulary 2 of 5)

**Drawn 2026-09-25 against `main` @ `3c11f46da`.** Floor 6120/6120, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 208. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first

- `FINDING-INTUERI-the-outcome-vocabulary.md`, including the supervisor-pattern RULING.
- `WEIGH-STONE-255.34-connect-says-what-happened.md`, **the shape of vocabulary 1**. Copy it. Its lesson:
  **measure the consumers' fields before you fix a variant's shape.** The fact table is a direction, and
  the arms decide the fields.

## The lies, measured (intueri, and 255.31's table)

`AcceptOutcome` (`wat/kernel/outcomes.wat` ~:481) and Rust `AcceptFail` (`src/kernel/listener.rs` ~:48–55):

- `Closed`, documented *"rendezvous shut down / address dropped (clean)"*, **folds a stop into a drop**:
  - thread `listener.rs` ~:129–131 maps **both** `Disconnected` (every sender gone: a real drop) **and**
    `Shutdown` (a substrate stop) to `Closed`;
  - **process `listener.rs` ~:393 is only ever a stop** (`SelectOutcome::Shutdown`). On the process locus
    `Closed` never means "address dropped".
- `Failed` on the thread locus is **unreachable**: `listener.rs` ~:135 fires only from `RecvError::Failed`/
  `PeerCrashed`, which the thread comms recv never yields (`src/comms/thread.rs` ~:199–220). Yet its doc
  names a decode error.

## The new vocabulary — `:wat::kernel::AcceptOutcome :- [R S]`

| variant | fact | loci |
|---|---|---|
| `Accepted [peer]` | a peer was admitted | all (unchanged) |
| `Closed` | the listener is gone for good (every sender dropped) | thread (`Disconnected`) |
| `Stopped` | a stop was requested; nothing closed | thread (`Shutdown`), process (~:393) |
| `Failed [cause]` | an io failure | **process only**; say so in the doc. The thread arm is unreachable: measure it, and delete it if it is |

**Measure the consumers' fields first** (4 files carry each variant). If a consuming arm binds a field on
`Closed`, keep the field and say why, as 255.34 did.

## The work

1. The `defenum`, the Rust `AcceptFail`, and each producing arm returning its fact (the table).
2. **Consumers (4 files):** a `Closed` arm stays `Closed`, and a **new `Stopped` arm** is added with the
   same body, so the matches stay exhaustive. **Read each body.** If it treats a stop as a drop in a way
   that is now wrong, report it; do not guess. With only 4 sites, a hand edit is fine. Say so.
3. Rust tests and goldens that name the variants: update them. A line-pinned `.edn` golden that moves:
   recapture it with `UPDATE_EDN=1` and show the diff is the line number only.
4. **Rows** (`tests/…/probe_arc255_35_*`):
   - accept on a dropped thread listener → `Closed`;
   - accept interrupted by a stop, on both loci → `Stopped` (pre-stone `Closed`). If a stop cannot be
     driven from wat in a test, say how you drove it.

## STOP triggers

1. A consumer's body depends on stop-vs-drop in a way the rename cannot carry → report each site.
2. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the thread `Closed`/`Stopped` split, the process `Stopped` | as in the table; pre-stone words recorded |
| the thread `Failed` arm | measured unreachable and removed, or a reason it stays |
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
- A `.wat` corpus migration of more than a handful of sites is a **wat-fix codemod** run with
  `./target/release/wat`. Never python/sed for `.wat`.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.35-accept-says-what-happened.md` beside this brief.
