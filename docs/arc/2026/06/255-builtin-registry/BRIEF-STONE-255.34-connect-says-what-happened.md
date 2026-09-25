# BRIEF — STONE 255.34: `ConnectOutcome` says what happened (vocabulary 1 of 5)

**Drawn 2026-09-25 against `main` @ `a14c9d2f3`.** Floor 6118/6118, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 208. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first

`FINDING-INTUERI-the-outcome-vocabulary.md`, the whole file: the intueri verdict, the fact table, **and the
supervisor-pattern RULING** in its final section. The builder ruled the direction with four YES
(2026-09-25):

- **V1:** a fixed set of facts per outcome type, one name per fact, and a producer builds only the facts
  its locus can produce;
- **R2:** **drop `Refused`**. It has no producer on any current locus, and a future TCP locus will add it
  back as a language update;
- the vocabulary lands **one outcome type per stone**. **This is stone 1: `ConnectOutcome`.**

## The lies, measured (intueri, verified by the orchestrator)

- `Refused`: documented *"RETRYABLE transport (the server may come up)"* (`wat/kernel/outcomes.wat:504-505`,
  `src/kernel/address.rs` ~:48/:175/:231, `src/kernel/outcome.rs` ~:376). **No producer delivers
  "retryable":**
  - thread `address.rs` ~:178 fires on a dropped rendezvous receiver, which never returns;
  - process `address.rs` ~:235 fires on an autobound name nothing rebinds, and maps **every**
    `connect_addr` error, not just ECONNREFUSED.
  - **Measured: nothing in the corpus retries.** The consuming arms are `assertion-failed!` (or print).
- `Rejected` holds **two facts**:
  - process `address.rs` ~:267: the answerer is not the minter (identity);
  - thread `address.rs` ~:142: this address value is an inert wire copy (255.31).

## The new vocabulary — `:wat::kernel::ConnectOutcome :- [S R]`

| variant | fact | produced by |
|---|---|---|
| `Connected [peer]` | dialed and admitted | both loci (unchanged) |
| `Closed` | nothing is listening at this address, and it is gone for good | thread `address.rs` ~:178; process `address.rs` ~:235 when the error is ECONNREFUSED/ENOENT (**measure which errnos the autobind path can actually yield**) |
| `Undialable [cause]` | this address value cannot be dialed here: an inert wire copy | thread `address.rs` ~:142 |
| `WrongPeer [cause]` | the answerer is not who the address names | process `address.rs` ~:267 |
| `Failed [cause]` | a transport io failure | process `address.rs` ~:252/:282, **and** any `connect_addr` error that is not the gone-for-good fact |

⭐ **`Refused` is removed**, with its "retryable" doc, everywhere it is repeated (`address.rs`, `outcome.rs`,
`outcomes.wat`). Rust `ConnectFail` gets the same variants. Rewrite each doc so it states **its fact and
which loci produce it** (e.g. *"produced only where the far end is an independent process; no thread locus
produces this"*).

## The work

1. **The `defenum` and the Rust `ConnectFail`**, with every producing arm returning the variant for its
   fact, as in the table.
2. **The consumers: 226 arms per variant, in 129 files.** A **wat-fix codemod** (dry-run on a `/tmp` copy
   and `diff`, apply with `./target/release/wat`, commit with a replay fixture):
   - `ConnectOutcome.Refused` arm → a `ConnectOutcome.Closed` arm (same body);
   - `ConnectOutcome.Rejected` arm → **two** arms, `Undialable` and `WrongPeer`, each with the same body;
   - `Failed` and `Connected` stay.

   The matches must stay exhaustive. A `.wat.bad` fixture that pins the old names is migrated with the
   rest. Check whether its `.rs` driver pins a message, and update it.
3. **Rust tests and goldens** that name the old variants: update them. A line-pinned `.edn` golden that
   moves: recapture it with `UPDATE_EDN=1` and show the diff is the rename only.
4. **Rows** (`tests/…/probe_arc255_34_*`):
   - dial a dropped thread listener → `Closed` (pre-stone `Refused`);
   - an inert wire copy → `Undialable` (pre-stone `Rejected`);
   - a process wrong-peer → `WrongPeer`, if there is an existing way to drive it (there are OnlyThisPeer
     probes); if not, say so;
   - a dead process listener → `Closed`.

## STOP triggers

1. The process `connect_addr` path can produce an errno that is **genuinely retryable** on this locus (the
   far end can come back at the same address) → STOP, report it with evidence. That would be fact O, which
   the ruling reserves.
2. A consumer arm's body **depends on** which of `Undialable`/`WrongPeer` fired (reads its cause
   differently) → report each site; do not guess.
3. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `ConnectOutcome.Refused` / `ConnectFail::Refused` / "RETRYABLE" in live code | 0 |
| the rows | as above; each pre-stone word recorded |
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
- A `.wat` corpus migration is a **wat-fix codemod** run with `./target/release/wat` (the `cargo wat` in
  `~/.cargo/bin` is stale). Never python/sed for `.wat`. If the `src/` change and the codemod must ship
  together, read `wat/fix.wat`'s header **STASH-DANCE** note.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.34-connect-says-what-happened.md` beside this brief.
