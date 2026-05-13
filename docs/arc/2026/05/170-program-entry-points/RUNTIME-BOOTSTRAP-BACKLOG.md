# Arc 170 — Runtime-bootstrap backlog (substrate-owned)

**Created:** 2026-05-15 (post-slice-1i recognition session)
**Blocks:** SPAWN-MIGRATION-BACKLOG.md Steps 4+ (Phase F retire, consumer sweep, destructive reap)
**End state:** ONE substrate function establishes a wat-vm runtime context. `wat-cli` and `spawn-process` are tiny shims that call it. No call site re-implements bootstrap. No future spawn primitive can forget — there's nothing in the spawn primitive to remember; substrate owns it.

## Why this backlog exists

The original "fatal flaw" user named 2026-05-15: `spawn-process` children can't `println` because no ThreadIO + no trio services. We deferred it as "OUT OF SCOPE (Row C3) for Gap K — waits for slice 1F services on spawn-process."

When we tried to draft that fix (the 6-stone services-enforcement sketch), the user broke through to the deeper architecture:

> *"we need it deeper — unless you can convince me otherwise — it feels rediculuous that we need spawn-process and wat-cli to do identical work that the substrate should do on their behalf. wat-cli should be an extremely tiny shim on the vm."*

The right shape: **substrate-owned bootstrap.** Same discipline as ZERO-MUTEX (substrate never constructs the situation that needs Mutex), structured-stderr-only (substrate emits structured, callers don't), one-canonical-path (no synonyms). When N call sites need identical setup, the setup belongs in the substrate; call sites become benefactors.

Memory: `feedback_substrate_owns_not_callers_match` — the cognitive failure that led to the deeper-correction probing.

## The architecture (verified empirically 2026-05-15)

Today's situation:
- `invoke_user_main_orchestrated` (`src/freeze.rs:747-840`) IS the bootstrap. Called by `wat-cli` and fork-program-ast paths. Does: source stdio → spawn trio services → build RuntimeServices carrier → register thread-0 → install ThreadIO → invoke main → cleanup.
- `spawn_process_child_branch` (`src/spawn_process.rs:279+`) reimplements pieces inline. Does NOT call `invoke_user_main_orchestrated`. Does NOT install ThreadIO. Does NOT spawn trio. **Gap.**
- `eval_kernel_spawn_thread` (`src/runtime.rs:16335+`) — thread-internal spawn. Threads share their process's services Arc via SymbolTable clone. Each thread needs its own ThreadIO (thread-local). The current implementation may or may not install ThreadIO for the spawned thread; gap-of-omission needs to verify.
- fork-program-ast is being retired (SPAWN-MIGRATION-BACKLOG Steps 6-7). Its bootstrap path becomes spawn-process's bootstrap path. Substrate-owned helper means the migration moves the call site, not the bootstrap logic.

## The seven stones (substrate-owned bootstrap)

| # | Stone | What ships | Size | Blocks | Blocked by |
|---|---|---|---|---|---|
| **A** | `pub fn bootstrap_wat_vm_process(args: BootstrapArgs) -> ProcessRuntime` in the substrate. ONE function does: source/inherit stdio fds → spawn trio services → build RuntimeServices Arc → install ThreadIO for caller's thread → return augmented SymbolTable + cleanup guard. Extracted from current `invoke_user_main_orchestrated` (lines 757-810). | New substrate fn + refactor of `invoke_user_main_orchestrated` to delegate to it (no behavior change for fork-program-ast/wat-cli call sites). | M | B, C, D, E, F | none |
| **B** | `wat-cli` shrinks to shim. Argv → `bootstrap_wat_vm_process` → `apply_function(:user::main)` → cleanup. The bootstrap ceremony wat-cli embeds today disappears (delegated to substrate). | wat-cli ~30 lines smaller; argv handling stays caller-side. | S | E, F | A |
| **C** | `spawn_process_child_branch` shrinks to shim: dup2 fd 0/1/2 (currently only fd 2) → `bootstrap_wat_vm_process` → `apply_function(closure_entry)` → cleanup. `Process` struct fields reshape to **real stdio only** (`stdin: IOWriter`, `stdout: IOReader`, `stderr: IOReader`); retire the typed-channel fields (`tx`/`rx`) — slice 1c's typed-channel-at-process-boundary was a wrong turn, undone here. Mint substrate-as-teacher diagnostics for retired `process-send`/`process-recv`/`Process/tx`/`Process/rx` (Pattern 2 verb retirement). Mint wat-level `:wat::kernel::Sender/from-pipe` + `:wat::kernel::Receiver/from-pipe` wrappers — typed-channel abstraction LIVES ON at user level via EDN-over-pipes wrappers; users opt in. Migrate `run-hermetic-with-io` consumers + `:user::process` contract sites. TIERS.md amendment row clarifying tier-2 Sender/Receiver is a wat-level wrapper, not substrate-built-in. **Fixes the original "fatal flaw" — spawn-process children get full ambient stdio. Preserves TIERS.md uniformity at user-API level via wrappers.** | spawn_process.rs significantly smaller + honest; Process struct reshaped; wat-level wrappers minted; consumers swept. | M-L | E, F | A |
| **D** | `pub(crate) fn bootstrap_wat_thread(parent_sym) -> ThreadRuntime`. For spawn-thread's worker entry within an existing process. Inherits parent's services Arc via SymbolTable clone. Installs ThreadIO for the new thread's thread-local. Returns per-thread cleanup guard. Trivial process-internal install. | Substrate fn + spawn-thread routing. | S | E, F | A |
| **E** | `apply_function` enforces "must have runtime context." Debug-assertion (or type-level if cheap): if `sym.runtime_services()` is Some but ThreadIO not installed for current thread → panic with `RuntimeContextMissing`. Catches direct-Rust-caller bypass attempts (test harnesses must opt in via test-bootstrap helper). | Assertion in apply_function + test-harness helper. | S/M | F | A, B, C, D |
| **F** | Pattern 3 CheckError: scan for `libc::fork` / `std::thread::spawn` outside the canonical bootstrap module. Substrate-author teacher. Defense-in-depth for future substrate authors. | New CheckError variant + walker (cargo xtask or build-time check). | M | G | E |
| **G** | Documentation: ZERO-MUTEX § "Zero wat-vm contexts without runtime bootstrap by architecture." CONVENTIONS.md cross-ref. wat-cli + spawn-process module docs note "tiny shim, substrate owns runtime-context establishment." | Doc additions. | S | (terminal) | A-F |

**Critical path:** A → (B + C + D parallel) → E → F → G. Total: ≈ 5 substrate slices + 1 doc slice.

## Two honest edges (named, not hidden)

1. **Service threads themselves (chicken-and-egg, BY DESIGN):** the trio (`StdInService`/`StdOutService`/`StdErrService`) are spawned via `spawn-thread` during bootstrap (Stone A's step 2). They don't have services installed BECAUSE they ARE the services. The `sym.runtime_services()` returning `None` during the bootstrap window is the documented escape (see `freeze.rs:762-769`). This is structurally bounded: only 3 trio threads, only during bootstrap.

2. **Direct Rust callers of `apply_function`:** test harnesses + embedded use that call wat without going through `bootstrap_wat_vm_process`. Stone E closes this — `apply_function` panics if context missing. Tests opt in via a test-bootstrap helper. Substrate-author embedding is forced through the canonical path.

After A-G ships, the strong claim becomes accurate: **every wat program a user can write or run — anywhere it can execute — has runtime services. The only exceptions are the substrate's own service threads (chicken-and-egg, designed) and explicit Rust embeddings (which must opt in via test-bootstrap).**

## Dependency to SPAWN-MIGRATION-BACKLOG.md

The original 6-item migration backlog assumed `spawn-process` and `spawn-thread` were already capable of being the consolidated primitives. They aren't — the bootstrap gap means consolidation onto them would ship the migration onto broken targets.

Insertion order (amends SPAWN-MIGRATION-BACKLOG to 8 items):

| Spawn-mig # | Step | Depends on RUNTIME-BOOTSTRAP-BACKLOG |
|---|---|---|
| 3 | V5 retry — SHIPPED | none |
| **3.5 (NEW)** | **Runtime-bootstrap backlog stones A-D minimum** | This file |
| 4 | Phase F — retire `run-sandboxed-*` | A-D shipped |
| 5 | Consumer sweep — migrate `fork-program*`/`spawn-program*` callsites | A-D shipped |
| 6 | Destructive reap — kill `fork-program*`/`spawn-program*` | A-D shipped; E-F for level-2 strength |
| 7 | Phase H — clippy clean | (independent) |
| 8 | Slice 5 — INSCRIPTION | A-G shipped for honest "level-2 done" claim |

## Substrate-as-teacher discipline applied

This backlog IS the failure-engineering recovery from the original spawn-process stdio gap. Each stone eliminates a CLASS of failure structurally:
- **Stone A** eliminates "bootstrap duplication" — only one implementation exists
- **Stone B + C** eliminate "spawn primitive forgets to install services" — callers can't, because they no longer DO the install (substrate does)
- **Stone D** eliminates the spawn-thread thread-local-ThreadIO-missing edge
- **Stone E** eliminates "test/embedding bypass" — apply_function refuses to run without context
- **Stone F** eliminates "future substrate author adds a new fork/spawn outside the canonical site" — the check catches it
- **Stone G** documents the discipline so future readers find the teaching at the source

## What this is NOT

- NOT a parallel implementation alongside `invoke_user_main_orchestrated`. Stone A extracts FROM it; both wat-cli and the orchestrated path delegate to the new helper.
- NOT spawn-primitive-side install logic. wat-cli and spawn-process don't carry "remember to call services install" — there's nothing for them to call separately; the substrate does it as part of giving them a runtime.
- NOT a doctrine call to write more docs. The discipline is structural; G is the post-implementation summary.

## Discipline references

- [`docs/SUBSTRATE-AS-TEACHER.md`](../../../SUBSTRATE-AS-TEACHER.md) — Pattern 3 for Stone F
- [`docs/ZERO-MUTEX.md`](../../../ZERO-MUTEX.md) — "never construct the situation" precedent
- [`docs/COMPACTION-AMNESIA-RECOVERY.md`](../../../COMPACTION-AMNESIA-RECOVERY.md) § FM 17 — pre-action sweep before each stone
- [`docs/INTENTIONS.md`](../../../INTENTIONS.md) — one canonical path per task
- [`docs/arc/2026/05/170-program-entry-points/TIERS.md`](./TIERS.md) — the architecture
- [`docs/arc/2026/05/170-program-entry-points/SPAWN-MIGRATION-BACKLOG.md`](./SPAWN-MIGRATION-BACKLOG.md) — the blocked backlog
- [`scratch/FAILURE-ENGINEERING.md`](../../../../../../scratch/FAILURE-ENGINEERING.md) — eliminate the class
- Memory: `feedback_substrate_owns_not_callers_match` — the cognitive failure mode that led here

## Progress tracking

Update status header + stone status as each ships. SCORE doc per stone. Atomic commits per stone. Push after each commit.

When all 7 stones ship: this backlog closes; SPAWN-MIGRATION-BACKLOG unblocks at Step 4+; arc 170's "level-2 foundation" claim becomes honest.
