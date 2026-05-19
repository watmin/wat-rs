# Arc 213 stone δ-1 — Add `pidfd: Pidfd` field to ChildHandleInner

**Your ONE concern this spawn:** add `pidfd: Pidfd` field to `ChildHandleInner` (src/fork.rs:184). Change `ChildHandleInner::new(pid, lifeline_w)` signature to `(pidfd, lifeline_w)`; extract `pid` from `pidfd.pid()` internally. Update the 3 construction sites (γ-1 + γ-2 + γ-3) to pass the `Pidfd` instead of extracting pid then dropping the Pidfd.

**Strict additive at the WAIT/KILL layer.** δ-1 does NOT modify `wait_or_cached`, `Drop::drop`, or `eval_kernel_wait_child` — those continue to use `self.pid` via libc::waitpid/kill. δ-2 (next stone) migrates them to use `self.pidfd.wait_status()` / `self.pidfd.send_signal()`. δ-3 retires the libc fallback + removes `pid` field.

After δ-1: substrate STORES the pidfd at every forked-child construction site; doesn't yet USE it; libc::waitpid/kill paths unchanged. δ-1 is the smallest possible mint stone.

---

## Audit-grounded scope (verified post γ-3 at commit `4ae371a`)

### The struct (src/fork.rs:184-201)

```rust
pub struct ChildHandleInner {
    pub pid: libc::pid_t,                          // ← stays (libc paths use it)
    pub reaped: AtomicBool,
    pub cached_exit: OnceLock<i64>,
    pub lifeline_w: Option<std::os::fd::OwnedFd>,
    // NEW δ-1 field:
    // pub pidfd: Pidfd,                            // ← added
}
```

### The constructor (src/fork.rs:204-211)

**Current:**
```rust
pub fn new(pid: libc::pid_t, lifeline_w: Option<OwnedFd>) -> Self {
    Self { pid, reaped: AtomicBool::new(false), cached_exit: OnceLock::new(), lifeline_w }
}
```

**δ-1 target:**
```rust
pub fn new(pidfd: Pidfd, lifeline_w: Option<OwnedFd>) -> Self {
    Self {
        pid: pidfd.pid(),
        reaped: AtomicBool::new(false),
        cached_exit: OnceLock::new(),
        lifeline_w,
        pidfd,
    }
}
```

Signature change: `pid: pid_t` → `pidfd: Pidfd`. Three caller updates (mechanical).

### The 3 construction sites (γ-phase code that currently drops the Pidfd)

**Site 1 (γ-1):** `src/fork.rs:680` — eval_kernel_fork_program_ast
- Currently: extracts `let pid = pidfd.pid();` then `Arc::new(ChildHandleInner::new(pid, Some(lifeline_w)))` then implicitly drops `pidfd` at function scope end
- After δ-1: `Arc::new(ChildHandleInner::new(pidfd, Some(lifeline_w)))` — Pidfd moves into ChildHandleInner

**Site 2 (γ-2):** `src/fork.rs:1085` — fork_program_from_source
- Same pattern; same migration

**Site 3 (γ-3):** `src/spawn_process.rs:255` — eval_kernel_spawn_process
- Same pattern; same migration

After all three sites migrate, the implicit `drop(pidfd)` at function-scope end is GONE — the Pidfd lives inside the Arc<ChildHandleInner>.

### What does NOT change in δ-1

- `wait_or_cached` (line 217) — still uses `self.pid` via libc::waitpid
- `Drop::drop` (line 236) — still uses `self.pid` via libc::kill + libc::waitpid
- `eval_kernel_wait_child` (line 296) — still uses `handle.pid` via libc::waitpid
- ALL libc::waitpid + libc::kill paths remain functional

δ-2 migrates those paths to use `self.pidfd` methods. δ-3 retires the libc fallback + removes `pub pid` field.

---

## What to migrate

### 1. ChildHandleInner struct + constructor (src/fork.rs:184-211)

Add `pub pidfd: Pidfd` field (no Option — every construction site has a Pidfd). Change `new(pid, lifeline_w)` to `new(pidfd, lifeline_w)`; extract pid internally.

Update the doc comment on `pid` to note "diagnostic + libc interop until δ-3 retires it." Add doc comment on `pidfd` per the worked design.

### 2. Site 1 — `src/fork.rs:680` (γ-1 site)

Current line surrounding context (lines ~670-685):
```rust
// γ-1: pidfd used only to retrieve pid. δ migrates ChildHandleInner ...
let pid = pidfd.pid();
let lifeline_w = lifeline_writer.into_owned_fd();
let handle = Arc::new(ChildHandleInner::new(pid, Some(lifeline_w)));
```

After δ-1:
```rust
let lifeline_w = lifeline_writer.into_owned_fd();
let handle = Arc::new(ChildHandleInner::new(pidfd, Some(lifeline_w)));
```

The `let pid = pidfd.pid();` line becomes redundant (ChildHandleInner extracts internally). The doc-comment hint about "δ migrates" can be updated to "δ-1: pidfd stored in handle; δ-2 will route waits through it."

### 3. Site 2 — `src/fork.rs:1085` (γ-2 site)

Same pattern as Site 1. Drop the `let pid = pidfd.pid();` line; pass `pidfd` directly.

### 4. Site 3 — `src/spawn_process.rs:255` (γ-3 site)

Same pattern as Site 1. Drop the `let pid = pidfd.pid();` line; pass `pidfd` directly.

### 5. Pidfd import in fork.rs (if needed)

`Pidfd` is defined in src/fork.rs itself (α), so no import needed within fork.rs. spawn_process.rs already imports `crate::fork::{...}` items; the `Pidfd` type may need adding to that use line if it isn't already.

---

## What NOT to do

- **DO NOT** modify `wait_or_cached` (line 217) — δ-2 territory
- **DO NOT** modify `Drop::drop` (line 236) — δ-2 territory
- **DO NOT** modify `eval_kernel_wait_child` (line 296) — δ-2 territory
- **DO NOT** remove the `pub pid: libc::pid_t` field — δ-3 territory (libc paths still need it after δ-1)
- **DO NOT** retire `libc::waitpid` or `libc::kill` calls — δ-3 territory
- **DO NOT** touch γ-phase fork sites' OTHER work (closure body, OwnedFd reconstruction, etc.) — only the ChildHandleInner::new call line
- **DO NOT** change the public signature of `eval_kernel_fork_program_ast`, `fork_program_from_source`, or `eval_kernel_spawn_process` — wat dispatch arms / public APIs
- **DO NOT** introduce new types, helpers, or modules

---

## The proof gate (workspace baseline preservation)

δ-1 stores a pidfd without changing behavior — every test should pass unchanged. Use the union of γ-1/γ-2/γ-3 test binaries as the proof gate:

| Binary | Pre-δ-1 baseline (post-γ-3 at `4ae371a`) |
|---|---|
| `cargo test --release --test probe_pidfd_primitive` (α regression) | **2/2 PASS** |
| `cargo test --release --test arc112_scheme_probe` | **1/1 PASS** |
| `cargo test --release --test arc112_slice2b_process_send_recv` | **1/1 PASS** |
| `cargo test --release --test probe_closure_body_prelude_lift` | **5/5 PASS** |
| `cargo test --release --test probe_counter_actor_process_diag` | **3/3 PASS** |
| `cargo test --release --test probe_declaration_form_lift` | **6/6 PASS** |
| `cargo test --release --test probe_def_not_special` | **5/5 PASS** |
| `cargo test --release --test probe_lifeline_orphan_clean_via_fork_program` | **1/1 PASS** |
| `cargo test --release --test probe_lifeline_orphan_clean_via_substrate` | **1/1 PASS** |
| `cargo test --release --test probe_pdeathsig_diagnostic` | **1/1 PASS** |
| `cargo test --release --test probe_pdeathsig_kills_orphan_child` | **1/1 PASS** |
| `cargo test --release --test probe_run_hermetic_no_deadlock` | **2/2 PASS** |
| `cargo test --release --test probe_spawn_process_parent_type` | **3/3 PASS** |
| `cargo test --release --test probe_spawn_process_stdin` | **1/1 PASS** |
| `cargo test --release --test probe_spawn_process_stdio` | **1/1 PASS** |
| `cargo test --release --test wat_arc170_program_contracts` | **24/24 PASS** |
| `cargo test --release --test wat_arc170_stone_a_drain_and_join` | **4/4 PASS** |
| `cargo test --release --test wat_arc208_process_io_result` | **7/7 PASS** |
| `cargo test --release --test wat_process_peer_ipc_round_trip` | **3/3 PASS** |
| `cargo test --release --test wat_harness_deps` | (verify pre-spawn) |
| `cargo test --release --test probe_shutdown_cascade_crossbeam` | (verify pre-spawn) |
| `cargo test --release --test probe_shutdown_cascade_pipefd` | (verify pre-spawn) |
| `cargo test --release -p wat-cli --test wat_cli` | **15/15 PASS** |

**Orchestrator records final pre-spawn baseline (all binaries listed above) in the spawn prompt; sonnet verifies post-migration.** ANY regression IS δ-1's.

### Verification protocol (post-migration)

1. `cargo build --release 2>&1 | tail -5` — clean build
2. Re-run each cargo test command above; record pass counts
3. Compare post-counts to pre-counts (in spawn prompt)
4. Write SCORE at `docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-DELTA-1-CHILDHANDLE-PIDFD-FIELD.md`

---

## STOP triggers — VERBATIM

Non-negotiable.

1. **You modify `wait_or_cached`.** δ-2 territory. STOP.

2. **You modify `Drop::drop` on ChildHandleInner.** δ-2 territory. STOP.

3. **You modify `eval_kernel_wait_child`.** δ-2 territory. STOP.

4. **You remove the `pub pid: libc::pid_t` field.** δ-3 territory. STOP — δ-1 keeps it.

5. **You retire `libc::waitpid` or `libc::kill` calls.** δ-3 territory. STOP.

6. **You touch γ-phase fork-site code OTHER THAN the ChildHandleInner::new call line.** Out of scope. STOP — the closure body, OwnedFd reconstruction, parent-side close are all γ work; δ-1 only updates the one constructor invocation per site.

7. **A test that PASSED on baseline FAILS post-migration.** STOP. Inscribe which test + diagnostic + your hypothesis.

8. **cargo build FAILS.** STOP. Inscribe error. One syntactic-fix retry allowed (e.g., missing `Pidfd` import in spawn_process.rs).

9. **You feel the urge to also migrate ChildHandleInner to use a Pidfd-only state model.** STOP — δ-3 is that endpoint; δ-1 is additive only.

---

## What the SCORE file contains

`docs/arc/2026/05/213-libc-fork-mismanagement/SCORE-213-DELTA-1-CHILDHANDLE-PIDFD-FIELD.md`:

1. Header: `# Arc 213 stone δ-1 — SCORE: ChildHandleInner pidfd field added`
2. Summary: field minted; 3 construction sites updated; wait/kill paths unchanged
3. File changes:
   - `src/fork.rs` — ChildHandleInner struct + new() + 2 construction sites
   - `src/spawn_process.rs` — 1 construction site + possibly import update
4. Verification: pre/post pass counts for each of the 23 test binaries listed
5. Notes:
   - Pidfd lifetime now bound to Arc<ChildHandleInner> (Pidfd::Drop fires on last Arc drop)
   - Any compiler errors that surfaced + how resolved
   - Confirmation that no behavior changed (δ-1 stores; δ-2 uses)
6. Mode classification

---

## Constraints

- Edit `src/fork.rs` (ChildHandleInner struct + new() + 2 construction call sites)
- Edit `src/spawn_process.rs` (1 construction call site + possibly import update)
- ZERO other code edits
- ZERO git operations (orchestrator commits)
- Run cargo build + 23 test binaries

---

## Time prediction

30-45 min. Smallest stone since β. Single struct field addition + one signature change + 3 mechanical caller updates.

---

## Mode classification

- **Mode A:** field added; signature changed; 3 sites updated; cargo build clean; ALL baselines preserved (87+harness+shutdown-cascade binaries); SCORE written; mode-classified
- **Mode B (acceptable):**
  - Pidfd ownership / Arc-clone interaction has a non-obvious complication you can describe but not resolve in this stone; REVERT + inscribe + return
  - A test fails in a way that surfaces unexpected behavior change (Pidfd's Drop closing the fd earlier than expected, etc.)
- **Mode C:** STOP rule broken (touched δ-2/δ-3 territory, modified wait/Drop paths, removed pid field, retired libc::waitpid/kill)

The substrate teaches; α minted Pidfd; β/γ proved it on spawn paths; δ-1 stores it in the substrate-canonical handle. δ-2 routes the wait through it. δ-3 retires the libc fallback.
