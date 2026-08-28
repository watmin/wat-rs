# STONE O-i — the value door gets the arity guard it never had

> Read `DESIGN-STONE-O-one-declaration-feeds-both-doors.md` first — its ⛔ CORRECTION banner and
> "THE SECOND DEFECT" section carry this strike's whole motivation.

## The work

`:wat::core::apply` reaches 44 registered verbs through `dispatch_substrate_impl`, which calls their
`value_handler` **with no arity check at all**. Every value handler opens with
`vals.first().expect("arity-checked")` — naming a check that happens only on the OTHER door. So a
wrong-arity `apply` **kills the process**, where the identical wrong-arity direct call returns a
clean error. Measured on today's tree:

```
(:wat::i64::+ 20)                        →  err ":wat::i64::+: expected 2 args, got 1"   ← AST door
(:wat::core::apply :wat::i64::+ [20])    →  PANIC  src/runtime.rs:11605  "arity-checked"  ← value door
```

You give `dispatch_substrate_impl` the guard, **in one place, from the registry's own record**. The
registry already knows every verb's arity (`IntrinsicEntry::arity`, populated by the
`#[wat_intrinsic]` macro from the handler signature). Nothing else changes.

★ **One guard, not 25 patches.** There are 25 unchecked-index sites across 5 intrinsic files plus the
shared `arith_{i64,f64,bigint,rational}_*_inner` fns. **Do not touch any of them.** Guarding the one
door they are all reached through makes every one of them unreachable-while-unguarded — the class
goes, not the instances. `.expect("arity-checked")` becomes true where it is written.

## Rooms — verified against `fe602d707`

```
src/runtime.rs:11561   dispatch_substrate_impl   — the seven lines you are changing. The WHOLE strike.
src/runtime.rs:11605   arith_i64_i64_inner       — where the panic actually fires; DO NOT EDIT IT
src/intrinsic/mod.rs:366  lookup_entry(name) -> Option<&IntrinsicEntry>   — gives you arity + handler
src/intrinsic/mod.rs:288  IntrinsicEntry::value_handler                    — what lookup_value returns
src/intrinsic/mod.rs:144  enum Arity { Exact(usize), Variadic }            — the record to read
src/value/signal.rs:202   RuntimeErrorKind::ArityMismatch { op: String, expected: usize, got: usize }
crates/wat-macros/src/wat_intrinsic.rs:545   the AST door's ArityMismatch — MATCH THIS EXACTLY
```

## Implementation sketch

```rust
pub(crate) fn dispatch_substrate_impl(
    impl_name: &str,
    vals: &[Value],
) -> Option<Result<Value, EvalBreak>> {
    let entry = crate::intrinsic::registry().lookup_entry(impl_name)?;
    let handler = entry.value_handler?;
    if let Arity::Exact(n) = entry.arity {
        if vals.len() != n {
            return Some(Err(/* ArityMismatch { op: impl_name.into(), expected: n, got: vals.len() } */));
        }
    }
    Some(handler(vals))
}
```

⚠ **`Some(Err(…))`, never `None`.** Returning `None` on an arity mismatch drops through to
`eval_apply`'s step (d) and reports **"unknown function"** for a verb that plainly exists — trading
the panic for the OTHER lie this arc is killing. The verb was found; only its arity was wrong, and
the error must say so.

⚠ **`Arity::Variadic` takes no check.** Zero or more args are all valid for a variadic verb, exactly
as the generated AST shim already treats them (`wat_intrinsic.rs:531`, no arity check on the variadic
branch). Measured: **all 44 verbs carrying a value door today are `Exact(N)`**, so the Variadic arm
is forward-compatible cover, not dead code — Stone O-iii will register variadic algebra.

⚠ **The span.** `dispatch_substrate_impl` has no `Span` parameter. Use whatever the neighbouring
value-path errors use (`crate::rust_caller_span!()` is what every `*_inner` fn raises with); **do not
add a `Span` parameter to thread a better one through** — that widens the strike into every caller
and the span question is explicitly out of Stone O's scope. If you cannot raise the error without a
new parameter, that is STOP-3.

## Blast radius

`src/runtime.rs`, the body of `dispatch_substrate_impl`. Nothing else. No macro change, no intrinsic
file change, no `eval_apply` change, no new registry field, no signature change to any public fn.

## STOP triggers — each REJECTS. Ship nothing and report the gap.

1. **The guard changes any RIGHT-arity behaviour.** All 44 verbs called with correct arity through
   `apply` must return exactly what they return today. If any differs, STOP.
2. **The arity error text differs from the direct call's.** Same `RuntimeErrorKind::ArityMismatch`,
   same `op` string (the verb's FQDN), same `expected`/`got`. If you cannot produce byte-identical
   text, STOP and report what differs — a different error on the value door re-creates the split
   this stone exists to close.
3. **You need a new parameter on `dispatch_substrate_impl`.** STOP and report. Its callers are out of
   this strike's blast radius.
4. **Any `expect("arity-checked")` site tempts you.** Do not edit one. If you believe a site is
   reachable-while-unguarded even after the door is guarded, that is a real finding — STOP and name
   the path, do not patch the site.

## Acceptance — run each, report the actual output

```
 0. ★ THE PANIC IS GONE, AND THE PROBE SAYS SO ITSELF.
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-the-value-door-panics-on-arity.wat
    That probe's header states its own acceptance: today it dies on row 3; after this strike it
    must PRINT row 3 and EXIT 0, with row 3's text matching row 1's. Paste all three rows + the
    exit code. Do not edit the probe.

 1. ★ THE SECOND PANIC IS GONE TOO — the one in a different file, so the fix is proven CENTRAL
    and not incidental to i64:
      (:wat::core::apply :wat::vector::concat [<one PersistentVector>])
    panics at src/intrinsic/vector.rs:214 today. It must now return ArityMismatch. Write the
    probe as a scratch .wat under wat-scripts/scratch-pad/ (that directory is loader-gated, so
    it must `--check` clean); do not put it in /tmp.

 2. ★ PROVE THE GUARD IS WHAT DID IT — BY BREAKING ITS DOOR. Comment out the arity check, re-run
    row 0, show the panic returns, restore. `NISI FRANGAS, NIHIL PROBAS.` Confirm the edit LANDED
    before reading its output: a no-op edit prints a meaningless green.

 3. ★ NO RIGHT-ARITY BEHAVIOUR MOVED.
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-apply-lies-about-what-exists.wat
      ./target/release/wat wat-scripts/scratch-pad/255-stone-o-apply-has-three-broken-doors.wat
    Every row of both must be IDENTICAL to today's output. Both files record today's expected
    output in their headers. Paste both runs.

 4. ★ A MISMATCH SAYS "WRONG ARITY", NOT "UNKNOWN FUNCTION". Show the error kind for the row-0
    case is ArityMismatch — not UnknownFunction. This is STOP-2's positive form and it is the
    row most likely to pass for the wrong reason.

 5. cargo build --release --all-targets — clean.

 6. cargo nextest run --release -E 'binary_id(wat::wat_lang)' and any test naming apply or arity.
    Report the Summary line verbatim.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. Verify with `pwd` first. Any path containing
  `.claude/worktrees/` is harness state — never operate on it.
- Everything in the FOREGROUND. Your turn ends when the numbers are in your hands, not when a
  command is launched.
- You may run `cargo build`, `./target/release/wat --check`, `./target/release/wat <file>`, and a
  scoped `cargo nextest run --release -E '<filter>'`. The orchestrator runs the full floor and
  clippy centrally — leave those alone.
- You may not spawn sub-agents.
- Do not commit, push, stash, revert, or create a worktree. Leave the tree dirty; the orchestrator
  weighs and commits.
- If a number surprises you, report the surprise. The last four stones in this arc were each
  corrected by a rider catching a defect in my own brief, and one refused an order that would have
  deleted live code. A brief that turns out to be wrong is the most useful thing you can hand back.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Then the honest deltas — what surprised you,
what this brief got wrong, what you had to decide that it did not settle.
