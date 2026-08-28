# NOTE — `walk_for_restricted_call` fires on a MENTION, not a call

> Surfaced by P5-a's rider while trying to measure render-doc output for `:wat::kernel::spawn-thread`.
> Verified on disk 2026-08-28 at `ab753e4f5`. **No row: this is a finding, not a drawn stone.**

## What it does

`src/check.rs:1430`:

```rust
fn walk_for_restricted_call(node: &WatAST, enclosing_fn: &str, …) {
    if let WatAST::Keyword(name, name_span) = node {          // ← ANY keyword node
        …  errors.push(DefRestrictedCallerNotAllowed { … })
    }
    for child in node.children().iter() { walk_for_restricted_call(child, …) }  // ← everywhere
}
```

There is no head-position test. The walker recurses through every child and raises on **any**
`WatAST::Keyword` naming a `:restricted-to` verb, wherever it appears.

## What that costs

Reflection over a restricted verb is **unreachable from ordinary `.wat`**:

```wat
(:wat::core::render-doc :wat::kernel::spawn-thread)
;; DefRestrictedCallerNotAllowed — because the verb appears as an ARGUMENT,
;; not because anything called it.
```

`render-doc`, `show-source`, `metadata-of`, `signature-of` — the whole reflection surface arc 255
has spent the arc making honest — cannot be pointed at any restricted verb from a `.wat` program.
P5-a's rider had to reach past the check pass entirely (`wat::freeze::eval_in_frozen` in a throwaway
Rust harness) to take a before/after reading.

## Why it is not obviously a bug

Restricting the *mention* is defensible: a keyword in hand can be passed to `:wat::core::apply` and
called, so "you may not name it" and "you may not call it" collapse if `apply` is reachable. That
argument is real and may be the reason the walker is written this way.

**What is NOT defensible is the gap between the name and the behaviour.** The function is
`walk_for_restricted_call`; the error is `DefRestrictedCallerNotAllowed`; both say *call*. Nothing
in either says a reflection verb reading the name as data is refused. A reader who trusts the name
will be wrong, and the P5-a rider was — it cost a detour into a Rust harness to get one measurement.

## What would close it

Either the name and diagnostic say **mention** (cheap; makes the shipped behaviour honest), or the
walker distinguishes a call head from a data position and the reflection verbs are let through
(larger; needs the `apply` argument above answered first, and `apply` NOW REACHES more verbs than it
did when this was written — arc 255's whole O-iv sweep changed that premise).

⚠ **That second clause is the reason this is a NOTE and not a one-line rename.** The restriction's
premise — "a named verb can be applied" — was TRUE-but-narrow when written and is broader now.
Whoever draws this must re-derive the premise, not inherit it.
`[[feedback_a_rulings_premise_expires_but_the_ruling_stands]]`
