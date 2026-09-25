# BRIEF — STONE 255.37: MEASURE the owner family (drop `Thread/Process derive Peer` in a worktree)

**Drawn 2026-09-25 against `main` @ `e19e60cd3`.** Floor 6124/6124. **Executor: grok, via pulsare.**
**MEASUREMENT ONLY: nothing lands on `main`.** Write the SCORE, then `pulsare_yield kind=scored`.

## Read first

- `FINDING-INTUERI-naming-the-two-recv-vantages.md`: W2 (two recv outcome types) is named
  `PeerRecvOutcome`/`OwnerRecvOutcome`, and its **C1** is why this measurement exists.
- `FINDING-INTUERI-the-outcome-vocabulary.md`, including the supervisor-pattern ruling.

## Ruled (builder, 2026-09-25)

**M0:** `recv` stays a kernel **intrinsic**, not a defclause and not a surface. Its type rule picks the
outcome from the receiver's **declared family**. A receiver that derives `:wat::spawn::Spawned` (the
**owner's** end: crash channel, join, lineage) gets `OwnerRecvOutcome`. A `:wat::kernel::Peer` (a
**counterparty's** end: data only, never a reason) gets `PeerRecvOutcome`. The edge
`(:wat::core::derive :wat::kernel::Thread :wat::kernel::Peer)` and its `Process` twin (`wat/spawn.wat:267-268`,
arc 291) **go**. The owner keeps `send`/`recv` on its child **through the owner family**, not by being a
`Peer`.

## What to measure, in a git worktree

Use `git worktree add <session-scratch>/wt-255.37 HEAD`, and symlink `../holon-rs` beside it if the build
needs it. Never edit the main tree.

1. **Make `Spawned` a real, parametric type** so it can carry channel types: `(:wat::spawn::Spawned :- [S R])`.
   Today it is a bare, parameterless `derive` target with no declaration (`spawn.wat:254-258`). Choose the
   declaration form and **say what you chose and why**.
2. **Drop the two `derive … Peer` edges.**
3. **Retype the rule, minimally:** `project_peer_io` (`src/check.rs` ~:11247) compares head names
   `"wat::kernel::Thread" || "wat::kernel::Process" || "wat::kernel::Peer"`. `src/check.rs` has **33** such
   head-name compares. Make the owner/peer distinction by **family** (derives `Spawned` vs is `Peer`) where
   `recv`/`send`/`select`/`poll`/`close`/`try-send` need it. `recv`'s *outcome type* stays `RecvOutcome` for
   now. **This measurement is about the typing edge, not the vocabulary.**
4. **Run the floor** (`scripts/floor.sh` in the worktree) and a `--check` sweep of every tracked
   `.wat`/`.wat.bad` with both binaries. Classify **every** red:
   - (a) an owner handle typed as `Peer` (a field, parameter or return) that must be retyped to the owner
     family. Known: `Launched.handle` `spawn.wat:329`, every defservice `Handle.handle`, `bracket.wat:242`/`:762`,
     `recv-all-loop` `spawn.wat:658`/`:683`;
   - (b) `select`/`poll` over a vector mixing owner and peer handles (bracket's collect-loop, …);
   - (c) a site that genuinely needs one piece of code to receive from **either** an owner or a peer.
     **The key question: does any exist?**
   - (d) a checker case the change broke;
   - (e) other.

   Counts, files, and one whole failing block verbatim per class.
5. For class (a), say whether a **wat-fix codemod** could retype the sites (a type-position rewrite
   `(:wat::kernel::Peer :- [..])` → `(:wat::spawn::Spawned :- [..])` **only** where the value is an owner
   handle), or whether each needs judgement.
6. Report the head-name compare count before and after, and which ones the family rule replaced.

## Doctrine

- The floor is `scripts/floor.sh` (release nextest) captured to `.floor/`. Read the Summary line.
  ⛔ **There is no known flake**: never re-run to green; quote the failing block verbatim and name the
  assertion.
- Capture `rc=$?` on the **next** statement; never put `$?` in a string that contains `$(…)`.
- **If this brief contradicts the code, the code wins — say so plainly.**
- **Write `SCORE-STONE-255.37-measure-the-owner-family.md` in the MAIN tree's
  `docs/arc/2026/06/255-builtin-registry/`, and commit ONLY that file on `main`** (`git add -- <that path>`).
  Save the worktree diff to the session scratchpad and name its path in the SCORE. Remove the worktree
  when you are done (`git worktree remove`). **Do not push.**
