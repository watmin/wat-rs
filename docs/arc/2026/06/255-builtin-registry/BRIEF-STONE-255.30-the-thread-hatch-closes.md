# BRIEF — STONE 255.30: the thread escape hatch closes — every comm carries only data

**Drawn 2026-09-25 against `main` @ `eb5284ff3`.** Floor 6112/6112, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 211. **Executor: grok, via pulsare.** Strike, write the SCORE, then
`pulsare_yield kind=scored`.

## Read first, in order

1. `FINDING-what-relies-on-the-thread-escape-hatch.md`: what the hatch is (mostly **a check nobody
   wrote**). The §7 wall runs only at `self-peer`/`connect`/`accept`; the thread-spawn producer,
   `send`/`recv` and `poll` never ask purity. With the hatch closed statically, **only 3 fixtures**
   change verdict.
2. `FINDING-B3-a-thread-address-can-be-data.md` Part 2: **at runtime a thread self-peer is already an
   ordinary `Peer`** (`PEER_TYPE_PATH`). `ThreadSelfPeer` exists only in the checker, as the purity
   exemption. The spawn's real value (pre-wired channels, crash reason, join, readiness) lives on the
   owner's `Thread'`/`Process'`, **not on the child's channel**.
3. `WEIGH-STONE-255.28-…` (purity sees through generics) and `WEIGH-STONE-255.29-…` (a thread address is
   data; a `Shared` address is pure).

## Ruling (builder, 2026-09-25)

⭐ **Only data crosses a comm, on every locus. Resources are declared only in `:ephemeral`.** Its
consequence: **a self-peer is a plain `(:wat::kernel::Peer :- [S R])`, pure like every other peer.**

## The work

1. **Delete `ThreadSelfPeer`.**
   - Remove the checker type: `src/types.rs` roster entries, and every `src/check.rs` arm that exempts it
     (the finding lists ~:10954, ~:11278, ~:12363, ~:14991, plus `infer_thread_prog_type`'s acceptance).
   - Remove `(:wat::core::derive :wat::kernel::Peer :wat::kernel::ThreadSelfPeer)` (`wat/spawn.wat:282`).
   - **One way:** no alias.
2. **Respell every use as `(:wat::kernel::Peer :- [S R])`.** 88 code lines in 65 `.wat`/`.wat.bad`
   files, plus 18 mentions in `tests/*.rs` and 25 lines in `src/`. The `.wat` corpus is a **wat-fix
   codemod** (`wat/fix.wat`; copy a recorded rename such as
   `wat-scripts/fixes/rename-core-string-to-string.wat`). Dry-run on a `/tmp` copy and `diff`, apply with
   `./target/release/wat`, commit with a replay fixture. If the checker change and the rename must ship
   together, read `wat/fix.wat`'s header **STASH-DANCE** note.
3. **Every comm asks purity.** The §7 wall (`check_wire_peer_purity_span`) also runs at:
   - the **thread-spawn producer** (`infer_thread_prog_type`): the child's self-peer I/O types;
   - wherever else a `Peer`'s I/O becomes fixed and **no** check runs today.

   Census those sites. `send`/`recv` on a `Peer` whose type was checked at its producer need no second
   check: say which you chose and why.
4. **Rewrite the §7 wall's remedy text.** It tells users to switch to `ThreadSelfPeer`. The new remedy:
   carry resources in `:ephemeral` state, never on a channel.
5. **The 3 fixtures that exercise the hatch** (`tests/comms/probe_arc293_W2a_struct_no_cross.wat`,
   `…W2c_controls.wat`, `…W2d_positive.wat`) become **negative rows**: an impure payload on a thread peer
   is refused. Update their Rust drivers and say why in each.
6. **Rows** (`tests/…/probe_arc255_30_*`):
   - a struct payload on a thread self-peer: refused (pre-stone accepted);
   - a pure payload: accepted;
   - a defservice on a thread locus: runs, since the lineage `Status.Started` now carries a `Shared`
     address, which 255.29 made data;
   - the bracket thread pool with kwargs (`PoolMsg.Setup`): runs.

## STOP triggers

1. A **class (a)** site appears: a real resource on a channel that neither the finding nor 255.29 covered
   → STOP. Report each with its first error verbatim; the builder rules where it goes (`:ephemeral`).
2. A **checker case breaks** (class b) → STOP, report it.
3. A census file changes rc → report each with its first error.
4. The pre-wired spawn mechanism (crash reason, join, readiness) must not change. Any change in their
   behaviour → STOP.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `ThreadSelfPeer` in live code | 0 (census; survivors named: replay fixtures, comments) |
| the struct-on-thread-peer row | refused; pre-stone accepted |
| thread-locus defservice and kwargs bracket pool | run |
| the 3 hatch fixtures | now negative rows |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` (or classified) · NEW 2 / RECOVERY 0 · ≤ 211 |

## Doctrine (the house rules; `wat-rs/CLAUDE.md` is not injected, so they are here)

- The floor is **`scripts/floor.sh`** (`cargo nextest run --release`), captured to `.floor/`. Read the
  Summary line, never a piped exit code.
- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Never re-run to green. Copy the failing block
  **verbatim** from the captured log, name the assertion that fired, fix it at its cause if it is this
  stone's, then re-run the whole floor and say so.
- Also run clippy (force a recheck by touching a file), `scripts/replay/census.sh --diff` against a
  pre-change census you take first, `scripts/replay/delta.sh`, and the keyword heresy ledger (it may
  shrink, never grow).
- Line-pinned `.edn` goldens that move: recapture them with `UPDATE_EDN=1`, and show the diff is
  `:line` only.
- ⭐ **Prove every row can say BOTH words** on a pre-stone binary built from this brief's commit. Capture
  `rc=$?` into a variable on the **next** statement.
- **Timing, if you measure it:** A/B the pre and new binaries **interleaved in one session**. A band from
  another session is not a bar (`WEIGH-STONE-255.29-…`).
- `.wat` edits across files use a **wat-fix codemod** run with `./target/release/wat` (`cargo wat` is
  stale). Never python/sed for `.wat`.
- **If this brief contradicts the code, the code wins — say so plainly** in the SCORE.
- Commit locally with `git add -- <paths>`; **do not push**. Write
  `SCORE-STONE-255.30-the-thread-hatch-closes.md` beside this brief.
