# BRIEF — STONE 255.1c-kernel-becomes-a-directory · nine files stop repeating a prefix

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits.** Do not run
either; do not commit, push, stash, or revert. **Ending your turn ENDS you** — nothing wakes you.
Run everything in the FOREGROUND.

Anchor `/home/watmin/work/holon/wat-rs/`; `pwd` first. Any `.claude/worktrees/` path is harness state.

**Two commands are yours:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/)'
```

## Why

`ls src/intrinsic/` shows **nine of fourteen entries sharing a `kernel_` prefix.** A repeated prefix
in a flat namespace is a directory that has not been made yet — the filename is carrying hierarchy
the filesystem should carry. `intueri`'s structure rule: *the file tree should mirror the domain;
when you `ls`, you should see the architecture.* Right now you see one word nine times.

**The precedent is already on disk:** `src/intrinsic/special/` is a directory with its own `mod.rs`
carrying a real module doc. Two files earned that; nine have not got it.

## The work

```
src/intrinsic/kernel_abort.rs     →  src/intrinsic/kernel/abort.rs
              kernel_ambient.rs   →                kernel/ambient.rs
              kernel_error.rs     →                kernel/error.rs
              kernel_identity.rs  →                kernel/identity.rs
              kernel_message.rs   →                kernel/message.rs
              kernel_resource.rs  →                kernel/resource.rs
              kernel_serve.rs     →                kernel/serve.rs
              kernel_source.rs    →                kernel/source.rs
              kernel_stdio.rs     →                kernel/stdio.rs
```

**Use `git mv`** so history follows. The `kernel_` prefix DROPS from every filename — the directory
carries the namespace now, exactly as `special/binding.rs` does.

Then:
- **NEW `src/intrinsic/kernel/mod.rs`** — see below.
- `src/intrinsic/mod.rs`: the nine `mod kernel_*;` lines become **one `mod kernel;`**, alphabetically
  among the existing entries.

## `kernel/mod.rs` — the tier's own doc, said ONCE

Model it on `src/intrinsic/special/mod.rs`. It should say what unites these nine, which is a real
thing and is already written down in `255/DESIGN-STONE-255.1c-kernel-stdio.md`:

> **`:wat::kernel::` is not a family. It is a TIER** — braiding independent concerns that each have a
> different reason to change, a different test surface, and in several cases a different module.

Name the nine homes and, in one line each, the subject each holds. State that the tier's literal
dispatch in `runtime.rs` is now **empty** — every `:wat::kernel::` verb reaches its handler through
the registry.

★ **Anything currently repeated across the nine module docs that is true of the TIER rather than of
one home belongs here instead** — say it once and cut it from the nine. Anything true of only one
home stays where it is. **Do not delete a per-home finding to make this tidy**; several of those
findings are the whole point of their stones (the `peer-pid` blanket-accept note in `identity.rs`,
the two-delegate TCO derivation in `serve.rs`, the first-`Effectful`-rows note in `stdio.rs`).

## ★ Fold in: fourteen stale citations in `runtime.rs`

`grep -n 'intrinsic/kernel_' src/runtime.rs` → **14 comments** citing the old paths. **Eight of them
point at `kernel_remainder.rs`, which NO LONGER EXISTS** — its rows moved into four homes an hour
ago. That is FM 14 (surface retirement leaving stale internal references), and repointing them now
costs nothing because they must change for the move regardless.

Each citation must name the file its subject actually lives in now. `abort` / `source` / `identity` /
`serve` took `kernel_remainder.rs`'s thirteen rows between them — **read the row, then pick the
file**; do not guess from the line's neighbourhood.

⚠ **This means `runtime.rs` DOES change** — comments only, no code. So the five line-pinned `.edn`
goldens can shift. **Keep every citation a single line** so the edit is 1-for-1 and the net line
delta is ZERO; report `git diff --numstat src/runtime.rs` so I can confirm `0 0` or ratify a shift.

## Blast radius

```
MOVE  9 × src/intrinsic/kernel_*.rs → src/intrinsic/kernel/*.rs   (git mv)
NEW   src/intrinsic/kernel/mod.rs
EDIT  src/intrinsic/mod.rs    nine `mod` lines become one
EDIT  src/runtime.rs          14 citation comments, COMMENTS ONLY, one line each
```

Nothing else. No `check.rs`, no `wat/`, no `.edn`, no tests, **no code in `runtime.rs`**.

## STOP triggers

1. **STOP-1 — a row's content would have to change** to survive the move (an import that cannot be
   resolved from the new depth, a `pub(crate)` that stops reaching). Report it; the fix is likely a
   `use` path, but I want to see it before it lands.
2. **STOP-2 — `runtime.rs`'s numstat is not `0 0`.** Report the delta and which citations wrapped;
   do not adjust surrounding code to compensate.
3. **STOP-3 — a citation's correct target is ambiguous** (a comment covering rows now in two homes).
   Report it rather than picking.
4. **STOP-4 — blast radius insufficient.**

## What "done" looks like

- `cargo build --release` exits 0
- the scoped run's full Summary line, labelled scoped
- `git diff --numstat src/runtime.rs` — I need this whether or not it is `0 0`
- `git status --short` showing nine renames (`R`), one new file, two modified
- one line on what `kernel/mod.rs` claims the tier IS, and what you moved up into it from the nine
- the honest deltas

Runtime band: 30–45 minutes.
