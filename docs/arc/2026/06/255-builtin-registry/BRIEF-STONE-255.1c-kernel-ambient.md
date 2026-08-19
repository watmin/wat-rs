# BRIEF — STONE 255.1c-kernel-ambient · HOME #4: carve the seven `:Ambient` verbs

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits** — do not run
either, and do not commit, push, stash, or revert anything. **Ending your turn ENDS you**; nothing
wakes you, so run every command you do run in the FOREGROUND and block on it. Your turn ends when the
numbers are in your hands.

Anchor: `/home/watmin/work/holon/wat-rs/`. Verify with `pwd` as your first action. Use
`git -C /home/watmin/work/holon/wat-rs` for any git *read*. Any path containing `.claude/worktrees/`
is harness state — never operate on it.

**Two commands are yours, and only these two:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/)'
```

Read exit codes directly — never through a pipe (`… | tail` returns `tail`'s status).

## The work, in one paragraph

Seven `:wat::kernel::` verbs — `stopped?`, `sigusr1?`, `sigusr2?`, `sighup?`, `reset-sigusr1!`,
`reset-sigusr2!`, `reset-sighup!` — currently dispatch from literal match arms in `runtime.rs`. Move
them into a new intrinsic-registry home, `src/intrinsic/kernel_ambient.rs`, as thin
`#[wat_intrinsic]`-annotated wrappers around the **same** delegate calls, and delete the literal arms.
All seven carry `@Category Ambient`. The registration must not change routing: the fn that actually
runs is unchanged; only the path that reaches it differs.

## Read in order — why you are being sent to each

1. **`src/intrinsic/kernel_stdio.rs`** (211 lines) — home #3. **This is the shape you copy.** Module
   doc, the `///` doc-block contract (`@added`/`@Purity`/`@Determinism`/`@Category`/`@ret`), the
   wrapper-around-a-delegate body. `read-frame` at the bottom is your **nullary precedent** — no arg
   params, just `env, sym, list_span`. All seven of yours are nullary.
2. **`git show 3ae6c824 -- src/intrinsic/mod.rs src/runtime.rs`** — the exact wiring diff for home
   #3: one `mod` line in `mod.rs`, and the literal arms replaced by a comment naming where they went.
   Copy that comment convention.
3. **`src/runtime.rs:6737`** (`stopped?`) and **`src/runtime.rs:6882–6906`** (the six signal arms) —
   the arms you delete.
4. **`src/runtime.rs:25944`** `eval_user_signal_query` and **`:25968`** `eval_user_signal_reset` —
   the two shared bodies, plus **`eval_kernel_stopped`**. **Read all three before you declare
   anything** (see the declaration table below — you are re-deriving it, not copying it).
5. **`src/runtime.rs:68, 122–124`** — `KERNEL_STOPPED`, `KERNEL_SIGUSR1/2`, `KERNEL_SIGHUP`.
   Already `pub`; you need them reachable from the new module.
6. **`src/check.rs:17930`** and **`:18044–18070`** — the registered `TypeScheme`s. Your `@ret`
   spellings must agree with these or `doc_arg_ret_types_match_checker_scheme` goes red.
7. **`wat/runtime-meta.wat:163–169`** — `:Ambient`'s shipped prose. It names all seven members. Read
   it; do not edit it.
8. **`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-255.1c-kernel-ambient.md`** — the stone.
   Its "THE ONE CONTRACT DECISION, PINNED" section governs everything below.

## Implementation sketch — fill this in, do not invent a different shape

```rust
//! `:wat::kernel::` ambient-state intrinsics — arc 255 home #4 ...
//! (module doc: what the family is, why :Ambient, and the purity derivation)

use wat_macros::wat_intrinsic;
use crate::span::Span;
use crate::value::{Environment, EvalBreak, SymbolTable, Value};

/// `(:wat::kernel::sigusr1?)` → `:wat::core::bool`. ...
///
/// @added         1.0.0
/// @Purity        <derived from the body>
/// @Determinism   <derived from the body>
/// @Category      Ambient
/// @ret     :wat::core::bool ...
/// @example-norun (:wat::kernel::sigusr1?) #=> false
#[wat_intrinsic(":wat::kernel::sigusr1?")]
pub(crate) fn eval_kernel_sigusr1(
    env: &Environment,
    sym: &SymbolTable,
    list_span: &Span,
) -> Result<Value, EvalBreak> {
    crate::runtime::eval_user_signal_query(
        &[], ":wat::kernel::sigusr1?", &crate::runtime::KERNEL_SIGUSR1, list_span,
    )
}
```

The three delegate fns are private in `runtime.rs` today — widen them to `pub(crate)` and nothing
else. `env`/`sym` may be unused by these bodies; follow whatever `kernel_stdio.rs` does for a
delegate that ignores a param, and if the macro requires the params, keep them.

## The declaration table — RE-DERIVE IT, then tell me whether you agree

The stone predicts the following. **Read each body and reach your own answer first.** If you agree,
say you agree and why. If you disagree on any row, say so with the body line that decides it — a
dissent with a citation is worth more to me than assent.

| verb | Purity | Determinism | Category |
|---|---|---|---|
| `stopped?` `sigusr1?` `sigusr2?` `sighup?` | Pure | Nondeterministic | Ambient |
| `reset-sigusr1!` `reset-sigusr2!` `reset-sighup!` | Effectful | Deterministic | Ambient |

The governing precedent for "a read of ambient state is Pure": `:wat::time::now`
(`src/intrinsic/time.rs:61–70`) is `@Purity Pure` + `@Determinism Nondeterministic` and reads the wall
clock. Twenty registered rows sit in that quadrant.

## ★★ THE PREDICTED RED — this is the stone's RESULT, not a failure

`pure_declared_matches_is_effectful_op` (`src/intrinsic/mod.rs:544`) asserts a **biconditional**
between the declared `@Purity` and `runtime::is_effectful_op` — which classifies by **prefix**, and
prefix-matches every `:wat::kernel::` verb as effectful.

**The four readers declare `Pure`. The gate will go RED on exactly those four.**

That is expected, it is written into the design ahead of time, and **it is what this stone was drawn
to measure.** When it fires:

- capture the failing test's **whole stdout+stderr block verbatim** — never a summary, never a
  `| head` window
- name which of the four verbs the assertion fired on, and in what order
- **stop there and report.** Do not proceed to make it green.

If it comes back **green**, that is a bigger finding than the red — it means my reading of either
`is_effectful_op` or the gate is wrong. Say so plainly and show me the run.

## Blast radius

```
NEW      src/intrinsic/kernel_ambient.rs
EDIT     src/intrinsic/mod.rs        one `mod kernel_ambient;` line
EDIT     src/runtime.rs              delete 7 literal arms (+ the replacement comment);
                                     widen 3 delegate fns to pub(crate)
```

**Nothing else.** No new types. No edit to `wat/runtime-meta.wat`, `src/check.rs`,
`crates/wat-doc/`, `crates/wat-macros/`, or any test.

## STOP triggers — every one of these means SHIP NOTHING FURTHER AND REPORT

1. **STOP-1 — the purity collision.** Described above. Declare from the body, let the gate go red,
   capture it verbatim, stop. **Do not** change a `@Purity` to satisfy the gate. **Do not** touch
   `is_effectful_op`. **Do not** weaken, `#[ignore]`, or narrow the gate's assertion.
2. **STOP-2 — routing changed.** If registering a verb changes what actually runs — a different
   delegate, a different arg path, a behavioural difference in any test that exercises signals
   (`wat-tests/process/signal-*.wat`, `wat-tests/service-signal-observer.wat`) — stop. Registration
   moves the *lookup*, never the *handler*.
3. **STOP-3 — a `@ret` spelling disagrees with the registered `TypeScheme`.** Report the pair
   (`check.rs` line, your `@ret`) and stop. Do not edit `check.rs` to match your doc.
4. **STOP-4 — you cannot derive a verb's `@Category` as `Ambient` from its body.** `:Ambient`'s prose
   names all seven, but the prose is a claim and your body-read is the check. If a body says
   otherwise, that is a finding — report it, do not stretch to fit.
5. **STOP-5 — the blast radius above is insufficient.** If the work structurally requires editing a
   file not on that list, stop and tell me which file and why. Do not widen scope on your own; the
   last stone's rider hit exactly this and was right to stop.

## What "done" looks like

- `cargo build --release` exits 0 (a `compile_error!` from a missing `@Category` fails the build, so
  a green build already proves every row is annotated)
- `cargo nextest run --release -E 'test(/intrinsic::tests::/)'` has run, and you report its **full
  Summary line** plus, for any failure, the complete verbatim block. The five tests in that module
  are `all_see_fqdns_resolve_to_registered_intrinsics`, `doc_arg_ret_types_match_checker_scheme`,
  `purity_mandated_examples`, `pure_declared_matches_is_effectful_op`, `yields_type_matches_fn_arg_param`
- your **declaration table**, with agreement or dissent per row and the deciding body line
- the honest deltas: what surprised you, what you inspected that the brief did not send you to

Runtime band: 35–55 minutes, most of it two release builds.
