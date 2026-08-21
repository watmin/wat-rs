# BRIEF — STONE 255.1c-kernel-error · HOME #6: the two error types' four verbs

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

## ⚠ Your test filter is BLIND to part of this stone's blast radius — do not report a green floor

`test(/intrinsic::tests::/)` does not reach `tests/diagnostics/`. Five `.edn` goldens there pin an
exact `src/runtime.rs` line number, and **deleting four arms will shift it and turn them red.** That
is expected, it is the orchestrator's ratification step, and it is not yours to fix.

**So: report your scoped run as what it is — a scoped run.** Do not write "the floor is green" or
"all tests pass". The two previous riders on this arc each reported green while the full floor was
red, for exactly this reason. Say "scoped suite green; diagnostics goldens not covered by my filter."

## The work in one paragraph

Four `:wat::kernel::` verbs — `LociDiedError/message`, `Failure/message`, `Failure/location`,
`LociDiedError/to-failure` — dispatch from literal match arms. Move them into a new registry home,
`src/intrinsic/kernel_error.rs`, as thin `#[wat_intrinsic]` wrappers around the **same** delegates,
and delete the literal arms. **They do NOT all share a `@Category`** — see below.

## Read in order

1. **`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-255.1c-kernel-error.md`** — the stone. Its
   opening section explains why this home holds two categories and why that is correct.
2. **`src/intrinsic/kernel_message.rs`** — home #5, the most recent shape. Copy it.
3. The four arms — `runtime.rs:6754, 6763, 6764, 6765`. **Read the dispatch comment at `:6757`** — it
   tells you the `Failure/*` accessors read `error.message` off a mandatory `error` field.
4. The four bodies — `runtime.rs:27423, 27452, 27716, 27812`. **Read all four before declaring.**
5. The four schemes — `check.rs:18101, 18121, 18130, 18147`. **Your `@arg`/`@ret` must match these
   exactly** — unlike home #5, the gate is live here.
6. **`wat/runtime-meta.wat`** — `:Projection`'s prose (names three of your four) and `:Transform`'s.

## The axis table — RE-DERIVE, then agree or dissent with the deciding line

| verb | Purity | Determinism | Category |
|---|---|---|---|
| `LociDiedError/message` | derive | derive | **Projection** |
| `Failure/message` | derive | derive | **Projection** |
| `Failure/location` | derive | derive | **Projection** |
| `LociDiedError/to-failure` | derive | derive | **Transform** |

★ **Do not homogenize the Category column.** `:Projection` returns a component that already existed;
`to-failure` matches `ev.variant_name` and CONSTRUCTS a `Failure` — a different-kind value. If your
body-read says otherwise on any row, that is a finding: report it, do not tidy it.

★ **`Failure/*` project one hop deeper** — `failure.error.message`, not `failure.message`. Still a
projection; derive it from the body, not the name.

## ⚠ The gate is live — the trap that will catch you

Measured scheme shapes:

```
LociDiedError/message     params [:wat::kernel::LociDiedError]  ret :wat::core::String
Failure/message           params [:wat::core::Record]           ret :wat::core::String
Failure/location          params [:wat::core::Record]           ret Option<:wat::kernel::Location>
LociDiedError/to-failure  params [:wat::kernel::LociDiedError]  ret :wat::kernel::Failure
```

**`Failure/*` take `:wat::core::Record`, NOT a `Failure` path.** Writing the obvious
`@arg f :wat::kernel::Failure` turns `doc_arg_ret_types_match_checker_scheme` red. Match the scheme.

## Blast radius

```
NEW   src/intrinsic/kernel_error.rs
EDIT  src/intrinsic/mod.rs   one `mod kernel_error;` line
EDIT  src/runtime.rs         delete 4 arms (+ replacement comment); widen 4 delegates to pub(crate)
```

Nothing else. No `check.rs`, no `wat/`, no `.edn`, no test edits.

## STOP triggers — SHIP NOTHING FURTHER AND REPORT

1. **STOP-1 — routing changed.** Registration moves the lookup, never the handler.
2. **STOP-2 — you need to touch `check.rs`.** Including to make a `@ret` agree. The scheme is the
   authority; your doc follows it. Report the pair and stop.
3. **STOP-3 — a body's DOING does not match its predicted Category.** Report with the deciding line.
4. **STOP-4 — the blast radius is insufficient.** Name the file and why; do not widen alone.

## What "done" looks like

- `cargo build --release` exits 0
- your scoped run's **full Summary line**, labelled as scoped (see the warning above); any failure's
  whole block verbatim
- your axis table with agreement or dissent per row, and the deciding line for `to-failure`
- `git status --short` and `git diff --numstat src/runtime.rs` — I need the net line delta for the
  goldens ratification
- the honest deltas

Runtime band: 25–40 minutes.
