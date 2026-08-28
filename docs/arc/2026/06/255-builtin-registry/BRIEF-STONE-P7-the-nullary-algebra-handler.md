# STONE P7 — `sniff_kind` cannot see a NULLARY ALGEBRA handler

> The remainder of O-iv-d. **Eleven verbs are blocked by the generator, not by themselves.**
> Read `BRIEF-STONE-O-iv-d-the-remainder.md` for the wave and the four-valued disposition axis.

## The defect, on disk

`crates/wat-macros/src/wat_intrinsic.rs:201`:

```rust
let is_algebra = matches!(
    item.sig.inputs.iter().next(),
    Some(FnArg::Typed(pt)) if is_ref_value(&pt.ty) || is_ref_value_slice(&pt.ty)
);
```

`inputs.iter().next()` on a fn with **no** `&Value` first param is never `is_algebra`. So a handler
with **zero params**, or one whose **only** param is `&Span`, falls to BINDING — and BINDING's emit
arm (`:726`) then writes:

```rust
#fn_name(#(#arg_forwards,)* env, sym, list_span)     // UNCONDITIONAL. All three, always.
```

…into a fn taking nothing. Eleven identical `E0061`s. O-iv-d's rider hit exactly this, held STOP-2,
reverted all eleven, and reported it. **And `emit` already believes the shape is legal** — its own
comment at `:776` calls "a nullary ALGEBRA fn taking only a span" *"a legal, if unusual, shape"*,
guarding a `call_args` build that can never run.

## ★ THE CONTRACT DECISION — pin this exactly

**The predicate is not "does this signature START with `&Value`". It is "can this signature be
BINDING AT ALL".**

Derive it from `:726`, which is the whole argument: BINDING passes `env`, `sym`, `list_span`
unconditionally, so **every** BINDING handler has at least three params. Therefore:

| shape | BINDING form? | so |
|---|---|---|
| `fn f()` | ❌ — `f(env, sym, span)` is `E0061` | **must be ALGEBRA** |
| `fn f(span: &Span)` | ❌ — same `E0061` | **must be ALGEBRA** |

This is **not a heuristic that might reclassify something legitimate.** Neither shape has a BINDING
form that compiles today, so nothing that compiles today can move. The compiler is the control.

```rust
let is_algebra = match item.sig.inputs.iter().next() {
    // A handler with no params — or whose ONLY param is the call span — has NO BINDING
    // form: `emit`'s Binding arm passes `env, sym, list_span` unconditionally, so such a
    // signature cannot compile as BINDING. Classifying it ALGEBRA is the only reading
    // under which it compiles at all (arc 255 Stone P7).
    None => true,
    Some(FnArg::Typed(pt)) => {
        is_ref_value(&pt.ty)
            || is_ref_value_slice(&pt.ty)
            || (is_ref_span(&pt.ty) && item.sig.inputs.len() == 1)
    }
    // A `self` receiver stays false so `sniff_args` rejects it with its own real message.
    Some(FnArg::Receiver(_)) => false,
};
```

**Everything downstream is already correct — verify, do not change it:**
- `sniff_kind`'s loop (`:248`) sets `span_seen` from the `is_ref_span` arm **regardless of
  position**, so a sole `&Span` yields `Algebra(Exact([]), true)` and a bare `fn f()` yields
  `Algebra(Exact([]), false)`.
- The value door (`:769`) builds `call_args` as ONE list precisely so `n == 0` emits no dangling
  comma. The AST door (`:815`) guards `args.len() != 0`, evaluates nothing, and calls the value door
  with an empty slice.

## The work — part 1, THE MACRO ALONE (the control)

Change **only** the predicate. Then `cargo build --release --all-targets`.

★ **The tree must still compile, unchanged, with no handler behaving differently.** That is the
control that proves the predicate did not widen past the two impossible shapes. **If anything at
all breaks here, STOP** — the derivation above is wrong and the stone must be redrawn.

## The work — part 2, the eleven verbs

```
:wat::uuid::v4 · :wat::uuid::nil                                         src/intrinsic/uuid.rs
:wat::time::now                                                          src/intrinsic/time.rs
:wat::math::pi                                                           src/intrinsic/math.rs
:wat::kernel::stopped? · sigusr1? · sigusr2? · sighup?                   src/intrinsic/kernel/ambient.rs
:wat::kernel::reset-sigusr1! · reset-sigusr2! · reset-sighup!            src/intrinsic/kernel/ambient.rs
```

Every one is today a three-param shim forwarding to a `crate::runtime::eval_*` with `&[]`:

```rust
 #[wat_intrinsic(":wat::math::pi")]
 pub(crate) fn eval_math_pi_intrinsic(
-    _env: &Environment, // rune:lint(unused-env) — `pi` takes no wat-facing args
-    _sym: &SymbolTable, // rune:lint(unused-sym) — see above
     span: &Span,
 ) -> Result<Value, EvalBreak> {
     crate::runtime::eval_math_pi(&[], span)
 }
```

**Which of the two shapes each verb takes is the verb's own business, and I have NOT decided it for
you.** Keep the `&Span` where the body genuinely uses it; drop it where the param is already
`_span`. Both shapes must appear among the eleven — **report the split**, because a stone that
exercises only one shape has proven only half the fix.

⚠ **`uuid::v4`, `time::now` are nondeterministic and the three `reset-*!` MUTATE global signal
flags.** Migration grants no new capability — each is already callable directly — but say you
noticed, and check each `@Purity`/`@Determinism` still reads correctly afterward.

## STOP triggers — each REJECTS. Ship nothing on that row and report the gap.

1. **Part 1 breaks anything.** The predicate widened past the impossible shapes. Do not narrow it by
   hand and continue — report it; the derivation is the thing that failed.
2. **A verb turns out to need `env`/`sym` for real, or reads `<arg>.span()`.** It is BINDING. Leave
   it, name it, say which.
3. **Any behaviour changes** — value or error text, direct or through `apply`.
4. **You find yourself editing `emit`'s Binding arm, or adding a marker attribute/argument.** Both
   are out of scope by affirmative cut: the derivation makes a marker unnecessary, and BINDING's
   emit is correct as written. If the work seems to need either, the design is wrong — STOP.

## Acceptance — every bar derived from a measurement, not from what I expect

```
 0. ★ YOUR OWN READ of the eleven BEFORE migrating. Every disagreement with my list reported.
 1. ★ PART 1 ALONE COMPILES THE WHOLE TREE UNCHANGED. The control. Report the command and result
      before you touch a single verb.
 2. ★ (apply :wat::math::pi []) ANSWERS.
      BEFORE, verbatim, measured this session at HEAD a085cb172:
        #wat.runtime/NotValueDispatchable {:message ":wat::math::pi is registered, but no handler
        taking EVALUATED arguments is registered under that name, and apply dispatches with
        evaluated arguments. Call it directly." :name ":wat::math::pi"}
      AFTER: the value of pi.
 3. ★ ALL ELEVEN REACH apply — one scratch .wat, `--check` clean, each verb before and after.
      Include one effectful verb and one `reset-*!`.
 4. ★ BOTH SHAPES EXERCISED. Report which verbs became `fn f()` and which `fn f(span: &Span)`.
 5. ★ DIRECT CALLS BYTE-IDENTICAL for all eleven, before and after, diffed.
      `git show HEAD:<path>` for the pre-image — never `git stash`.
 6. ★ THE RUNE LEDGER FALLS BY EXACTLY ELEVEN, BOTH KINDS. Derived from the per-file census:
      uuid 2 + time 1 + math 1 + ambient 7 = 11 removed; resource 1 + string 1 + source 3 = 5 stay.
        grep -rc --include=*.rs 'rune:lint(unused-env)' src/ | awk -F: '{s+=$2} END {print s}'
      BEFORE 16 → AFTER 5.  Same command for `unused-sym`: BEFORE 16 → AFTER 5.
      A different number is a FINDING, not a rounding — say which file disagreed.
 7. ★ REGISTRY POPULATION UNCHANGED at 380.
        grep -rcE --include=*.rs '^[ \t]*#\[wat_intrinsic' src crates | awk -F: '{s+=$2} END {print s}'
 8. ★ PURITY/DETERMINISM INTACT. (:wat::runtime::metadata-of <verb>) for uuid::v4, time::now,
      and one reset-*! — before and after, and say what each declares.
 9. cargo build --release --all-targets — clean; report any warning VERBATIM.
10. cargo nextest run --release -E 'test(intrinsic) + test(apply) + test(uuid) + test(kernel) + test(math)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing will wake you, and there is no
  notification coming. Your turn ends when the numbers are in your hands, not when a command starts.
- `cargo build`, scoped `cargo nextest`, `./target/release/wat` — yes. **The full floor and clippy
  are the orchestrator's**; do not run them.
- **You may not spawn sub-agents.**
- No `git stash`, in any form. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean. Not the session scratchpad.
- ⚠ Check that your own added prose does not contain the literal pattern you are grepping for —
  three riders tripped their own acceptance grep on their own comments in one day.

## Report back with

Row-by-row: the command, its actual output, PASS/FAIL. Your own read of the eleven with every
disagreement. The shape split from row 4. What you noticed about the effectful verbs. Then the
honest deltas — what surprised you, and anything you could not measure.
