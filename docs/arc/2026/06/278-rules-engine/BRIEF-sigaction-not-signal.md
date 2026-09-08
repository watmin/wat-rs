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
