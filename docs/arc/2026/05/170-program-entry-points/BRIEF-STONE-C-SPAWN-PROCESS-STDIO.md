# Arc 170 RUNTIME-BOOTSTRAP-BACKLOG Stone C BRIEF — spawn-process stdio reshape

**Sonnet.** Substrate reshape + wat-level wrappers + consumer sweep. **Closes the original "fatal flaw" — spawn-process children get full ambient stdio.** Workspace ends green.

User direction 2026-05-15:
> *"why can we not mask Sender<T>, Receiver<T> on top of the pipes? if its bidirectional EDN we are covered?"*

Four-questions verdict: option (a-layered) — substrate stdio only at OS boundary; wat-level Sender/Receiver wrappers OVER the pipes preserve TIERS.md uniformity.

## Why this stone exists

Today (post-arc-170-slice-1c): `spawn_process_child_branch` only dup2s fd 2 (stderr). Child's fd 0 (stdin) and fd 1 (stdout) inherit from parent — wherever parent's terminal points. `Process` struct fields `stdin`/`stdout`/`stderr` lie about what they are (typed-channel pipes named as stdio). Child's `println` errors with `ServiceNotRunning` (no trio services because no bootstrap call).

Stone A shipped (`92926a2`): `bootstrap_wat_vm_process` exists as the canonical substrate-owned bootstrap.

Stone C does the consumer-side of A: spawn-process delegates to A + dup2s fd 0/1 to real pipes + reshapes Process struct fields to real stdio + retires the typed-channel-at-process-boundary surface (which was slice-1c's wrong turn). Wat-level Sender/Receiver wrappers preserve the typed-channel mental model where users want it.

## Required reading IN ORDER

1. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/170-program-entry-points/RUNTIME-BOOTSTRAP-BACKLOG.md` — the 7-stone backlog this is Stone C of (Stone A already shipped)
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/170-program-entry-points/SCORE-STONE-A-EXTRACT-BOOTSTRAP.md` — Stone A's shape; `bootstrap_wat_vm_process` is the helper Stone C calls
3. `/home/watmin/work/holon/wat-rs/src/spawn_process.rs` — `spawn_process_child_branch` (line ~279+) is the call site to refactor
4. `/home/watmin/work/holon/wat-rs/src/freeze.rs` — `bootstrap_wat_vm_process` (the helper you call) + `BootstrapArgs` + `ProcessRuntime`
5. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — search for `process_send` / `process_recv` / `Process/tx` / `Process/rx` / `eval_kernel_process_stdout` / `eval_kernel_process_stderr` / `eval_kernel_process_stdin` — the substrate dispatch sites for the Process surface
6. `/home/watmin/work/holon/wat-rs/wat/kernel/queue.wat` (or wherever `Sender<T>` / `Receiver<T>` type aliases live) — the existing typed-channel types you'll wrap
7. `/home/watmin/work/holon/wat-rs/wat/test.wat` — `run-hermetic-with-io-driver` (line ~725+, Layer 2) — uses the typed channels today; needs migration
8. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md` — Pattern 2 verb retirement discipline (substrate emits teacher diagnostic; consumer sweep follows)
9. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/170-program-entry-points/TIERS.md` — currently claims `Sender<T>`/`Receiver<T>` at every tier; needs an amendment row noting tier-2's wat-level wrapper implementation

## What ships

### Substrate (src/)

**1. `spawn_process_child_branch` (src/spawn_process.rs:279+):**
- Open three OS pipes per child (today: only stderr_pair). Add `(stdin_r, stdin_w)` + `(stdout_r, stdout_w)` to mirror the existing stderr pair.
- After fork in child: dup2 stdin_r → fd 0, stdout_w → fd 1, stderr_w → fd 2 (today: only fd 2)
- Delegate to `bootstrap_wat_vm_process(BootstrapArgs { frozen: &world })` after `startup_from_forms` returns the world
- Use `runtime.symbols()` for `apply_function` (instead of `world.symbols()`)
- `ProcessRuntime::drop` handles cleanup via Stone A's Drop impl

**2. `Process` struct reshape (src/spawn_process.rs:219+):**
- Today's fields: `stdin: IOWriter`, `stdout: IOReader`, `stderr: IOReader`, `ProgramHandle`, `tx: Sender<I>`, `rx: Receiver<O>` (6 fields)
- After Stone C: `stdin: IOWriter`, `stdout: IOReader`, `stderr: IOReader`, `ProgramHandle` (4 fields). REMOVE `tx` and `rx` fields.
- The `stdin` field becomes the parent's WRITE end of the child's stdin pipe (parent writes; child reads via fd 0)
- The `stdout` field becomes the parent's READ end of the child's stdout pipe (child writes via fd 1; parent reads)
- The `stderr` field is unchanged (parent's READ end of child's stderr; existing)
- The struct's `type_name` stays `:wat::kernel::Process` (no rename needed since field shape changes, not name)

**3. Retire substrate accessors for typed-channel fields:**
- `eval_kernel_process_send` / `eval_kernel_process_recv` (whatever they're named) — replace dispatcher arm with substrate-as-teacher Pattern 2 synthetic `CheckError::TypeMismatch`:
  - callee: `:wat::kernel::process-send` / `:wat::kernel::process-recv`
  - expected: shape hint to use `Process/stdin` + `Sender/from-pipe` wrapper
  - got: `(retired verb)`
- Similar for `Process/tx`/`Process/rx` field accessors if they exist
- `Process/stdin` / `Process/stdout` / `Process/stderr` accessors stay (they now return the real pipes from Stone C's reshape — the load-bearing fix)

**4. `arc_170_stone_c_typed_channel_at_process_boundary_retire_hint` in src/check.rs::collect_hints:**
- Detects retired callee in `callee` field
- Hint text names: *"`:wat::kernel::Process` typed-channel API retired (arc 170 Stone C). Real stdio is canonical at OS boundary. Wrap pipes with `:wat::kernel::Sender/from-pipe` (over `Process/stdin`) or `:wat::kernel::Receiver/from-pipe` (over `Process/stdout`) for typed semantics — wat-level wrapper over EDN-over-pipes."*

### Wat-level (wat/)

**5. Mint `:wat::kernel::Sender/from-pipe` + `:wat::kernel::Receiver/from-pipe` wrappers:**

Likely in `wat/kernel/queue.wat` (where Sender/Receiver type aliases live today) or a new `wat/kernel/pipe-channel.wat`:

```scheme
;; Wrap an IOWriter (real OS pipe write end) as a Sender<T>.
;; Sending a value EDN-encodes + writes the line; recipient reads + decodes.
(:wat::core::defn :wat::kernel::Sender/from-pipe<T>
  [writer <- :wat::io::IOWriter]
  -> :wat::kernel::Sender<T>
  ...)

;; Wrap an IOReader (real OS pipe read end) as a Receiver<T>.
(:wat::core::defn :wat::kernel::Receiver/from-pipe<T>
  [reader <- :wat::io::IOReader]
  -> :wat::kernel::Receiver<T>
  ...)
```

The internal implementation: each `send` EDN-encodes via wat-edn + writes a line; each `recv` reads a line + decodes. If the existing Sender/Receiver substrate types are crossbeam-backed only, you may need a new wrapper TYPE (e.g., `:wat::kernel::IoSender<T>` / `:wat::kernel::IoReceiver<T>`) — OR a dispatch via arc 146's machinery (probably future arc; pick the simpler path for Stone C: distinct types with `from-pipe` constructor naming). Document the choice in honest delta.

### Consumer sweep (wat/, wat-tests/)

**6. `run-hermetic-with-io-driver` (wat/test.wat:725+):**
- Today: probably uses `process-send` / `process-recv` on the Process struct
- After: drains `Process/stdout` lines as Vec<String>; reconstructs each as wat values via EDN parse (or migrates to use Sender/Receiver wrappers explicitly)

**7. `:user::process` contract sites:**
- Today: `[rx <- Receiver<I> tx <- Sender<O>] -> :nil` (per TIERS.md)
- After: `[] -> :nil` or `[stdin <- IOReader stdout <- IOWriter] -> :nil` depending on whether users want raw pipes (canonical) or the wrappers (opt-in)
- Match canonical pattern: zero-arg + users reach for `(:wat::runtime::stdin)` ambient + `(:wat::kernel::println v)` (per CIRCUIT.md)

**8. Test/example consumers** — grep for any caller of `process-send` / `process-recv` / `Process/tx` / `Process/rx` and migrate.

### Docs

**9. TIERS.md amendment** — add row clarifying that tier-2 (`spawn-process`) `Sender<T>`/`Receiver<T>` is a wat-level WRAPPER over EDN-over-pipes, not a substrate-built-in type at that tier. The user-API uniformity claim holds; implementation differs per tier.

## Path-honesty discipline (carries forward from Gap K / slice 1i)

Every probe body MUST exercise the SAME surface its file NAME identifies. No silent path-switching. If a property can't be verified on the named path, declare OUT-OF-SCOPE in the SCORE; do not switch paths to make a test pass.

## Verification

Empirical probes:
- `tests/probe_spawn_process_stdio.rs` — spawn-process child body calls `(:wat::kernel::println "hello")`; parent reads via `Process/stdout`; assert "hello" in the lines
- `tests/probe_spawn_process_stdin.rs` — parent writes to `Process/stdin`; child reads via `(:wat::kernel::readln)`; assert child got the value
- `tests/probe_spawn_process_no_typed_channels.rs` — Process struct has 4 fields (stdin/stdout/stderr + handle); typed-channel field access (process-send, process-recv) surfaces structured-stderr-only contract error via substrate-as-teacher (or compile-time check error if Pattern 3)
- `tests/probe_sender_receiver_from_pipe.rs` — wat-level wrappers: send via Sender/from-pipe wraps EDN encoding; recv via Receiver/from-pipe decodes; round-trip works

Detection counts:
- `ProcessJoinBeforeOutputDrain`: 0 (still)
- new substrate-as-teacher hint (`arc_170_stone_c_typed_channel_at_process_boundary_retire_hint`): 0 after consumer sweep completes

Workspace: ends green. Pre-Stone-C: 167 pass / 7 fail. Post-Stone-C: workspace count may differ (consumer migration adds new patterns); the OLD 7 Pattern A/C failures may resolve (some used typed channels) or persist (Pattern A/C orthogonal). Surface honestly in SCORE.

## Hard constraints (corrected pattern per memory `feedback_brief_constraint_contradictions`)

- DO NOT modify pre-existing arc artifacts (DESIGN.md, INSCRIPTION.md, BRIEF-*.md, EXPECTATIONS-*.md, REALIZATIONS-*.md) other than:
  - Creating the SCORE-STONE-C-SPAWN-PROCESS-STDIO.md deliverable specified below
  - Adding a TIERS.md amendment row (TIERS.md is the architecture concept doc; this slice changes the architecture so the doc updates here)
- DO NOT modify `src/check.rs::ProcessJoinBeforeOutputDrain` (the detection from Gap K stays; you'll add a NEW hint in `collect_hints` for the Pattern 2 retirement)
- DO NOT add wall-clock timeouts ANYWHERE (no `set_*_timeout`, no `std::thread::sleep`, no arbitrary numbers)
- DO NOT touch deftest macro (separate concern)
- DO NOT touch `~/.claude/` memory system
- DO NOT use `cd <subdir> && ...` — use absolute paths or `git -C <repo>` (FM 7)
- DO NOT commit / push / git add — orchestrator atomic-commits after scoring
- DO NOT use `timeout 600` or any > 120s wrapper
- DO NOT name a probe file in a way that doesn't match what its bodies test (Row G discipline)
- DO use `timeout -k 5 N` on every cargo invocation; N=30 probe, N=90 workspace
- DO use `pkill -9 -f "target/release/deps/test-"` if orphans appear; report in SCORE

## Mode B trigger

- If the substrate refactor reveals that `Process` struct's `tx`/`rx` fields are deeply embedded in test infrastructure that can't easily migrate: STOP and report — we may need to split this stone or stage migration
- If `Sender<T>` / `Receiver<T>` substrate types can't accept `from-pipe` constructors without breaking the crossbeam-based tier-1 implementation: STOP and report — separate dispatch decision per honest delta #3
- If workspace post-Stone-C falls below pre-Stone-C pass count (167) by more than the typed-channel-consumer count: STOP and report — unexpected regression
- If the consumer sweep can't fully migrate `run-hermetic-with-io-driver` (e.g., the wrapper API has gaps): STOP and report — fix the wrapper, then continue

## Ship criteria (10 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `spawn_process_child_branch` opens 3 OS pipes + dup2s fd 0/1/2 (today only fd 2) | grep + read |
| B | `spawn_process_child_branch` calls `bootstrap_wat_vm_process` after `startup_from_forms` (delegate to Stone A's helper) | grep + read |
| C | `Process` struct has 4 fields: stdin (IOWriter), stdout (IOReader), stderr (IOReader), ProgramHandle. NO tx, NO rx | grep + read |
| D | `process-send` / `process-recv` substrate dispatch retired; Pattern 2 teacher (`arc_170_stone_c_...`) emits structured TypeMismatch with migration hint pointing at `Sender/from-pipe` / `Receiver/from-pipe` | grep + read; probe exercises retired callee + sees the hint |
| E | Wat-level `:wat::kernel::Sender/from-pipe` + `:wat::kernel::Receiver/from-pipe` wrappers minted; round-trip EDN encoding works | cargo test probe |
| F | `probe_spawn_process_stdio` PASSES — child println, parent captures via Process/stdout | cargo test |
| G | `probe_spawn_process_stdin` PASSES — parent writes Process/stdin, child reads via readln | cargo test |
| H | `probe_sender_receiver_from_pipe` PASSES — wrapper round-trip | cargo test |
| I | Consumer sweep complete: `run-hermetic-with-io-driver` migrated; `:user::process` contract sites updated; new pattern hint fires 0 times in workspace tests | grep cargo output |
| J | Workspace ends green (count may shift due to consumer migration; honestly accounted in SCORE) | full workspace |

**10 rows. All must PASS.**

## Scope (what's IN)

- Substrate: spawn_process.rs refactor + Process struct reshape + retire process-send/recv dispatch + mint Pattern 2 teacher hint in check.rs::collect_hints
- Wat-level: Sender/from-pipe + Receiver/from-pipe wrappers (likely in wat/kernel/queue.wat or new wat/kernel/pipe-channel.wat)
- Consumers: run-hermetic-with-io-driver migration + :user::process contract updates + any other process-send/recv callers
- Probes: 4 new path-honest probes (spawn_process_stdio, spawn_process_stdin, no_typed_channels, sender_receiver_from_pipe)
- TIERS.md amendment row clarifying tier-2 Sender/Receiver is wat-level wrapper

## Scope (what's OUT)

- Stone D (spawn-thread thread-init) — separate stone
- Stone B (wat-cli shim) — separate stone
- Stones E/F/G (apply_function context check + Pattern 3 CheckError for libc::fork/std::thread::spawn outside canonical site + final docs) — separate stones
- ScopeDeadlock walker / ProcessJoinBeforeOutputDrain detection changes — out
- spawn-thread changes — out
- fork-program-ast changes — out (it's being retired in SPAWN-MIGRATION-BACKLOG, not Stone C)
- arc 146 dispatch unification of Sender/Receiver tier 1 vs tier 2 — out; pick distinct types or naming convention for Stone C

## Predicted runtime

**120-180 min sonnet.** Substrate Rust + wat-level wrappers + consumer migration + 4 probes + TIERS.md amendment.

**Hard cap:** 300 min (1.7×; this is a bigger stone than Stone A). Wakeup at T+3600s (runtime cap; will check + reschedule if needed).

## Honest deltas (anticipated)

1. **Sender/Receiver type unification** — if existing Sender<T>/Receiver<T> are crossbeam-only, you may need distinct types (e.g., `IoSender<T>` / `IoReceiver<T>`) or wat-side type aliases. Surface choice with rationale.
2. **`:user::process` contract change** — from `[rx tx] -> nil` to `[] -> nil`. This is a USER-VISIBLE API break for spawn-process Layer 3 callers. Document the migration path; sweep affected sites.
3. **Process struct field re-ordering** — likely you'll want stdin/stdout/stderr as fields 0/1/2 (matching fd numbers) + ProgramHandle as field 3. Field-order changes may break callers using positional field access; check the wat-side Process/stdin etc. accessors are name-based (likely they are).
4. **TIERS.md "user-visible IPC" column** — today says `Sender<T>`/`Receiver<T>` for tier 2. Amend: tier 2's Sender/Receiver are wat-level wrappers, NOT substrate-built-in. Update the row accordingly.
5. **`run-hermetic-with-io` test infrastructure** — depending on what it surfaces today, migration may be straightforward (use Sender/from-pipe wrappers) or require restructuring. Surface honestly.

## Cross-references

- `RUNTIME-BOOTSTRAP-BACKLOG.md` Stone C row (amended at this commit's predecessor) — names the (a-layered) scope
- `SCORE-STONE-A-EXTRACT-BOOTSTRAP.md` — Stone A's helper is what you delegate to
- `SUBSTRATE-AS-TEACHER.md` Pattern 2 — the verb-retirement discipline for `process-send`/`process-recv`
- `TIERS.md` — the architecture doc you amend at the end
- `CIRCUIT.md` line 39 — canonical Server/Client pattern via println; this stone makes it work on spawn-process
- Memory: `feedback_substrate_owns_not_callers_match` — the discipline this stone embodies
- Memory: `feedback_brief_constraint_contradictions` — the corrected constraint pattern in this BRIEF

## Deliverable

Write `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/170-program-entry-points/SCORE-STONE-C-SPAWN-PROCESS-STDIO.md` with:
- 10-row scorecard (PASS/FAIL per row)
- Before/after of `spawn_process_child_branch` + `Process` struct shape
- The new wat-level wrappers' signatures + module-home rationale
- The Pattern 2 teacher hint's `expected`/`got` strings + where in `collect_hints` it lives
- 4 new probe filenames + path-honesty audit per probe
- Workspace state pre/post (honest accounting of any test-count shifts)
- Consumer migration list (every file touched in sweep)
- Honest deltas (≥ 4)

Then STOP. Report what shipped + path to SCORE doc + 10-row scorecard summary.

GO.
