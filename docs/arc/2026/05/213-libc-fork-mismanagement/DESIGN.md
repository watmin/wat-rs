# Arc 213 — libc::fork mismanagement under workspace pressure

**Status:** OPEN 2026-05-18 — opened to address `probe_lifeline_pipe_proof`'s pressure-flake whose intermittent nature was documented by arc 211c's audit. Arc 211 closure depends on this arc's resolution per the **tooling-proven-by-use** discipline (see INTERSTITIAL § 2026-05-18 (post-arc-211e)).

**Priority:** BLOCKING arc 211 INSCRIPTION (along with arc 212).

## Origin

`probe_lifeline_pipe_proof` (created in arc 170 Slice D / FD-multiplex Phase work; commit `198c30b`) demonstrates deterministic parent-death detection via lifeline pipe. The test's doc-comment claims:

> *"100/100 trials produce zero orphans regardless of supervisor exit timing."*

In isolation: yes, the test passes 100/100. Verified multiple times this session.

**Under workspace parallel test pressure:** the test flakes. Failure SET rotates between `probe_lifeline_pipe_proof` and `test` umbrella's deftest-hermetic subtests (per arc 211c audit + post-211e workspace runs). Never both simultaneously fail. Never reliably reproduces — pure pressure-flake.

The mechanism uses raw `libc::fork` directly with manual pipe management — NO substrate spawn-process / spawn-thread involvement. The flake suggests OS-level mismanagement: fd-table pressure, scheduling timing, pipe inheritance race under heavy parallel load.

## Scope

**In scope:**
- Investigate the flake's root cause under workspace pressure
- Determine: is it a fixable bug, or a fundamental OS-pressure characteristic of the test mechanism?
- Either:
  - **Fix:** identify + repair the libc::fork-management issue; test passes 100% under workspace pressure
  - **Document:** SCORE inscribes honest assessment ("OS-pressure characteristic; mechanism correct in isolation; expected-intermittent under parallel pressure"); test marked `#[ignore]` or test runner config excludes it from workspace failure counts
- Use arc 211's panic-as-EDN tooling to capture the failure structurally if it surfaces a panic
- If the failure is pure-hang (no panic emitted), document that arc 211a/b's tooling didn't directly help — and inscribe what tooling WOULD have helped (informs future arc work)

**Out of scope:**
- Broader libc::fork patterns elsewhere in substrate (unless investigation reveals shared root cause)
- The lifeline-pipe mechanism itself (proven correct in isolation; not redesigning)
- FD-multiplex Phase 6 paperwork (separate task #305; may interact but not part of this arc)

## Closure conditions

1. Investigation produces honest diagnosis (rooted in actual run-data, not speculation per `feedback_no_speculation`)
2. EITHER:
   - Fix ships AND probe_lifeline_pipe_proof passes under workspace pressure (≥100 trials clean)
   - OR honest "expected-intermittent" assessment ships AND test excluded from workspace failure count
3. SCORE doc inscribes findings (including: did arc 211's tooling assist? what gaps remain?)
4. Arc 211 closure becomes unblocked (other pre-condition: arc 212)

## Cross-references

- Arc 170 FD-multiplex Phase 1A-3 (the lifeline mechanism work that produced this test)
- Arc 170 Phase 1D SCORE (substrate-mechanism probe + leak-zero gate — this test IS the gate)
- Arc 211 SCORE-211C-AUDIT (confirmed pressure-flake nature)
- Arc 211 DESIGN § "Tooling-proven-by-use closure condition" (the blocking relationship)
- Arc 211 INSCRIPTION (pending; awaits this arc)
- INTERSTITIAL § 2026-05-17 "Orphan-process leak investigation" (broader FD-management concerns; shared diagnostic territory)
- INTERSTITIAL § 2026-05-18 (post-arc-211e) "Tooling proven by use — closure-discipline extension"
- `tests/probe_lifeline_pipe_proof.rs` (the test)

## Tooling-proven-by-use principle

This arc serves dual purpose:
1. **Resolve probe_lifeline_pipe_proof's disposition** (substrate correctness OR honest documentation)
2. **PROVE arc 211's tooling enabled this resolution** (substrate-tooling-validation)

Two possible validation paths:
- **If failure surfaces a panic** — arc 211b's structured EDN provides readable diagnostic; arc 211a's ctor ensures the hook is installed. Tooling proves itself directly.
- **If failure is a pure-hang (no panic)** — arc 211's tooling didn't directly help. SCORE inscribes the gap. That inscription IS load-bearing for future tooling arcs (we'd know what arc 211 didn't cover and could open follow-up tooling work).

Either outcome validates the principle: tooling-proven-by-use, not tooling-assumed-working. When arc 213 closes, arc 211 closes (along with arc 212).

---

## Scope EXPANDED 2026-05-18 (post-orphan-leak-discovery + Linux-5.3-primitives commitment)

The "pressure-flake" framing above was incomplete. Investigation 2026-05-18 (following arc 212's δ-comm-purge cascade methodology applied to the orphan-leak question) revealed two distinct failures + identified the substrate-honest goal-state.

User direction 2026-05-18: *"we are linux first - we leverage the best of breed at all times - what is the correct longterm syscall pattern - we are approaching the goal"* + *"my os is linux 6 ... 5.3 is from 2019 - we use the tools we have - zero doubt - do it perfect"*.

### Two distinct failures (not one)

**Failure 1 — substrate non-compliance (real production gap):**
- `src/fork.rs:153 — run_in_fork` is a substrate fork primitive that BYPASSES the lifeline mechanism entirely. No `prctl(PDEATHSIG)`. No lifeline pipe. Children orphan if parent dies before `waitpid` completes.
- 9 callers across substrate + tests: 5 in `src/runtime.rs` lib-tests, 3 in `tests/wat_harness_deps.rs`, 1 each in `probe_shutdown_cascade_crossbeam.rs` + `probe_shutdown_cascade_pipefd.rs` + `wat-cli/tests/wat_cli.rs`
- Production orphan evidence observed 2026-05-18 — PIDs 169036 (PPid=1, "wat-test:::wat-") + 169054 (PPid=169036) survived `cargo test --release --workspace` completion. The supervisor was a `run_in_fork`-spawned child; its grandchild (spawned via `spawn-process`, which DOES install lifeline) couldn't die because its lifeline_w was still held by the un-dyingable supervisor.
- The substrate's "every spawn has a lifeline" guarantee is a LIE — `run_in_fork` violates it.

**Failure 2 — probe observation cheat (separate, /proc-based):**
- 5 probe files read `/proc/PID/stat` to observe child process state instead of using kernel-direct syscalls.
- The 1/100 `probe_lifeline_pipe_proof` flake is from the probe's `/proc/PID/stat` read racing the kernel's process-state-publication window — NOT from the lifeline mechanism failing. The lifeline mechanism (in the probe) is structurally sound.
- The substrate-honest oracle for "is this process dead" is `waitid(P_PIDFD, pidfd, WEXITED)` or `poll(pidfd, POLLIN)`. `/proc` is a fuzzy publication layer; the kernel knows precisely.

### The substrate's goal-state Linux 5.3+ process primitives

We are Linux-first per `feedback_no_windows`. We are on Linux 6+. Linux 5.3 (Sep 2019) is the floor — every primitive below has been kernel-stable for 5+ years. We use the kernel's strongest guarantees, not legacy POSIX.

**Canonical substrate fork-and-observe protocol:**

```
1. PRE-FORK SETUP (parent):
   - Create lifeline pipe (parent holds write_end, child inherits read_end via fork)
   - Create stdio pipes (3× for stdin/stdout/stderr)
   - Build clone3 args with:
       CLONE_PIDFD          — atomic pidfd at fork time (no PID-reuse race)
       CLONE_CLEAR_SIGHAND  — clean signal-handler state in child (no parent-thread inheritance)
       (optional: CLONE_INTO_CGROUP for resource control; not load-bearing for arc 213)

2. CREATION:
   clone3() → returns (pid, pidfd) atomically
     - Child inherits all fds including lifeline_r (atomic with fork)
     - pidfd is bound to THIS specific child; PID-reuse race eliminated

3. CHILD POST-FORK SETUP:
   - setpgid(0, 0)  — child becomes its own process group leader (cascade target)
   - dup2 stdio pipes onto fd 0/1/2
   - Drop lifeline_w copy (child only needs read end; only parent should hold write)
   - Execute child program; lifeline_r inherited as substrate-runtime-known fd

4. PARENT OBSERVATION (event-driven, kernel-direct):
   - poll(pidfd, POLLIN)                       — exit-event notification
   - waitid(P_PIDFD, pidfd, WEXITED|WNOWAIT)  — peek exit status without reaping
   - waitid(P_PIDFD, pidfd, WEXITED)          — atomic exit-status read + reap
   - pidfd_send_signal(pidfd, sig)            — signal THE child (PID-reuse-safe)
   
   NO /proc reads. NO PID-reuse race windows. Pure kernel-event-driven.

5. CHILD PARENT-DEATH DETECTION:
   - poll(lifeline_r, POLLIN)  — POLLHUP fires when parent closes write_end
   - OR read(lifeline_r)       — returns 0 (EOF) when last write_end closes
   Unrace-able: lifeline_r was inherited ATOMICALLY with clone3; no post-fork registration needed.

6. PROCESS-GROUP CASCADE:
   - killpg(child_pgid, sig) — cascade to entire child subprocess tree
   - Existing substrate doctrine per `project_signal_cascade`; integrates with above

7. SIGNAL HANDLING (when integrating signal-as-event):
   - signalfd() — convert signals to fd-readable events
   - Integrate with the FD-multiplex poll loop
   - NO async signal handlers in substrate code (signal-safety trap rejected)
```

### Current → goal-state mapping

| Current primitive | Goal primitive | Why goal is structurally stronger |
|---|---|---|
| `libc::fork()` | `clone3() + CLONE_PIDFD + CLONE_CLEAR_SIGHAND` | Atomic pidfd binding; clean child signal state; no inherited handlers |
| `waitpid(pid, ...)` | `waitid(P_PIDFD, pidfd, WEXITED)` | PID-reuse-safe; richer status info; can peek-without-reap via WNOWAIT |
| `libc::kill(pid, sig)` | `pidfd_send_signal(pidfd, sig)` | Signals THE child specifically — not "whoever holds that PID now" |
| `/proc/PID/stat` reads | `poll(pidfd, POLLIN)` / `waitid(P_PIDFD, ..., WNOHANG)` | Kernel-event-driven; no procfs publication-lag window |
| Async signal handlers | `signalfd()` integrated with FD-multiplex | Normal-context handling; eliminates signal-safety restrictions |
| `pidfd_open(pid)` | NEVER USE | Race window between PID lookup + binding; only clone3-returned pidfd is race-free |
| Manual `prctl(PR_SET_PDEATHSIG)` | Lifeline pipe inherited atomically via clone3 | No post-fork install-race; setup is pre-fork; inheritance is atomic |

### L2 enforcement (substrate-imposed-not-followed for fork primitives)

Same shape as `WatAST::children()` newtype wall for arc 212's L4 endgame. Module privacy + canonical helpers:

- `libc::fork`, `libc::clone3`, `libc::waitpid`, `libc::waitid`, `libc::kill`, `libc::pidfd_open`, `libc::pidfd_send_signal`, `libc::signalfd` — ALL inaccessible to consumers (module-private in `wat::fork` or `wat::os`)
- ONE canonical fork helper: `wat::fork::spawn_lifelined(args) -> (Pid, Pidfd, LifelineWriter)`
- ONE canonical observe helper: `Pidfd::{poll_exit, wait_status, send_signal}`
- The `Pidfd` type has NO `from_pid` constructor — it can ONLY come from `spawn_lifelined`'s return value
- Substrate refuses any path that:
  - Calls `libc::fork()` outside the canonical helper (compile error via module privacy)
  - Constructs a `Pidfd` from PID alone (no public constructor; typestate equivalent)
  - Uses raw `kill(pid)` or `waitpid(pid)` outside the migration window (compile error)

Wrong shape becomes structurally impossible. "Fork without lifeline" cannot be expressed. "Signal a PID-reused process" cannot be expressed. "Observe process state via fuzzy oracle" cannot be expressed.

### Stone chain (L0 → L4 trajectory for arc 213)

| Stone | Layer | What | Effect |
|---|---|---|---|
| **α** | L0 substrate | Mint canonical `Pidfd` type + `spawn_lifelined` helper using `clone3 + CLONE_PIDFD + CLONE_CLEAR_SIGHAND`; install lifeline pipe + setpgid; the foundation primitive | Foundation primitive exists |
| **β** | L1 migration | Migrate `run_in_fork` (the immediate production-orphan gap) to use `spawn_lifelined` | Production orphan leak eliminated |
| **γ** | L1 migration | Migrate the 3 existing `libc::fork()` sites (fork.rs:153/614/920) to `spawn_lifelined` | All substrate fork paths use canonical helper |
| **δ** | L1 migration | Migrate all `waitpid(pid)` + `kill(pid)` sites to `Pidfd::wait_status` / `Pidfd::send_signal` | All process operations PID-reuse-safe |
| **ε** | L1 migration | Probes switch `/proc/PID/stat` → `Pidfd::poll_exit` / `Pidfd::wait_status(WNOHANG)` | Kernel-direct observation; no fuzzy oracle |
| **ζ** | **L2 enforcement** | `libc::fork`/`clone3`/`waitpid`/`waitid`/`kill`/`pidfd_*` become module-private; only canonical helpers public; compile-time refusal of any direct libc::* path | "Fork without lifeline" / "kill by PID alone" / "observe via /proc" structurally impossible forever |
| **η** | INSCRIPTION | Doctrine inscribed: "wat-rs uses Linux 5.3+ process primitives canonically; legacy POSIX is migration scaffolding, never new code"; arc 213 closes; arc 211 closure unblocks (one of two pre-conditions; arc 212 the other) | Arc 213 closed; substrate process-management doctrine etched |

### Closure conditions (REPLACES the original closure conditions above)

1. Canonical `Pidfd` + `spawn_lifelined` primitive shipped (α)
2. All `libc::fork()` callers migrated to canonical helper (β + γ); production orphan-leak class eliminated
3. All `waitpid` / `kill` migrated to `Pidfd` methods (δ); PID-reuse race class eliminated
4. All probes use kernel-direct observation (ε); /proc as oracle eliminated from test infrastructure
5. `libc::*` process primitives are module-private; only canonical helpers public (ζ); wrong shape structurally impossible
6. `probe_lifeline_pipe_proof` passes 100/100 trials reliably (the fuzzy /proc observation that caused the 1/100 flake is GONE; mechanism + observation both substrate-honest)
7. INSCRIPTION ships (η); arc 213 closed; arc 211 closure unblocked

### Doctrine inscription (lands in INSCRIPTION at η)

> **wat-rs uses Linux 5.3+ process primitives canonically.**
> 
> The substrate's fork-and-observe protocol uses `clone3() + CLONE_PIDFD` for atomic process creation, `waitid(P_PIDFD, ...)` for race-free exit observation, `pidfd_send_signal()` for race-free signaling, the lifeline-pipe (inherited atomically via clone3) for parent-death detection, and `signalfd()` for any signal-as-event integration. `/proc/PID/stat` is never an oracle in our system except for the documented `/proc/self/fd` enumeration case (no syscall equivalent exists).
> 
> Legacy POSIX primitives (`fork`, `waitpid`, `kill`, `pidfd_open(pid)`, async signal handlers) are migration scaffolding. New substrate code uses the modern primitives. Existing code migrates as it touches each site.
> 
> Per `feedback_no_windows`: *"if others want to run wat on their os - they need to make their os not suck ass."* We use Linux's best primitives. Other OSes don't have equivalents; we're not portable; we're correct.

### Per-failure-engineering doctrine

| FE component | Application |
|---|---|
| 1. Failure is data | The orphans we observed + the 1/100 probe flake are TWO distinct substrate-gap data, surfaced via δ-comm-purge methodology applied to arc 213 |
| 2. Stop immediately | Halted speculative "race condition possible on Linux" framing twice; recognized the substrate is using wrong oracles + bypassing lifeline mechanism |
| 3. Eliminate the CLASS | L2 enforcement at ζ — every wrong shape structurally impossible. Orphan-leak class extinct. PID-reuse race class extinct. Fuzzy-oracle class extinct. Forever. |

### Cross-references (updated)

- This DESIGN's original "Origin" + "Scope" sections preserved above as historical record (the pre-discovery framing)
- INTERSTITIAL § 2026-05-18 (mid-cascade) "PURGE" — the methodology that surfaced these substrate gaps
- INTERSTITIAL § 2026-05-18 (Linux-5.3-commitment) — the doctrine moment this expansion lands (entry to be inscribed after this commit)
- Arc 212 DESIGN § "Scope EXPANDED 2026-05-18 (post-L4-conversation)" — sibling pattern (L2 substrate enforcement for WatAST::children())
- `feedback_any_defect_catastrophic` — the doctrine that drives immediate pivot
- `feedback_refuse_easy_solutions` — the doctrine that rejected "race condition possible on Linux" framing
- `feedback_no_windows` — the Linux-first commitment that unlocks using these primitives without portability layers
- `project_signal_cascade` — the existing process-group cascade discipline; integrates with goal-state protocol
- `docs/ZERO-MUTEX.md` — the substrate's broader "structurally-impossible-wrong-shape" doctrine
