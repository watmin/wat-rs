# BRIEF — `sigaction`, not `signal`

Replace the five `libc::signal()` installs with `libc::sigaction()`, flags **explicit** and
**behaviour preserved**. `src/process/child.rs`.

A **declaration**, not a behaviour change.

## Read in order

1. **`FINDING-the-writes-kept-the-1970s.md`** — why the invisible `SA_RESTART` matters, and why
   clearing it is *not* the fix.
2. **`src/process/child.rs:70-95`** — `install_substrate_signal_handlers`, the five installs.
3. **`src/process/child.rs:28-45`** — the async-signal-safety rationale for the handler bodies.
   **Unchanged by this stone; do not touch the bodies.**
4. **`src/io.rs:670`** — the *"deliberately unresolved"* comment this stone makes answerable.

## The work

**1. Five `sigaction` installs** — `SIGINT`, `SIGTERM`, `SIGUSR1`, `SIGUSR2`, `SIGHUP` — each with
`sa_flags` **written explicitly**.

**2. Preserve today's behaviour.** glibc's `signal()` means BSD semantics, so `SA_RESTART` is set
today; set it explicitly. **Behaviour identical, default now visible.**

**3. Set `sa_mask`** deliberately (empty is the `signal()` equivalent — say so in a comment).

**4. Check the return value.** `signal()`'s was being discarded; `sigaction` failing at boot should
not pass silently.

**5. Update `src/io.rs:670`'s comment** — the question is no longer "deliberately unresolved"; the
flag is now stated at the install site. Point the comment there.

## ⚠ THE TREE IS DIRTY WITH THREE FILES THAT ARE NOT YOURS

| file | whose | state |
|---|---|---|
| `src/io.rs` | **stone 1** (`a write never blocks outside the multiplexer`) | struck, verified, uncommitted |
| `src/comms/process.rs` | **stone 1** | struck, verified, uncommitted |
| `wat-scripts/fanout/circuit.wat` | `a reconnect is not an abandonment` | struck, uncommitted |

**Do not revert, commit, or edit any of them.** ⚠ Note `src/io.rs` appears in *your* work too — you
change **one comment** in it (`:670`), nothing else. Stone 1's changes to that same file must survive
your edit intact.

★ If you restore anything, restore only `src/process/child.rs`.

## Blast radius

`src/process/child.rs`, plus the one comment in `src/io.rs`. Rust substrate — a rebuild is in play.

## STOP triggers

- **STOP-1** — if any flag change alters observable behaviour, **STOP.** This stone preserves; a
  different one decides.
- **STOP-2** — **do not clear `SA_RESTART`.** Its own stone, and likely unnecessary after stone 1.
- **STOP-3** — **do not touch the handler bodies** or their async-signal-safety reasoning.
- **STOP-4** — floor red on any arm: **STOP; do not re-run.** Name the exact arm.
- **STOP-5** — if `CLONE_CLEAR_SIGHAND` interacts with handler installation in a way the current
  code depends on, **STOP and report it.** That interaction is exactly what this area is for.
- **STOP-6** — **do not revert or commit `src/io.rs`, `src/comms/process.rs`, or `wat-scripts/`.**
  They carry two other stones' uncommitted work. Your only edit in `src/io.rs` is the one comment at
  `:670`.
