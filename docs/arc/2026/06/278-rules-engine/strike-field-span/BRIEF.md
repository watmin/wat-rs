# BRIEF — make the caret land on the keyword, and make the wrong span unwritable

Four sites produce `UnknownField` and they point at four different things. The only one that names
the offending keyword is **unreachable**, and its doc is the clearest statement of the contract in
the file. Three docs promise the field's span; three live sites pass an enclosing form's. Read
`DESIGN.md` beside this file first — its ★ pins the type-level cure, it records that **the
work-list row for E2 is wrong on its detail**, and its "out of scope" cuts three shapes.

## Read in order

1. `src/rete/validate/typing.rs:90-114` — `check_field_at`, its doc (*"Takes the span of the FIELD
   rather than the clause"*), and the `span: Span` parameter that cannot tell the two apart.
2. `src/rete/validate/typing.rs:45` and `:75` — `WatAST::Keyword(k, _)` discards the field's own
   span, and nine lines later `clause.span()` is passed. The right span is in hand the whole time.
3. `src/rete/validate/typing.rs:79-88` — `check_field`, the second caller; **one** caller of its own
   (`mod.rs:415`).
4. `src/rete/validate/mod.rs:779-796` — `walk_nested_constructors`: the same `Keyword(k, _)` pattern,
   the same enclosing-form span.
5. `src/rete/validate/mod.rs:940-952` — the `fact_span` producer.
6. `src/rete/validate/mod.rs:1014-1047` — `reorder_then_kwargs`: the doc that states the contract,
   and the `Err(bad)` arm that carries `bad.span` and never runs.
7. `src/rete/validate/mod.rs:974-978` — the `!all_known || has_missing` early return that is why.

## Sketch

```rust
// The producer takes the NODE, so the span can only come from the field itself.
fn check_field_at(field: &WatAST, rule_name: &str, fact_type: &str,
                  field_names: &[String], errors: &mut Vec<ReteCheckError>) {
    let WatAST::Keyword(k, span) = field else { return };   // the `_` becomes a binding
    …
}
```

Callers stop destructuring the span away and hand over the node. A newtype
(`FieldSpan(Span)`, constructible only from a keyword node) is the alternative if the node cannot be
threaded — either satisfies the ★, and if you pick the newtype say why.

## Blast radius

`validate/typing.rs`, `validate/mod.rs`, probes. `check_field_at` has 2 callers, `check_field` has 1.

## Traps named in advance — each with its step

1. **★ PROVE THE UNREACHABILITY BEFORE DELETING.** `:976` returns on `!all_known || has_missing`;
   `reorder_then_kwargs` errors when a kwarg names no field. They *look* like the same condition.
   **Step:** drive it — make a fixture whose kwargs name an unknown field and confirm which arm
   reports, then confirm the dead arm never does. If it CAN fire, do not delete it; report that and
   fix its span instead. A wrong deletion removes a real refusal.
2. **Three producers, three probes.** They are separate call paths — inline constraint, nested
   constructor, kwargs fact — and one probe proves one. **Step:** a fixture per path, each asserting
   the caret is the keyword's own extent, not the enclosing form's.
3. **Assert the extent, not just the file.** The old caret was cols 31–76 and the new one should be
   col 65 length 10 — both are in the same `.wat`. **Step:** assert the exact `(line, col, end_col)`,
   or the probe passes on the defect it was written for.
4. **`no_loose_string_assert` routes a structured value to an `.edn` golden**, not to
   `assert_eq!` on a substring. The previous strike hit this. **Step:** read that lint's rubric before
   writing the assertion, and run `binary_id(wat::lint)` before reporting.
5. **`check_operand_field_ref` already bundles its args into `ClauseCtx`** because clippy's arity
   ceiling caught it at 8. **Step:** if changing signatures pushes another fn over, bundle the way
   that one did — the ceiling is doing its job; `#[allow]` is the patch.
6. **D1's residual added `UnknownEnumVariant` on this same path.** It has its own span and must keep
   pointing where it points now. **Step:** confirm the enum-variant probes still pass unchanged.

## STOP triggers

- **STOP-1** — if the dead arm turns out to be reachable, STOP the deletion, report it, and fix its
  span. DESIGN's "delete it" rests entirely on trap 1's drive.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if threading the node is impossible at one of the three producers, STOP and name it
  rather than passing a bare `Span` there "just for that one". The ★ is that the wrong span be
  unwritable; one exempted site defeats it.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-silent-zero/` (A2b) — split-by-type on the same class of
"one value, two meanings", same arc.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Seventeen riders before you each returned a prescription of
mine that did not survive contact. The last found that a number I had put in a stone as the entire
justification for a design decision **reproduced under no definition** — it came from a script I
never committed. If a step here is wrong, unnecessary, or impossible, say it plainly.
