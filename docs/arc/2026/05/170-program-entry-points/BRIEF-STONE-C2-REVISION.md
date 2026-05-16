# Arc 170 Stone C2 REVISION BRIEF — substrate-composition proof

**Phase:** Revision of Stone C2 per direction (b) settled 2026-05-16. See `INTERSTITIAL-REALIZATIONS.md` § 2026-05-16 (Stone C1 SHIPPED; Stone C2 PARTIAL — "mock is the easy framing"). See also `SCORE-STONE-C2-PROCESSPEER.md` § Revision (post-direction-(b)).

**Predecessor state on disk (uncommitted):**
- `src/types.rs` +61 — ProcessPeer<I, O> struct registration — **KEEP**
- `src/check.rs` +52 — Process/readln + Process/println type schemes — **KEEP**
- `src/runtime.rs` +151 — eval handlers + dispatch arms — **KEEP**
- `src/typed_channel.rs` +87 — `make_process_peer_for_test` helper — **RETIRE in this revision**
- `tests/wat_arc170_stone_c2_processpeer.rs` — 3 mock tests — **RENAME + REWRITE T2**

## Goal

Convert the test from Rust-mock-driven (sub-decision (b) sonnet picked from the original BRIEF) to a real-spawn substrate-composition proof. The test exercises the existing wat-level primitives that Stone D's `run-processes` bracket macro will compose. Stone D is the user-facing surface; Stone C2's test is substrate-composition proof — not the user-facing IPC pattern.

## Required path (NO alternatives)

**NO new substrate verbs.** The reflex to mint `:wat::kernel::ProcessPeer/from-process` is REJECTED — see INTERSTITIAL § First reflex (rejected). Every primitive needed already exists:

- `:wat::kernel::spawn-process forms -> :Process<I, O>` (existing — see `tests/probe_spawn_process_stdio.rs` for the AST construction pattern)
- `:wat::kernel::Process/stdin proc -> :wat::io::IOWriter` (existing — `src/check.rs:12916`)
- `:wat::kernel::Process/stdout proc -> :wat::io::IOReader` (existing — `src/check.rs:12925`)
- `:wat::kernel::Sender/from-pipe writer -> :Sender<O>` (existing eval primitive — `src/runtime.rs:17550`; production caller `wat/test.wat:921`)
- `:wat::kernel::Receiver/from-pipe reader -> :Receiver<I>` (existing eval primitive — `src/runtime.rs:17580`; production caller `wat/test.wat:928`)
- `:wat::kernel::ProcessPeer/new rx tx -> :ProcessPeer<I, O>` (auto-synthesized — `src/runtime.rs:1879`+`:1906`; parametric handling at `:1865`)
- `:wat::kernel::Process/println peer data -> :nil` (Stone C2 ships — `src/runtime.rs:17319` adjacent eval handler)
- `:wat::kernel::Process/readln peer -> :I` (Stone C2 ships — `src/runtime.rs:17261` adjacent eval handler)
- `:wat::kernel::Process/drain-and-join proc -> :Result<...>` (Stone A shipped — commit `2a198bd`)

## Tasks

### 1. Retire `make_process_peer_for_test` from `src/typed_channel.rs`

Delete the function (currently lines ~574-659) AND the inner `make_pipe_for_test` helper if no other caller. Audit grep for any remaining callers; the only caller was the old Stone C2 T2.

### 2. Rename test file

`tests/wat_arc170_stone_c2_processpeer.rs` → `tests/wat_process_peer_ipc_round_trip.rs`

Concept-anchored, not implementation-anchored. Future agents searching for "how does IPC work" find this.

### 3. Rewrite the test file's header

Replace the existing module-level doc comment with framing that:
- Names Stone D's `run-processes` bracket macro as the **user-facing surface**
- States this file is **substrate-composition proof**, NOT the user-facing IPC pattern
- Documents that user code never writes spawn-process + ProcessPeer/new + drain-and-join manually — it writes the bracket
- Mentions that `drain-and-join` is public (per Stone B's intentional design) but its public availability does NOT promote it to the user-facing IPC surface

### 4. Keep T1 (type mint) — both ProcessPeer<i64,String> and ProcessPeer<String,i64> type-check

Currently `stone_c2_process_peer_type_mint_both_orientations_type_check`. Function name MAY be renamed to drop the "stone_c2" prefix in favor of a concept name like `process_peer_type_mints_in_both_parametric_orientations`. The test body stays the same.

### 5. Keep T3 (asymmetry assertion) — no ProcessPeer/Server type emitted

Currently `stone_c2_process_peer_is_client_side_only_no_server_type_emitted`. Rename to a concept name like `process_peer_is_client_side_only_no_server_variant_emitted`. Body unchanged.

### 6. REWRITE T2 — real subprocess round-trip composing existing primitives

Replace the mock-driven T2 with a real-spawn round-trip. The test:

1. Spawns a subprocess via `:wat::kernel::spawn-process` whose child program is a single `:user::main` that reads one line via ambient `(:wat::kernel::readln -> :wat::core::String)` and echoes it back via `(:wat::kernel::println line)`.
2. Constructs a `:wat::kernel::ProcessPeer<wat::core::String, wat::core::String>` via `ProcessPeer/new`, composing `Receiver/from-pipe` over `Process/stdout server` and `Sender/from-pipe` over `Process/stdin server`.
3. Calls `(:wat::kernel::Process/println peer "hello")` to send the request.
4. Reads the reply via `(:wat::kernel::Process/readln peer)`.
5. Calls `(:wat::kernel::Process/drain-and-join server)` for clean shutdown.
6. Asserts reply equals "hello".

Variable naming uses **client/server** roles (the test conversation framing), NOT child/parent (OS-tree). Server = the spawned subprocess (the "server" of echo requests). Client = the test process running spawn-process.

Function name: e.g., `process_peer_round_trips_string_via_real_subprocess`.

**Construction pattern:** mirror `tests/probe_spawn_process_stdio.rs` and `tests/wat_arc170_program_contracts.rs:t4_spawn_process_keyword_fn_round_trips_typed_value` for how to construct the `(:wat::core::forms ...)` argument to spawn-process from a Rust source string (parse_all → wrap in keyword). Then drive the rest (peer construction + verbs + drain-and-join) via embedded wat source using `wat::parse_one!`.

**Hermetic time-bound:** if `eval` blocks indefinitely on a wat-level deadlock, fail fast with a panic carrying diagnostics from the subprocess stderr (mirror the stderr-drain-on-Disconnected pattern at `tests/wat_arc170_program_contracts.rs:308-323`).

### 7. Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release -p wat --test wat_process_peer_ipc_round_trip
cargo test --release --workspace --no-fail-fast
```

All 3 tests in the renamed file must pass. Workspace baseline: failures ≤ baseline (3 stable + lifeline flake variance). Net: -1 file (deleted old test) + +1 file (new test) + 0 substrate additions = additive on net.

### 8. STONES.md update — tick Stone C2 `[x]`

Edit `docs/arc/2026/05/170-program-entry-points/BRACKET-IMPLEMENTATION-STONES.md`:
- § Stones, Stone C2 subsection: flip the unchecked items to `[x]` (substantive verification this revision provides)
- § Status: replace the `[~]` Stone C2 PARTIAL line with `[x] Stone C2 — ProcessPeer<I, O> + 2 verbs + real-spawn integration test (2026-05-16 post-revision, substrate-composition proof; user-facing surface is Stone D's run-processes bracket)`

## STOP triggers (true emergencies — surface, do not paper over)

1. **`ProcessPeer/new` auto-gen fails to type-check the wat-level call** — surface as substrate gap; do NOT mint a workaround constructor verb. STOP, surface with the exact error.
2. **`Sender/from-pipe` / `Receiver/from-pipe` typing breaks when fed into ProcessPeer/new** — surface as substrate gap. STOP.
3. **Workspace baseline regresses** (failures > baseline) — STOP, surface the new failure.
4. **Any urge to mint a constructor verb** — that's the rejected reflex (`feedback_no_new_types`). STOP, surface what you observed.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** the harness may launch you into `.claude/worktrees/agent-<id>/`. IGNORE that path. Your FIRST action: `cd /home/watmin/work/holon/wat-rs/` (verify with `pwd`). All file edits, builds, tests, and git operations happen against `/home/watmin/work/holon/wat-rs/` — the real repo. The harness worktree directory just sits there as scaffolding; do NOT operate on it, do NOT investigate it, do NOT add files to it. Use absolute paths or `cd` to the real repo at session start; if you ever see `.claude/worktrees/` in a path output, re-cd to the anchor. (Doctrine: `feedback_no_worktrees` + `docs/COMPACTION-AMNESIA-RECOVERY.md` § Failure mode 7-bis — but the prescription there is "ignore harness worktrees and operate at the anchor," NOT "refuse to start.")
- DO NOT mint any new substrate verb, type, or struct (`feedback_no_new_types`).
- DO NOT modify the substrate Stone C2 implementation (`src/types.rs`, `src/check.rs`, `src/runtime.rs` ProcessPeer code). It works; the verbs dispatch correctly via the existing mechanism.
- DO NOT modify Stone C1's ThreadPeer or `make_thread_peer_pair_for_test` (ThreadPeer is symmetric; its Rust-side mock is honest for in-process channels).
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / past STONE BRIEFs/EXPECTATIONS/SCOREs other than ticking Stone C2 in STONES.md per § 8 + the SCORE-STONE-C2-PROCESSPEER.md (orchestrator already revised — do NOT re-edit).
- DO NOT modify `INTERSTITIAL-REALIZATIONS.md` (orchestrator already inscribed the revision context).
- DO NOT modify `SCORE-STONE-C2-PROCESSPEER.md` (orchestrator already revised; SCORE captures "Revision (post-direction-(b))" section).
- DO NOT touch arc 117/133 sibling-binding walker — Stone G's concern.
- DO NOT touch arc 198 artifacts.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.
- DO NOT write new INSCRIPTION or USER-GUIDE content (Stone H handles those).

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `make_process_peer_for_test` retired from `src/typed_channel.rs`; no remaining callers | grep returns 0 hits in src/ and tests/ |
| B | Test file renamed to `tests/wat_process_peer_ipc_round_trip.rs`; concept-anchored | file exists at new path; old path absent |
| C | T2 rewritten as real-spawn round-trip; T1 + T3 unchanged in behavior | grep shows `spawn-process` + `ProcessPeer/new` + `Process/println` + `Process/readln` + `Process/drain-and-join` in test source; T1+T3 logic intact |
| D | 3 new tests pass | `cargo test --release -p wat --test wat_process_peer_ipc_round_trip` → 3 passed / 0 failed |
| E | Workspace test failure count ≤ baseline | full workspace cargo test failures ≤ baseline + flake variance; no new failures attributable to this revision |

## Honest deltas to capture in SCORE

- Did `ProcessPeer/new` auto-gen + parametric inference work cleanly from wat? Or did it surface a substrate gap requiring orchestrator-side handling?
- Did `Sender/from-pipe` / `Receiver/from-pipe` typing flow cleanly into ProcessPeer/new? Any unification surprises?
- Was the variable naming (client/server) clean in the test code, or did the absence of explicit role-types cause confusion?
- Were there subprocess lifecycle gotchas (drain-and-join ordering, EOF semantics) that surfaced?
- Were there test-infrastructure quirks (eval blocking on subprocess deadlock; stderr drain pattern) needed to make T2 diagnostic?

Append to `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-C2-REVISION.md` (NEW file — distinct from SCORE-STONE-C2-PROCESSPEER.md which is the orchestrator-written PARTIAL SCORE for the original mock-driven Stone C2).

## Time-box

60-90 min predicted. The work is bounded but exploratory:
- Helper retirement + rename: 15 min
- T2 rewrite: 30-45 min (first wat-level caller of parametric struct /new; possibly substrate-discovery surprises)
- Build/test cycles + workspace verification: 15-30 min

Hard stop: 120 min. If you hit a true STOP trigger, halt and write what you observed.

## Workspace baseline (commit `ecaf1c0`)

Pre-Stone-C2-revision baseline:
- `cargo build --release --workspace --tests`: clean (with original Stone C2 in place)
- `cargo test --release --workspace --no-fail-fast`: 3 stable failures (`probe_lifeline_pipe_proof`, `wat::test`, `wat_arc170_program_contracts`, `wat-cli`) — wait, that's 4 targets. The actual stable failures per the original Stone C2 SCORE: `lifeline flake + t6 unquote + totally_bogus + startup_error`.

Post-revision target:
- Workspace failures ≤ 4 (or whatever the actual pre-revision baseline is); revision contributes 0 new failures.
- New test target `wat_process_peer_ipc_round_trip`: 3 passed.

## On completion

1. Write `docs/arc/2026/05/170-program-entry-points/SCORE-STONE-C2-REVISION.md` per § SCORE methodology + § Honest deltas.
2. Surface final summary: rows passed/failed + workspace delta + path to SCORE.

You are launching now. T-minus 0.
