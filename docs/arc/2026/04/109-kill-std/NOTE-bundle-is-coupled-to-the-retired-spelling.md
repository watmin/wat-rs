# NOTE — `:wat::holon::Bundle` recognition is coupled to the RETIRED spelling, and is now unreachable in both

> Builder, 2026-08-30, reading the annihilate-the-prose rider's "do not convert" comment:
> *"hrm..... Bundle may be flawed....."*
>
> **It is.** Measured on the shipped binary after the wall. **Nothing drawn** — this records the
> defect and the fix.

## What the code requires

`src/holon/ast.rs:1131`, `is_holon_arg_canonical`:

```rust
":wat::core::Vector" => {
    items.len() >= 2
        && matches!(items[1], WatAST::Keyword(_, _))     // ← a BARE type keyword
        && items[2..].iter().all(is_holon_arg_canonical) // ← elements start at [2]
}
```

Fed the canonical `:- [T]` spelling, `items[1]` is the `:-` keyword — which still matches — but
`items[2]` is the bracket `WatAST::Vector`, and the predicate has **no `Vector` arm**, so it falls to
`_ => false`. The Bundle then does not fire as one step.

`src/lower.rs:243` carries the identical dependency in its own words: *"Expect exactly one argument:
a `(:wat::core::Vector :T item ...)` form"* — and its test at `:397` is written in the bare spelling.

## ⛔ THE DEFECT: BOTH SPELLINGS NOW FAIL, FOR OPPOSITE REASONS

```wat
;; BARE — the shape the predicate wants
(:wat::holon::Bundle (:wat::core::Vector :wat::holon::HolonAST (:wat::holon::Atom "a") …))
  -> "malformed :wat::core::vec form: first argument must be a `(Head :- [T])` type param-spec"
     REJECTED BY THE WALL. Unwritable in source.

;; CANONICAL — the only spelling a user may now write
(:wat::eval-step! (:wat::core::quote
  (:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST] (:wat::holon::Atom "a") …))))
  -> #wat.core/EvalError {:kind "no-step-rule"
                          :message "eval-step! has no rule for op: :wat::core::Vector"}
     NOT RECOGNIZED. The single-step path never fires.
```

**So the Bundle single-step optimization is dead for every program a user can actually write.** The
spelling it recognizes cannot be written; the spelling that can be written is not recognized.

## ★ AND ITS TESTS ARE GREEN ON AN UNREACHABLE INPUT

The tests that exercise this path feed AST directly — `quote` / `eval-step!` / a Rust string handed
to the stepper — so they never pass through the checker that now forbids their shape. **They assert
correct behaviour on a form no source program can produce.**

That is why the wall did not turn them red, and why the annihilate rider's empirical check
("converting this literal to `:- [...]` turns this test red") was *true* and still pointed the wrong
way: the red was not evidence the bare form must be kept — it was evidence the predicate never
learned the canonical one. **The comment it left is accurate about the mechanism and wrong about the
conclusion**, and it should be replaced rather than preserved.
`[[feedback_a_probe_answers_the_question_you_asked_not_the_one_you_meant]]`

## The fix, and it is small

Teach the two sites the canonical shape — **peel the param-spec instead of assuming a bare keyword**:

- `is_holon_arg_canonical` — use `peel_param_spec` (`src/types.rs:4793`) on `items[1..]`, then require
  the remaining elements to be canonical. One call; the same helper the checker uses.
- `lower_bundle` (`src/lower.rs:243`) — the same peel, and update its `:397` test to the canonical
  spelling.

★ **Then the test earns its green**: it will assert the single-step path on a form a user can
actually write, which it has never done.

⚠ **Do not "fix" this by re-admitting the bare form.** The wall is the ruling; this is a consumer
that never learned it. And do not delete the tests — they cover a real optimization whose only
defect is that it was written against a spelling that has since been retired.

## What is NOT established here

Whether anything else recognizes a constructor by positional-argument shape and would break the same
way. `is_holon_arg_canonical` and `lower_bundle` are the two this NOTE measured because the rider
named them. **A sweep for other `items[1]`-is-a-Keyword assumptions across the holon and stepper
paths is the obvious next question, and it is not answered here.**
