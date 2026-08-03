# BRIEF — apply the TCO pattern to `and`, `or`, `ann-form`

Task **#59**. Home arc for the root fix is **261** (CEK); this is the near-term patch the builder
ruled on 2026-08-02. Rows 2 and 3 of that discussion (relocating the `setrlimit`, CEK itself) are
**affirmatively cut** — see task #58. This is the whole of the work.

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in
the FOREGROUND and block on it. Your turn ends when the numbers are in your hands.

## The work, in one paragraph

`eval_tail` (`src/runtime.rs:3807`) dispatches `if` / `match` / `let` / `do` (+
`kernel::serve-dispatch-op`) to their `*_tail` TCO variants. Three forms that create a tail context
are missing from it: **`and`**, **`or`**, **`ann-form`**. Add `eval_and_tail`, `eval_or_tail`,
`eval_ann_form_tail` and their arms, mirroring `eval_if_tail`. This is not a workaround — it is the
identical mechanism the other four already use.

## Already measured — take as given, do not re-derive

Pinned binary, exit codes captured directly (never through a pipe), all at depth 150000, with `do`
(already in the list) as the same-shape control returning exit 0:

```
tail self-call under `if`   2,000,000 deep -> exit 0   TCO works when present, flat
and       (last operand)       150,000    -> SIGSEGV 139
or        (last operand)       150,000    -> SIGSEGV 139
ann-form  (wrapped expr)       150,000    -> SIGSEGV 139
```

Ruled out by grounding, not assumption: `:wat::core::try` is RETIRED (a tombstone arm, Stone
241.15); `:wat::core::when` DOES NOT EXIST; `cond` is a defmacro expanding to `if` and inherits.

## Why each is a legitimate tail position

- **`ann-form`** — a type ascription returns its wrapped expression's value untouched. A pure
  pass-through; TCO here is observationally free.
- **`and` / `or`** — only the **LAST** operand (earlier ones are tested, not returned). Legal
  because every operand is type-forced to bool, so `(and …all-true… last)` is value-identical to
  `last` — `eval_and`'s trailing `Ok(Value::bool(true))` reconstructs exactly what the last operand
  already produced.

## ★ THE ONE DECISION, ALREADY RULED — implement it, do not re-open it

`eval_and` (`runtime.rs:9600`) type-checks **every** operand at runtime and raises a located
`TypeMismatch` on a non-bool. **Tail-calling the last operand necessarily skips that check on that
operand** — you cannot inspect a value you have tail-called away. There is no shape that keeps both;
this was weighed.

**RULED: take the TCO, and make the weakening VISIBLE rather than silent.** Rationale: the checker
already forbids a non-bool operand (`infer_boolean_shortcircuit`), so in all checked code the
behaviour is identical; the runtime check is belt-and-braces that duplicates a checker guarantee.
The first N−1 operands keep their check.

**But this arc's law is that nothing weakens quietly**, so the ruling comes with two obligations:

1. **Document it at `eval_and_tail`/`eval_or_tail`** — state plainly that the last operand's runtime
   bool check is traded for TCO, that the checker is the real guard, and that an unchecked eval path
   (`:wat::eval-ast!` runs with no type-check pass — "trust-the-caller") is where the difference is
   observable.
2. **Pin the new behaviour with a test**, so it is recorded rather than discovered later. If you can
   drive a non-bool last operand through an unchecked path, assert what it now does. **If you cannot
   construct that path, say so in your report and do not fake it** — an unreachable difference is
   worth knowing about too.

## Read these rooms, in order

1. `src/runtime.rs:3807-3840` — `eval_tail`, its arms, and the rete gate added by #56. Your new arms
   go in this match.
2. `src/runtime.rs:9600` — `eval_and`. Note it takes `args: &[WatAST]` (**unevaluated forms**) and
   calls `eval_inner` on each; that is *why* the frame is here and why this fn owns the choice.
3. `eval_or` — its sibling, immediately adjacent.
4. `eval_if_tail` — **the exemplar**. Copy its shape.
5. `eval_let_tail` — **the wrinkle**: it returns a `TrackedValue` where the others return `Value`
   (its `eval_tail` arm unwraps with `.map(|tv| tv.value_owned())`). Check each mirror's return
   shape rather than assuming a uniform copy-paste.
6. `ann-form`'s runtime arm — locate it yourself; `check.rs:2998` has its inference arm as a
   starting point. **If `ann-form` turns out not to have a distinct runtime arm, or is erased before
   eval, STOP-2.**

## Blast radius

`src/runtime.rs` and one test file. **No `wat/` files, no `crates/`, no new types, no checker
changes.** The rete vocabulary is NOT involved — these are core forms.

## STOP triggers — rejection criteria. Ship nothing for that form; report the gap.

1. **STOP-1 — the tail handoff changes an answer.** If any existing test changes its result, halt.
   TCO must be observationally identical except for the documented last-operand check.
2. **STOP-2 — `ann-form` has no distinct runtime arm** (e.g. it is erased at expansion). Then it
   cannot lose TCO the way the probe suggested; report what you found and land the other two.
3. **STOP-3 — `and`/`or` short-circuit breaks.** Their existing laziness is load-bearing and already
   gated in `tests/rete/`. If preserving short-circuit and TCO together is not possible in the same
   fn, halt and report.
4. **STOP-4 — the `_` wildcard on an enum scrutinee is doctrine-illegal.** Name every variant.
5. **STOP-5 — scope.** Do NOT touch the `setrlimit` in `src/distribution/mod.rs`, do NOT add a
   recursion-depth guard, do NOT start CEK. All three are ruled out (#58).

## Gates — foreground, report every result line

```
cargo build --release --all-targets       # exit 0, ZERO warnings
cargo clippy --release --all-targets      # likewise
cargo test --release --test rete          # 225/0/9 at HEAD
cargo test --release --test lint          # 66/0
```

**Do NOT run `cargo nextest run`** — the orchestrator weighs the whole floor centrally once your
tree is quiescent. A narrow filtered `cargo test --release --test <target> -- <filter>` is expected.

### The gate that decides whether this shipped

**Each form needs a TCO test that goes RED without its arm.** Depth 150000 (the measured breaking
point; a smaller number TCO does not need proves nothing). For each of the three: land the arm,
write the test, then **remove the arm, watch the test die, put it back** — and report both
observations. A green TCO test with no red-without-it observation is the vacuous-gate class this arc
has hit five times.

Note the failure shape differs by context: under `cargo test` it surfaces as **SIGABRT** ("fatal
runtime error: stack overflow" — the test thread's guard page is intact), while the CLI shows a bare
**SIGSEGV**. Same class, different signal; do not treat the difference as a finding.

## Two lint traps that have bitten twice in this arc

- A doc comment or assert message that **parses as a wat list** trips `no_inlined_wat_in_tests`
  (the literal `"(not false)"` did it).
- A `contains(...)` on a rendered error trips `no_loose_string_assert` — match the typed
  `RuntimeErrorKind` instead.

Fix both at the root. **Do not add a `rune:lint` to silence either.**

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not add `#[allow(dead_code)]`
to silence a signal — if something has no reader, say so in your report.
