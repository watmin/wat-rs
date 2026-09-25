# BRIEF — STONE 255.38: the owner's end is its own type

**Drawn 2026-09-25 against `main` @ `3fdb5eb4b`.** Floor 6124/6124, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 208. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first

- `WEIGH-STONE-255.37-measure-the-owner-family.md` and your own `SCORE-STONE-255.37-…`. That measurement is
  this stone's map, and its diff is in `probes-255.37/255.37-worktree.diff.txt`.
- `FINDING-INTUERI-naming-the-two-recv-vantages.md` (C1).

## Rulings (builder, 2026-09-25)

- **M0:** `recv` (and `send`/`try-send`/`select`/`poll`/`close`) stay kernel intrinsics. Their type rules
  tell the owner's end from a counterparty's end by the receiver's **declared family**, not by head-name
  strings.
- **The owner's end is its own type.** A spawn handle (`:wat::kernel::Thread`/`Process`) owns its child:
  crash channel, join, lineage. A `:wat::kernel::Peer` is a counterparty's end: data only, never a reason.
  **The edges `(derive :wat::kernel::Thread :wat::kernel::Peer)` and its `Process` twin
  (`wat/spawn.wat:267-268`) go.**
- **S2 (four YES):** the owner family is declared as a **surface**, `(:wat::core::defsurface :wat::spawn::Spawned
  :- [S R] …)`, whose **features are what an owner can do**. `Thread` and `Process` implement it with
  `extend-type` and binders (`(:wat::core::extend-type :- [S R] (:wat::kernel::Thread :- [S R])
  (:wat::spawn::Spawned :- [S R]) …)`, per 255.22). Rejected:
  - S1, the phantom-field struct: Obvious N, it reads as a carrier;
  - S3, exempting markers from the unconsumed-parameter rule: Simple N.

## The work

1. **Declare `Spawned` as a surface.** Measure first which owner operations belong on it. The owner's
   lifecycle is `close`/join; data flows through `send`/`recv`, which are intrinsics. **Choose the features
   so the surface says what an owner is, not what every handle can do.** If the honest feature set is
   empty, or a surface cannot carry `:- [S R]` without a feature consuming them, **STOP and report the
   shape.** Say what you declared and why.
2. **Replace the two `derive … Spawned` markers** (`spawn.wat:257-258`) with the `extend-type` binders.
   **Delete the two `derive … Peer` edges.**
3. **The family rule:** `project_peer_io`, `select`, `poll`, `close` (255.37 measured these) accept an owner
   head **by its satisfaction of `Spawned`**, and a `Peer` as a `Peer`. That turns the head-name compares
   into a declaration-driven rule; the ledger should shrink (208 → about 198 in the measurement). **Measure
   how the rule reads surface satisfaction**, since 255.37's rule read the `derive` edge.
4. **Retype the seams, one at a time, re-measuring after each. It is a cascade: each retyped seam may
   uncover the next.**
   - `spawn-runner`'s signature (`wat/bracket.wat:241/321`);
   - `Launched.handle` (`wat/spawn.wat:329`);
   - `recv-all` / `recv-all-loop` (`spawn.wat:658/683`);
   - whatever the cascade reveals below them (255.37 predicts the defservice `Handle.handle` and
     `bracket.wat`'s runner-vector fold).

   Each retype is from `(Peer :- [..])` to `(Spawned :- [..])` **only where the value is an owner handle**:
   a judgement per site, with the site listed in the SCORE. Report the cascade as a sequence: seam retyped
   → reds now → next seam.
5. **Leave `recv`'s outcome type as `RecvOutcome`.** The vocabulary split (`PeerRecvOutcome`/`OwnerRecvOutcome`)
   is the next stone. This one makes the two ends two types.
6. **Rows:** an owner can still `send`/`recv`/`close` its child (thread and process); a `Peer` from
   `connect` cannot be passed where `(Spawned :- [..])` is expected (refused); a `Thread` cannot be passed
   where a `Peer` is expected (refused; pre-stone accepted).

## STOP triggers

1. **Class (c)**: any code that must receive from **either** an owner or a peer → STOP, and report it
   verbatim. 255.37 found none at the first seam; the cascade may reveal one.
2. The cascade touches **more than 15 sites** → STOP, and report the sequence so far.
3. A surface cannot express the owner family honestly (step 1) → STOP.
4. A census file changes rc → report each with its first error.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `derive … Peer` for Thread/Process | gone |
| `Spawned` | a surface; `Thread`/`Process` implement it |
| the three rows | as above; pre-stone words recorded |
| the head-name compares, and the ledger | fewer; the ledger ≤ 208 (tightened as its message says) |
| floor · clippy · census · delta | green · 0 · `no STOP-8` · NEW 2 / RECOVERY 0 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block **verbatim**
  from the captured log, name the assertion that fired, fix it at its cause if it is this stone's, then
  re-run the whole floor and say so. A cascade's reds are the progress meter (examinare), not a crisis.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow).
- Line-pinned `.edn` goldens that move: recapture them with `UPDATE_EDN=1`, and show a `:line`-only diff.
- ⭐ **Prove every row can say BOTH words** on a pre-stone binary built from this brief's commit. Capture
  `rc=$?` into a variable on the **next** statement.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.38-the-owner-end-is-its-own-type.md` beside this brief.
