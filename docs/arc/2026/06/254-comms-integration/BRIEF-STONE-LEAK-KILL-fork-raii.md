# BRIEF — Stone (214 Slice-4 prep / 253 inst-2): kill the orphan/fd leak CLASS — `into_raw_fd` fork-boundary surrender (3 sites) → RAII

## The work (one paragraph)

The fork/spawn paths surrender fd ownership by hand (`into_raw_fd()` disables `OwnedFd::Drop`), then rely on manual `libc::close` after `spawn_lifelined` returns. The window between the surrender and the manual close LEAKS: if `spawn_lifelined` errors (the `?` after the surrender) or anything panics, all six raw fds are never closed → the leaked pipe ends keep the child from seeing EOF → orphan process → the setsid+pkill containment exists to slaughter exactly this. **This is a CLASS at THREE identical sites — fix all three** (extirpare: pull the class, not one instance). Convert each to RAII: the parent **holds** `OwnedFd`s, passes raw `i32`s (via `.as_raw_fd()`, Copy) into the clone3 closure, the child re-wraps its inherited copies, the parent drops child-side ends after return, and on ANY early-return/panic every `OwnedFd` `Drop`s and closes — the leak becomes unrepresentable.

## The three sites (identical pattern — same fix each)

1. **`src/spawn_process.rs:148-230`** — the breadcrumb's NAMED primary site (`spawn-process`). Surrender at `:162-167`; manual close at `:227`.
2. **`src/fork.rs:611-700`** (`fork-program-ast`) — surrender `:623-628`, child closure `:639-669`, parent close `:682-685`, re-wrap `:689+`.
3. **`src/fork.rs:968-1090`** (the source-string `fork_program` variant) — surrender `:984-989`, manual close `:1083`.

All three are byte-for-byte the same shape (the comments are even identical). One RAII pattern fixes all three.

## Read in order (the rooms)

1. `src/spawn_process.rs:148-230` + the two `src/fork.rs` regions above — the three leak sites.
2. `src/comms/process.rs:1050-1066` — the RAII model: `OwnedFd` held, `from_raw_fd` used ONCE at construction, *"so Drop closes them."* Mirror this discipline.
3. The `use std::os::fd::{... IntoRawFd ...}` imports in each file (drop `IntoRawFd` where no longer used after the fix).
4. **`src/io.rs:593`** — `fd: AtomicI32::new(fd.into_raw_fd())` — a DIFFERENT pattern (long-lived raw fd stored in an atomic). AUDIT it: confirm a closing `Drop`/close path exists, or flag it as a separate leak. Do NOT force it into the fork-boundary fix; report the finding.

## The RAII pattern to mirror (the working shape)

```rust
// Parent holds OwnedFds (RAII). No into_raw_fd.
let (stdin_r, stdin_w) = make_pipe(OP)?;   // OwnedFd
let (stdout_r, stdout_w) = make_pipe(OP)?;
let (stderr_r, stderr_w) = make_pipe(OP)?;

// Raw ints for the clone3 closure — COPIES of the fd numbers, NOT ownership.
// .as_raw_fd() borrows; the OwnedFds remain owned by the parent scope and
// stay open across clone3 (child inherits them via the kernel fd-table copy).
let stdin_r_raw  = stdin_r.as_raw_fd();
let stdin_w_raw  = stdin_w.as_raw_fd();
// ... (all six)

let (pidfd, lifeline_writer) = spawn_lifelined(move |lifeline_r_raw: i32| {
    // CHILD (separate process after clone3): re-wrap its inherited copies.
    // SAFETY: inherited via clone3; child owns its copies; Drop is the child's.
    let stdin_r = unsafe { OwnedFd::from_raw_fd(stdin_r_raw) };
    // ... child_branch as before
})
.map_err(...)?;   // ← ON ERROR: the six parent OwnedFds are STILL ALIVE → Drop closes all. NO LEAK.

// PARENT after success: close the child-side ends by DROPPING their OwnedFds
// (replaces the manual libc::close at 682-685).
drop(stdin_r); drop(stdout_w); drop(stderr_w);
// Keep stdin_w / stdout_r / stderr_r (parent-side) as OwnedFds for the Process.
```

## The hazard to preserve (do NOT regress)

- **No double-close.** After clone3 (`spawn_lifelined` is clone3, fork-like, SEPARATE address space + fd-table copy — NOT `CLONE_VM`), the parent's `OwnedFd` and the child's re-wrapped `OwnedFd` are in different processes; each closes its own copy once. Within the parent, only the `OwnedFd` closes each fd (the closure captured `i32` Copies — no `Drop`). Confirm `spawn_lifelined` is not `CLONE_VM` (if it shares the address space, the OwnedFd-hold approach is unsound → STOP).
- **fds must stay open across clone3.** The parent `OwnedFd`s must live in scope THROUGH the `spawn_lifelined` call (so the child inherits open fds). The `as_raw_fd()` borrows keep them owned; do not drop before the call.
- **Child unchanged in behavior.** The child still re-wraps + runs `child_branch` identically.

## Verification (the leak must be DEAD)

- `cargo test --release --lib -p wat` — green (no regression in spawn/fork).
- **The leak-kill proof:** run the arc-170 `#[ignore]`'d process/fork tests (grep `#[ignore]` near fork-program/spawn-process integration tests) — un-ignore the ones the leak forced off and confirm they pass leak-free; OR write `tests/nursery/probe_fork_fd_lifecycle.rs`: fork-program-ast N times (each child exits), assert `/proc/self/fd` count is stable before/after (no fd leak) and no orphan/zombie remains. Run leak-safe (`integration-run.sh` containment); the point is to prove the leak is gone, so it must run WITHOUT the setsid+pkill crutch masking it.
- `cargo clippy --release` — no new warnings; `IntoRawFd` import removed if unused.

## STOP triggers (reject, do not work around)

1. **STOP** if `spawn_lifelined` uses `CLONE_VM` (shared address space) — then parent + child share the fd table and the OwnedFd-hold + child-re-wrap would double-close; report the clone flags and the real ownership model before any edit.
2. **STOP** if the parent genuinely needs a child-side fd after the fork for a reason the current code hides — surface it; do not leak to preserve it.
3. Leave uncommitted for scoring.

## Why this is the first swing

This is the surgical leak-kill at the named smell (`into_raw_fd`), verifiable (the leak dies / the ignored tests green), and it establishes RAII fd ownership in the fork path — the exact discipline the `Process<I,O>` peer type (next stone) is built on. It retires the setsid+pkill containment (arc-253 instance-2) at root. *No cattle to line up; no mess to cleanse.*
