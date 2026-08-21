# BRIEF — STONE 255.1c-split-the-remainder · one bucket becomes four homes

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits.** Do not run
either; do not commit, push, stash, or revert. **Ending your turn ENDS you** — nothing wakes you. Run
everything in the FOREGROUND.

Anchor `/home/watmin/work/holon/wat-rs/`; `pwd` first. Any `.claude/worktrees/` path is harness state.

**Two commands are yours:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/)'
```

## Why this exists

`src/intrinsic/kernel_remainder.rs` was named for **what was left over** — the `utils`/`misc` mumble.
The builder called it: *"uhm.... this is an awful name..."* And the name was diagnosing the grouping,
not just labelling it: its thirteen rows landed on **SIX** categories, so it is a bucket, not a home.

Home #6 ruled that a HOME is a code-organization unit and a CATEGORY is a per-row label — a home may
honestly hold two. **Six means the grouping was "not yet carved", which is not a subject.**

## The work — a PARTITION, not a rewrite

The rows are already contiguous by subject. Split the file into four, each with its own module doc.
**Do not change a single `@Category`, `@Purity`, `@Determinism`, `@arg`, `@ret`, `@example`, or fn
body.** Only the file they live in, and the module doc above them, changes.

```
kernel_abort.rs     raise! · assertion-failed!                     (currently ~284, ~318)
kernel_source.rs    here · call-site · macro-call-site · fn-forms  (~354, ~390, ~420, ~454)
kernel_identity.rs  require-wire-address · peer-wire? ·            (~491, ~534, ~570, ~620, ~659)
                    address-wire? · peer-pid · peer-process
kernel_serve.rs     retag-op · serve-dispatch-op                   (~701, ~772)
```

Then: delete `kernel_remainder.rs`, and replace its one `mod kernel_remainder;` line in
`src/intrinsic/mod.rs` with the four new `mod` lines, alphabetically among the existing ones.

## ★ The module doc is the judgment call — it is 253 lines covering all thirteen

Each new file needs a module doc that is **true of its own rows and no others**. Read the existing one
whole first (`kernel_remainder.rs:1–253`); it contains material that must be **distributed, not
duplicated**:

- the **`peer-pid` / blanket-accept** finding → `kernel_identity.rs`. It must keep saying plainly that
  registration does NOT remove `peer-pid` from the blanket-accept's shadow (task #110 / 255.1b-iv),
  and keep its capability-path call sites (`wat/bracket.wat:714` GRANT-BOOT, `:754` REVOKE-SHUTDOWN).
- the **`?`-suffix warning** for `:Probe` → `kernel_identity.rs`. *Do NOT file by the `?` suffix* —
  that is the axis-mix that sank `:Predicate`, and `peer-wire?`/`address-wire?` are `:Probe`'s first
  tenants ever.
- the **live-mutable-cell vs permanent-tag Determinism test** → `kernel_identity.rs` (it is what
  splits `peer-wire?`/`peer-pid` from `address-wire?`/`peer-process`).
- the **two-delegate collapse + TCO derivation** for `serve-dispatch-op` → `kernel_serve.rs`.
- the **`:ControlFlow` abandoning-vs-directing** reasoning → `kernel_abort.rs`.
- anything true of all four (the carve's provenance, the gate-coverage caveat) → say it **once** in
  the file it actually bears on; do not paste it four times.

★ **`kernel_identity.rs` is the one to get right.** It is ONE subject — *what is this peer or
address* — across THREE categories: `:Projection` projects the pid, `:Probe` asks whether it is a
wire, `:CheckGate` refuses a call site lacking a wire address. Its doc should say that, because it is
the clearest example on disk of home-vs-category and it is the reason the split is correct rather than
merely tidier.

## Blast radius

```
NEW     src/intrinsic/kernel_abort.rs · kernel_source.rs · kernel_identity.rs · kernel_serve.rs
DELETE  src/intrinsic/kernel_remainder.rs
EDIT    src/intrinsic/mod.rs   one `mod` line becomes four
```

**Nothing else.** No `runtime.rs`, no `check.rs`, no `wat/`, no `.edn`, no tests. `src/runtime.rs`
must be **byte-identical** when you are done — this stone moves no arms.

## STOP triggers

1. **STOP-1 — you need to change a row's content** to make the split compile (an axis value, a body,
   an import that cannot be resolved per-file). Report what and why; do not adjust a declared value.
2. **STOP-2 — `runtime.rs` shows a diff.** `git diff --stat src/runtime.rs` must be EMPTY. If it is
   not, something moved that should not have.
3. **STOP-3 — a row would land in a file whose subject it does not fit.** The partition above is mine
   and it is a claim; if a body says otherwise, report it.
4. **STOP-4 — blast radius insufficient.**

## What "done" looks like

- `cargo build --release` exits 0
- the scoped run's **full Summary line** (label it scoped — your filter does not reach
  `tests/diagnostics/`, though this stone should not move a single pinned line, which is itself the
  check: **`git diff --stat src/runtime.rs` empty means the goldens cannot have shifted**)
- `git status --short` and `git diff --stat` — I want to see runtime.rs absent from it
- a one-line statement per new file of what its module doc claims its subject IS
- the honest deltas

Runtime band: 30–45 minutes, most of it the module-doc split.
