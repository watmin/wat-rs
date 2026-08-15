# BRIEF — 296 H-1b: `program::Env`'s members lose their dotted prefixes

> The wall (H-1) is **already in the working tree, uncommitted**. It is not yours to rebuild, revert,
> or narrow. Your strike is the one heretic it found.

## WHERE THINGS STAND

H-1 landed a wall: **a dot in a name registered through `resolve::gate` is refused.** Build and clippy
are clean. The floor is red — `4422 run, 2026 passed, 2396 failed` — with **exactly one cause**:

```
GATE-REJECT   :wat::program::Env/wat.peer-kind        DottedName
GATE-REJECT   :wat::program::Env/wat.started-at       DottedName
GATE-REJECT   :wat::program::Env/wat.peer-started-at  DottedName
GATE-REJECT   :wat::program::Env/wat.process-id       DottedName
GATE-REJECT   :wat::program::Env/wat.os-thread-id     DottedName
GATE-REJECT   :wat::program::Env/wat.cpu-count        DottedName
GATE-REJECT   :wat::program::Env/user.program         DottedName
```

`:wat::program::Env` is loaded early as core stdlib, so seven rejected accessors cascade into
unresolved-callee failures across nearly every test that boots a runtime.

**Measured: `Env` is the SOLE dotted-binder record in the entire corpus.** The wall found one heretic
and nothing else.

## THE RULING — the prefixes are a FOSSIL, not a convention to preserve

Builder, 2026-08-15:

> *"we initially wanted to allow wat and user defined values... we instead swapped to having a
> dedicated user defined record where they hold all of their complexity. so the top level keys are by
> nature wat provided, anything on the user record is from the user."*

The `wat.` / `user.` prefixes exist to mark a distinction **the design abandoned**. Under the current
shape the position already carries it: a top-level field *is* wat-provided; anything inside the user
record *is* the user's. The prefix is re-stating structure that the structure already states.

So they are **dropped, not respelled.** Kebab (`wat-started-at`) was considered and rejected — it
would carry a dead distinction forward in a new spelling, which is the fossil surviving its own
retirement. This arc has already killed two of those today (`Record::of`, `ThreadPeer`).

| was | becomes |
|---|---|
| `wat.started-at` | `started-at` |
| `wat.peer-started-at` | `peer-started-at` |
| `wat.process-id` | `process-id` |
| `wat.os-thread-id` | `os-thread-id` |
| `wat.peer-kind` | `peer-kind` |
| `wat.cpu-count` | `cpu-count` |
| `user.program` | `program` |

No collisions: `EmptyEnv` is field-free, and `Env` is the only record declaring any of these
(measured). A user's own env record is a separate type, so structural subtyping is unaffected.

## THE WORK

**Surface: 277 occurrences — 20 `.wat` files, 22 `.rs` files.** `user.program` alone is 131 of them.

1. **`wat/program.wat:39-46`** — the declaration itself.
2. **The `.wat` corpus (20 files)** — **a wat-fix codemod, never hand edits** (R21, and
   `holon/CLAUDE.md`'s first rule). Copy an existing recorded migration from `wat-scripts/fixes/` as
   the shape, **dry-run against a `/tmp` copy and `diff` it**, then apply to every path and commit the
   codemod as the recorded migration. It is idempotent; re-running is 0 changes.
3. **The `.rs` sites (22 files)** — these are string literals, so the compiler will not find them.
   Most are test assertions naming a field. Sweep them with the field list above.
4. **`wat/program.wat:23-38`** — the doc block **explains the dead convention** ("All `wat.*` fields
   are reserved/platform-owned", "NOT `wat.*` — user data, distinct from platform-owned fields").
   Rewrite it to say what is now true: top-level fields are kernel-stamped by position; user data
   lives in `program`. A comment that teaches a retired design is the same defect class this arc fixed
   in `9be5cc90`.

## THE GATE

The floor returns to **4417 passed / 0 failed / 263 skipped**, plus H-1's `dotted_name_rejected`
probe. Nothing else moves. If the count lands anywhere else, say exactly where and why.

## STOP TRIGGERS — rejections. Report and leave the site.

- **STOP-1 — a second dotted-binder record surfaces.** The census says `Env` is the only one. If the
  wall rejects something else, that is a new finding — report it with its `file:line`; do not rename
  it on your own authority.
- **STOP-2 — a de-prefixed name collides** with an existing field on the same record or on a type
  that must structurally satisfy `Env`. Report the pair; do not disambiguate by inventing a suffix.
- **STOP-3 — the codemod cannot express the rewrite** and hand-editing `.wat` starts to look
  necessary. Hand-editing is precisely what the codemod exists to prevent. Report the shape that
  defeated it.
- **STOP-4 — the floor does not return to 4417 + 1** after the rename. Something other than `Env` was
  wrong. Capture it whole and report rather than chasing it.

## BLAST RADIUS

`wat/program.wat`, the 20 `.wat` corpus files, the 22 `.rs` sites, and one new
`wat-scripts/fixes/*.wat` codemod. **Do not touch** `src/resolve/registration.rs`, `src/types.rs`,
the error taxonomies, or any of H-1's 9 call sites — that work is landed in the tree and correct.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D
warnings` (0), then `scripts/floor.sh` and read the **Summary line** — never a piped exit code.

**On any red: do NOT re-run.** A re-run that goes green destroys the only evidence. Copy the failing
test's whole stdout+stderr block verbatim — never a `| head` window — name the exact assertion that
fired, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Run
every build and test in the FOREGROUND and block on it. Anchor at `/home/watmin/work/holon/wat-rs`;
`pwd` first. Leave the work uncommitted; the orchestrator weighs and commits.

Report: the codemod you wrote and its dry-run diff summary, the `.rs` sweep, the floor Summary line
verbatim, every STOP, and the honest deltas — especially anywhere this brief did not match the disk.
Every rider on this arc so far has found a defect in the orchestrator's own brief. That is the bar.
