# NOTE — "is this call kwargs or positional?" is decided in FIVE places

**Filed 2026-08-02**, surfaced by arc 278 Stone A (the `:then` migration), which had to hand-fix
three of the five and whose brief named only one. Arc 109's own "one door" pattern
(`BRIEF-runtime-error-one-door.md`, `BRIEF-typeerror-loaderror-one-door.md`) is the home for this.

## The measurement

One language fact — *given an aggregate call's argument list, is it **kwargs** or **positional**?* —
is written out five times:

| # | site | phase | on a malformed form |
|---|---|---|---|
| 1 | `check.rs:11577` `infer_kwargs_construct_check` | check | a `CheckError` |
| 2 | `runtime.rs:15608` `eval_kwargs_construct` | eval | a located raise |
| 3 | `rete/matcher.rs:625` `build_insert_fact` | rete, fire-time | a located `TypeMismatch` |
| 4 | `rete/validate.rs:662` `validate_and_reorder_then` | rete, freeze-time wall | a freeze error |
| 5 | `rete/compiled_rhs.rs:92` `compile_rhs` | rete, compiled fast path | `None` → falls back to the interpreted path |

Every one computes the same three-clause predicate:

```rust
let is_kwargs = args.len() >= 2
    && args.len() % 2 == 0                                                   // sites 3,4,5
 // && args.len().is_multiple_of(2)                                          // sites 1,2
    && args.iter().step_by(2).all(|a| matches!(a, WatAST::Keyword(_, _)));
```

## ★ They have ALREADY drifted

Sites 1 and 2 spell the parity check `is_multiple_of(2)`; sites 3, 4 and 5 spell it `% 2 == 0`.
**Semantically identical today** — almost certainly a clippy suggestion applied to two files and not
the other three. That is the point: it proves they are maintained *independently*, and a drift that
is currently cosmetic is one edit away from being semantic.

And the code already knows. `check.rs:11616`, in prose:

> `// Same kwargs-vs-positional test the eval arm + build_insert_fact use.`

**A comment asserting a sameness that nothing enforces.** That comment is the wall this note wants
to replace — it names the invariant and cannot hold it.

## Why this is worse than ordinary duplication

The failure mode is **not a crash**. If one site decides `kwargs` and another decides `positional`
for the same form, the argument list is zipped differently — **field values are silently assigned to
the wrong fields.** A wrong answer, produced quietly, at a boundary the type system cannot see
across because both readings are well-typed.

The exposure is concrete: sites 3, 4 and 5 are three phases of *the same* rete RHS — the freeze-time
wall, the compiled fast path, and the interpreted fire path. If the compiled path and the
interpreted path disagreed about one form, a rule would derive different facts depending on which
path executed it — and the differential harness compares wat against Clara, not the compiled path
against the interpreted one.

## What it is NOT

**Not "merge the five functions."** Their failure behaviour legitimately differs by phase — a
freeze-time wall must produce a located error, and `compile_rhs` must return `None` so the
interpreted path can take over. Collapsing them would break that.

It is **one predicate, five consumers**. The shape question (`is_kwargs`, and the resulting
key/value split) extracts to a single function; each site keeps its own error behaviour.

**Not a rete concern.** Two of the five are the checker and the evaluator for aggregate construction
generally. This is language-level.

## Evidence it is live, not theoretical

Arc 278 Stone A dropped the `(:wat::rete::insert …)` wrapper from the rete RHS. Three of the five
sites had to change. **The brief named one.** The other two were found only because the rider went
looking for *"who else reads this shape"* rather than trusting the stated scope — and nothing on
disk would have said the other four existed. A sixth site could be added tomorrow and no gate would
notice.

## The rungs

1. **A convention** — "remember to update all five." This is what exists now, in a comment, and it
   already failed to prevent the spelling drift.
2. **One predicate, five callers** — extract `is_kwargs`/the arg split to one function. Mechanical,
   small, and removes the drift class for the *shape*.
3. **A gate** — a `tests/lint/` scanner forbidding a second `step_by(2).all(… Keyword …)` outside the
   one door, turned on at zero offenders once rung 2 lands. `tests/lint/unused_span_justified.rs`
   proves Rust-side lints walking `src/` already ship here.

Rung 2 is the strike; rung 3 is what stops it regrowing. Neither is scoped or scheduled here.

## Not yet ground

Whether all five agree on the *edge cases* — a 0-arg call, a 1-arg call, an odd-length list, a
positional call whose first argument happens to be a keyword literal. They agree on the predicate's
text; nobody has driven the same odd form through all five and compared. **That comparison is the
first thing to run** if this is picked up — it converts "they look the same" into "they behave the
same," and it may find the drift is already semantic.
