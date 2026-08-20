# BRIEF — STONE 255.1c-kernel-message · HOME #5: carve `:Message`'s five

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits.** Do not run
either; do not commit, push, stash, or revert. **Ending your turn ENDS you** — nothing wakes you, no
notification is coming. Run everything in the FOREGROUND and block on it.

Anchor `/home/watmin/work/holon/wat-rs/`; `pwd` first; `git -C <anchor>` for git reads. Any path with
`.claude/worktrees/` is harness state — never operate on it.

**Two commands are yours:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/)'
```

Read exit codes directly, never through a pipe.

## The work in one paragraph

Five `:wat::kernel::` verbs — `send`, `try-send`, `recv`, `select`, `poll` — dispatch from literal
match arms in `runtime.rs`. Move them into a new registry home, `src/intrinsic/kernel_message.rs`, as
thin `#[wat_intrinsic]` wrappers around the **same** delegate calls, and delete the literal arms. All
five carry `@Category Message`. Registration must not change routing.

## Read in order — why each

1. **`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-255.1c-kernel-message.md`** — the stone. Its
   "★★ THE POINT" section governs what you may and may not do about types.
2. **`src/intrinsic/kernel_ambient.rs`** — home #4, the most recent shape. Module doc, the `///` doc
   contract, wrapper-around-delegate bodies. **Copy this shape.**
3. **`src/intrinsic/kernel_stdio.rs:150-170`** — `readln'`'s block. It is the ONE precedent for a verb
   whose real type is not a plain scheme, and it documents that honestly. Read how it says so.
4. The five arms — `runtime.rs:6826, 6832, 6833, 6858, 6861`.
5. The five bodies — `runtime.rs:31053, 31223, 31458, 32235, 33232`. **Read all five before declaring
   anything.**
6. The five inference arms — `check.rs:4049, 4061, 4069, 4176, 4188`. **These are the real type
   authority.** Your `@arg`/`@ret` must describe what they produce.
7. **`wat/runtime-meta.wat`** — `:Message`'s prose. It names exactly these five. Read; do not edit.

## ⛔ The type situation — read this twice

These five have **no registered `TypeScheme`**. Measured: no `env.register` for any of them; the
checker special-cases each with a bespoke `infer_*_prime` arm, because the types are projective
(`I` flows from `peer<I,O>` into the payload; `O` flows into the return) and cannot be written as a
fixed-arity scheme.

`doc_arg_ret_types_match_checker_scheme` begins `None => continue, // not yet in checker — skip`.
**It will therefore SKIP all five and go green. That green proves nothing about them.**

Two things follow, and both are hard rules:

- **Do NOT mint stub `TypeScheme`s in `check.rs` to give the gate something to agree with.** A stub
  that exists only to be agreed with is a gate reading a copy of the truth. If you find yourself
  editing `check.rs` at all, that is **STOP-2**.
- **Each row's doc must name its `infer_*_prime` fn as the authority**, in a `//` maintainer comment
  (not `///` — see `kernel_stdio.rs`'s note on which comments `render-doc` prints), and say that the
  declared `@ret` is a documented approximation where the true type is projective.

## The axis table — RE-DERIVE from the bodies, then tell me if you agree

| verb | Purity | Determinism | Category |
|---|---|---|---|
| `send` `try-send` `recv` `select` | Effectful | Nondeterministic | Message |
| `poll` | **derive it — see below** | Nondeterministic | Message |

★ **`poll` is the one to read closely.** The other four move or consume a payload — an effect another
locus can observe. If `poll` only REPORTS READINESS and consumes nothing, it has no observable effect
and is `Pure` + `Nondeterministic`. **If your reading says `Pure`, declare `Pure`** — that is the
honest answer, and it will appear as a fifth entry in
`declared_purity_vs_effectful_by_prefix_census` (which records disagreements as an INVENTORY, not a
failure). Report the census output either way. **Do not declare `Effectful` for symmetry with its
siblings** — that is the "make it agree" move this arc keeps deleting.

## In scope: `eval_poll_prime` has no doc comment

`runtime.rs:33232` — verified, its four siblings each have one describing the tier-by-tier contract.
Carving forces a `///` block; make that block state the contract, not just the axes.

## Blast radius

```
NEW   src/intrinsic/kernel_message.rs
EDIT  src/intrinsic/mod.rs     one `mod kernel_message;` line
EDIT  src/runtime.rs           delete 5 literal arms (+ replacement comment); widen delegates to
                               pub(crate) as needed; eval_poll_prime gains its doc
```

Nothing else. **No `src/check.rs` edit. No `wat/runtime-meta.wat` edit. No new types, no test edits.**

## STOP triggers — SHIP NOTHING FURTHER AND REPORT

1. **STOP-1 — routing changed.** Registration moves the LOOKUP, never the HANDLER. If a different fn
   runs, or any peer/channel test behaves differently, stop.
2. **STOP-2 — you need to touch `check.rs`.** Including "just a stub scheme so the gate passes."
   Report what you wanted and why; change nothing there.
3. **STOP-3 — a body's DOING is not "delivers or receives a payload across a peer/channel boundary."**
   `:Message`'s prose is a claim; your body-read is the check. If one does not fit, that is a finding
   — report it, do not stretch to fit.
4. **STOP-4 — the blast radius is insufficient.** Name the file and why. Do not widen alone.

## What "done" looks like

- `cargo build --release` exits 0 (a missing `@Category` is a `compile_error!`)
- the scoped run's **full Summary line**, verbatim; any failure's whole block, never a window
- **the census output** — run it visible with
  `cargo nextest run --release -E 'test(declared_purity_vs_effectful_by_prefix_census)' --no-capture`
  (a third command, authorized for this stone specifically, because the census is an artifact and a
  passing test prints nothing by default)
- your axis table with agreement or dissent per row, and the body line that decides `poll`
- `git status --short`, and the honest deltas

Runtime band: 35–55 minutes, mostly builds.
