# SCORE — Stone: the orphan/fd leak-kill (`into_raw_fd` fork-boundary class → RAII)

Scored against an INDEPENDENT orchestrator re-run. **PASS — the leak is dead.**

## Scorecard

| # | what | result |
|---|---|---|
| 1 | `into_raw_fd` surrender removed at ALL three sites | **PASS** — grep: no `into_raw_fd` in `spawn_process.rs`/`fork.rs` (only "No into_raw_fd" comments) |
| 2 | RAII: error-path leak killed by construction | **PASS** — `as_raw_fd()` borrows; the six `OwnedFd`s stay in scope through the `?` early-return → `Drop` closes all on error/panic (verified by reading the diff) |
| 3 | no double-close | **PASS** — child re-wraps in its separate post-clone3 address space; parent drops child-side ends once (`spawn_process.rs:228-230`, `fork.rs:683-685`, `:1076-1078`), keeps parent-side |
| 4 | `io.rs:593` audit | **PASS (not a leak)** — `PipeWriter::Drop` (`:598`) `libc::close`es the `AtomicI32` fd (`:605/:667`); single-owner, single close |
| 5 | lib baseline | **PASS** — my re-run: 940/0/1 |
| 6 | fd-leak proof | **PASS** — `probe_fork_fd_lifecycle` (my re-run): before=3, after=3 over 10 `fork_program_from_source` cycles; ran clean leak-safe (setsid+timeout, no pkill crutch) |
| 7 | clippy | PASS (agent; `IntoRawFd` imports removed; RAII diff introduces nothing warn-prone) |

## What shipped

`src/spawn_process.rs` (site 1, the named primary), `src/fork.rs` (sites 2 fork-program-ast + 3 source-string fork): `into_raw_fd()` → `as_raw_fd()` + parent-`drop()` RAII. `tests/nursery/probe_fork_fd_lifecycle.rs` (new, `#[ignore]` process test, runs via `integration-run.sh`). `IntoRawFd` imports dropped.

## Honest coverage note

The probe exercises site-3's **success** path (fd stable across cycles). The **error/panic** path (the *original* leak: `spawn_lifelined` fails after surrender) is killed **by construction** — the `OwnedFd`s Drop on the `?` — verified by reading the diff, not by the probe. Sites 1+2 carry the byte-identical fix (diff + grep confirmed) but are not separately probed. Coverage = construction-correctness (error path + all sites) ∪ probe (site-3 happy path). Sound.

## What this unlocks (the unwind)

This is **arc-253 instance-2 resolved at root** — the orphan/fd leak the setsid+pkill containment exists for is now unrepresentable. NEXT: the setsid+pkill containment apparatus is retirable (un-ignore the arc-170 `#[ignore]`'d process tests — task #183 — and confirm they pass leak-free without the crutch). Then the `Process<I,O>`/`Thread<I,O>` peer types on `comms` (this RAII fd-ownership is their first brick) = the rest of 214 Slice 4.
