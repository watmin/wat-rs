# STONE Q-2 — the threaded span must be USED

> **Q's plumbing is on the tree and the floor is RED.** Two existing lints fire on it, both saying
> the same thing, and they are right. Q-2 is not a follow-up I chose — **it is the shape the
> substrate's own walls insist on.**

## What happened

Stone Q widened `ValueHandler` to `fn(&[Value], &Span)` and threaded `apply`'s call span to the
value door. Its brief said the stone was plumbing only: *"Q threads a span that is currently dropped.
It does not yet let anyone USE it. Every existing diagnostic must be byte-identical."*

**That state is not reachable.** Two lints refuse it:

```
unused_span_justified          19 sites   `_span: &Span` — a span in hand, dropped
span_substitution_justified     1 site    src/runtime.rs:11629, dispatch_substrate_impl —
                                          mints rust_caller_span!() while a real wat span is in scope
```

And both lints explicitly rule out the escape:

> *"`rust_caller_span!()` does NOT earn standing: a Rust line is the HARM this lint names, not a
> location. A site that ignores its span AND raises at a Rust line is a FIX."*

★ **The plumbing and the use cannot be separated, because the substrate has a wall against exactly
that separation.** Q's own STOP-4 asked for a state the codebase forbids. That is the wall doing its
job, and the honest response is to finish the stone, not to rune around it.

⚠ **The `span_substitution` site is Stone O-i's own arity guard.** O-i reached for
`rust_caller_span!()` there because no span existed — correctly, at the time. Q gave it one. The lint
noticed within the same floor run.

## The work

**1. `dispatch_substrate_impl` (`src/runtime.rs:11629`)** — its `ArityMismatch` currently raises at
`rust_caller_span!()`. It now has `span: &Span`. Use it. A wrong-arity `apply` should point at the
call site in the user's `.wat`, not at a line in `runtime.rs`.

**2. The four shared arithmetic helpers** (`src/runtime.rs`) —
`arith_i64_i64_inner` · `arith_f64_f64_inner` · `arith_bigint_bigint_inner` ·
`arith_rational_rational_inner`. Each raises `TypeMismatch` / `DivisionByZero` /
`IntegerOverflow` at `rust_caller_span!()`. Give each a `span: &Span` and use it.

**3. The 19 hand-written value twins** — `i64.rs` ×7, `f64.rs` ×4, `bigint.rs` ×4, `rational.rs` ×4,
at the lines the lint names. Each currently takes `_span: &Span` and drops it. Rename to `span` and
pass it to its helper.

## ⚠ THIS CHANGES DIAGNOSTICS — that is the deliverable, and it must be shown

Q promised byte-identical diagnostics. **Q-2 explicitly breaks that promise**, in one direction only:
errors raised through the value door move from a `src/*.rs` line to the user's call site. **Every
such change must be exhibited, not just asserted** — see acceptance row 1.

If any diagnostic moves in the *other* direction — a real wat span replaced by a Rust one, or a span
that is now WRONGER than before — that is STOP-2.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **A site's honest disposition is a RUNE, not a FIX.** Possible: a helper may raise on a value with
   no truthful user location. If you find one, do NOT force the span in — write the rune with the
   reason the lint demands (it must say WHERE the error is located instead, and `rust_caller_span!()`
   does not qualify). Name every one.
2. **A diagnostic gets worse.** Any error that previously carried a real wat span and now carries a
   Rust line, or points at the wrong form. STOP.
3. **The AST door's diagnostics change.** Only the value door was span-less. `(:wat::i64::+ 1 "x")`
   called directly must be byte-identical; only `(apply :wat::i64::+ [1 "x"])` should improve.
4. **You touch the generated ALGEBRA value door's arity check.** The macro already raises it with the
   span it was handed; if it does not, that is a finding — report it, do not patch around it.
5. **Either lint still fires, or needs its allowlist widened.** These two lints are the reason this
   stone exists. Adding an exemption to silence them inverts the stone.

## Acceptance — run each, report the actual output

```
 0. ★ BOTH LINTS GREEN, WITH NO ALLOWLIST WIDENED.
      cargo nextest run --release -E 'test(unused_span_justified) + test(span_substitution_justified)'
    Summary verbatim, plus `git diff` on both lint files (expect EMPTY — you fix the sites, not the
    lints).

 1. ★ EXHIBIT EVERY MOVED DIAGNOSTIC. A scratch .wat under wat-scripts/scratch-pad/ (`--check`
    clean) that triggers, through `apply`, one error from each affected family: an i64 type
    mismatch, an i64 overflow, a division by zero, an f64 mismatch, a rational mismatch, a bigint
    mismatch, and a wrong-arity call. Show BEFORE (build the pre-image with `git show HEAD:<path>`,
    never `git stash`) and AFTER for each. Before: a `src/*.rs` location. After: the caller's
    `.wat` line:col. Paste both.

 2. ★ TWO CALL SITES, TWO SPANS. For at least one of those errors, trigger it from two different
    lines in one file and show the reported location differs. An error that reports the same
    location from both is still synthesized.

 3. ★ THE AST DOOR IS UNTOUCHED. The same errors raised by DIRECT calls, before and after,
    byte-identical. STOP-3's positive form.

 4. ★ THE COMMITTED PROBES STILL AGREE. Re-run and diff:
      255-stone-o-apply-lies-about-what-exists.wat
      255-stone-o-apply-has-three-broken-doors.wat
      255-stone-o-the-value-door-panics-on-arity.wat
      255-stone-o-i-vector-concat-value-door-panic.wat
      255-stone-o-iv-b-collections-sweep-apply.wat
    ⚠ The last two DO exercise value-door arity errors and are EXPECTED to change — exhibit those
    changes as row-1 evidence rather than treating them as regressions. The other three should be
    byte-identical. Say which is which and why.

 5. cargo build --release --all-targets — clean.

 6. cargo nextest run --release -E 'binary_id(wat::lint) + test(intrinsic) + test(apply)'
    Summary verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- **The tree is already dirty with Stone Q's plumbing, and the floor is RED because of it. Build on
  that tree; do not revert it.** Your job finishes what it started.
- Everything FOREGROUND. Ending your turn ENDS you; nothing wakes you. Land the numbers.
- You may run `cargo build`, `cargo nextest run --release -E '<filter>'`,
  `./target/release/wat --check <file>` and `./target/release/wat <file>`. The orchestrator runs the
  full floor and clippy centrally.
- You may not spawn sub-agents.
- **No `git stash`, in any form.**
- Do not commit, push, revert, or create a worktree.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. The full list of moved diagnostics with their
before/after locations. Every site you runed instead of fixed, with its reason. Then the honest
deltas.
