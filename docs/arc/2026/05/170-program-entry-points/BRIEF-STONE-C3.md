# BRIEF — Arc 170 Stone C3: type-keyword honesty fix (ThreadPeer + ProcessPeer field types)

**Phase:** Substrate honesty correction. Revises Stone C2's deliberate shortcut (using `:rust::crossbeam_channel::*` as the field-type keyword for ProcessPeer fields whose actual transport is PipeFd-backed).

**Predecessors:**
- Stone C1 SHIPPED — ThreadPeer<I,O> minted with `:rust::crossbeam_channel::*` field types
- Stone C2 SHIPPED — ProcessPeer<I,O> minted with the same naming shortcut
- Arc 109 K-channel rename — established `:wat::kernel::Sender/Receiver` as the canonical aliases (src/check.rs:3056-3057 + 492-493)

**Successor:** arc 203 slice 3 (ServiceWithProvisioning) — was blocked on this; unblocks when C3 ships.

## Goal

Rename `:rust::crossbeam_channel::Receiver/Sender` → `:wat::kernel::Receiver/Sender` in ThreadPeer + ProcessPeer field-type declarations, plus the `Sender/from-pipe` + `Receiver/from-pipe` return-type registrations, plus any consumer sweep where the dishonest names appear in type-annotation positions.

Substrate behavior MUST be unchanged. The aliases established by arc 109 already unify at the type-system level; this is pure rename for honesty.

## Required code path

### Substrate-side (src/types.rs)

Update ThreadPeer + ProcessPeer field declarations:
- `src/types.rs` ProcessPeer fields (lines 1049-1063): change `head: "rust::crossbeam_channel::Receiver"` → `head: "wat::kernel::Receiver"`; same for Sender
- Find ThreadPeer's analogous declaration (same file; same pattern); apply the same rename

### Substrate-side (src/check.rs)

Update `Sender/from-pipe` + `Receiver/from-pipe` return-type registrations. Grep for `Sender/from-pipe` + `Receiver/from-pipe` to locate; check the registered return type; change `:rust::crossbeam_channel::Sender<T>` → `:wat::kernel::Sender<T>` (and Receiver analog).

### Consumer sweep

Grep for explicit references to `:rust::crossbeam_channel::Sender<` or `:rust::crossbeam_channel::Receiver<` in:
- `tests/*.rs` — Rust-side wat::test! macro fixtures
- `wat-tests/*.wat` — user-side wat sources
- `wat/*.wat` — substrate-side wat sources

For each: substitute the honest name. Likely sites: arc 170 D-family tests (`tests/wat_arc170_d*`), Counter actor proofs (`wat-tests/counter-*-proof-*.wat`), arc 170 program contracts test, `wat/kernel/run_threads.wat`.

Update the substrate-side confession comment at `src/types.rs:1040-1045`. The shortcut is removed; the comment text becomes historical record of the original rationale + Stone C3 fix.

### What you do NOT need to do

- **No walker changes** — type-system aliases already unify; renaming the canonical name doesn't change resolution
- **No new error variants**
- **No runtime behavior change** — `typed_recv` / `typed_send` continue to branch transport-polymorphically on the Value variant
- **No arc 109 alias retirement** — the `:rust::crossbeam_channel::*` aliases stay registered as deprecated entry points; later cleanup may retire them, but not in scope here

## STOP triggers

1. **Aliases don't actually unify** — if substituting `:wat::kernel::Sender<T>` in a field declaration breaks type-checking somewhere (e.g., the alias isn't bidirectional, or specific call sites expect the crossbeam name literally), surface; we may need an alias-strengthening sub-stone before the rename
2. **Workspace baseline regresses** beyond the 3 pre-existing failures (deftest_wat_tests_tmp_totally_bogus + startup_error_bubbles_up_as_exit_3 + t6_spawn_process_factory_with_capture_round_trips) — STOP
3. **Sweep surface explodes** — if the dishonest names appear in >50 user-visible sites, STOP and surface; the rename may need to be sliced further
4. **Walker fires on the new names but not the old** — would mean aliases are walker-aware, breaking the unification claim; surface immediately

## HARD constraints

- **DO NOT commit.** Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. NEVER operate in `.claude/worktrees/`.
- DO NOT change runtime behavior — pure rename
- DO NOT add new substrate types/verbs/structs/special-forms
- DO NOT retire the `:rust::crossbeam_channel::*` aliases (deferred to later cleanup)
- DO NOT use `--no-verify` / `--no-gpg-sign`
- DO NOT touch Stones A/B/C1/C2/D1/D2 INSCRIPTIONs — they stay as historical record per `feedback_inscription_immutable`

## Decay disclosure (orchestrator)

The substrate touchpoints (src/types.rs:1003-1066 ProcessPeer; src/check.rs aliases at 3056-3057 + 492-493) are accurate as of 2026-05-17 — verified during BRIEF drafting via grep. ThreadPeer's analogous field declaration is presumed to mirror ProcessPeer's shape but not explicitly grepped at brief-time; sonnet verifies during execution.

The "aliases unify; no walker changes needed" claim rests on arc 109 + arc 133 inference-time alias reduction (src/check.rs:3408-3433 region). If sonnet finds the unification isn't as clean as the substrate code comments suggest, surface as a Stone C3 honest delta and we'll re-scope.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | ThreadPeer + ProcessPeer field-type declarations use `:wat::kernel::Sender/Receiver` | `grep -A3 "name: \":wat::kernel::ThreadPeer\\|ProcessPeer\"" src/types.rs` shows honest names |
| B | Sender/from-pipe + Receiver/from-pipe return type registrations use honest names | grep src/check.rs for the registrations confirms |
| C | Consumer sweep complete — no remaining `:rust::crossbeam_channel::Sender<\|Receiver<` in user-facing positions | `grep -rE ":rust::crossbeam_channel::(Sender|Receiver)<" tests/ wat-tests/ wat/` returns empty (or only documented internal references) |
| D | Workspace failure count = baseline | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures (the 3 documented pre-existing) |
| E | Runtime behavior unchanged | Counter actor proofs + Counter/Client capability proof + arc 170 D-family tests all pass (same as baseline) |

## Time-box

Predicted: 60-90 min sonnet. Hard stop: 120 min.

## Workspace baseline (verified post-arc-203-slice-2 commit `e8101d8`)

Clean except 3 pre-existing stable failures:
- `deftest_wat_tests_tmp_totally_bogus`
- `startup_error_bubbles_up_as_exit_3`
- `t6_spawn_process_factory_with_capture_round_trips` (NB: deadlocks; cargo no-fail-fast waits indefinitely; orchestrator reaps)

Post-C3 target:
- Pass count: = baseline (no new tests; existing tests continue to pass)
- Fail count: ≤ 3 (no regressions)

## On completion

1. Write `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-C3.md` per § SCORE methodology
2. Return final summary: rows passed/failed, workspace delta, file paths touched, honest deltas surfaced (especially if alias unification isn't as clean as assumed), suggested INTERSTITIAL corrections (if any)

You are launching now. T-minus 0.
