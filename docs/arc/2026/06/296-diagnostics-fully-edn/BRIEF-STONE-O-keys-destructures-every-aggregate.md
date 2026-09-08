# BRIEF — STONE O: `{:keys …}` destructures every aggregate

> Read `DESIGN-STONE-O-keys-destructures-every-aggregate.md` first. **The probe is committed and
> RED**: `tests/types/probe_arc296_keys_on_aggregates.rs`, 5 rows + 5 co-located fixtures,
> 2 green / 3 red.

## THE WORK, IN ONE PARAGRAPH

`src/check.rs:12725` accepts a `let` map-destructure target only when
`a.nature == Nature::Struct`. Widen it to accept **any aggregate**. The natures differ in purity,
not shape; all four carry named fields; destructuring reads fields. One predicate, one line of
substance, four kinds served.

## READ IN ORDER — the rooms

```
src/check.rs:12725              THE SUBJECT. The nature-equality guard and its comment.
src/check.rs:12691 · :12717     the surrounding field lookup + the TypeMismatch this raises
                                ("struct-destructure (x y) expects a struct type") — the message
                                must stop saying "struct" once the rule is not about structs.
src/types.rs:218-232            Nature: is_pure (the ONLY axis that separates the kinds) and
                                rank() (the substitution ladder). Read the doc comments; they are
                                what makes the widening derivable rather than a preference.
src/runtime.rs:4238             the `{:keys …}` / `{var :field}` destructure reader (arc 257.2).
                                ⚠ Check whether the RUNTIME side has the same nature restriction.
                                The checker refusing is what we measured; if the runtime also
                                gates, both move together or the fix is half.
tests/wat_lang/probe_arc257_keys_destructure.wat
                                arc 257.2's own probe — two defstructs, no defrecord. This is HOW
                                the gap survived; it is not a file to change.
```

## THE ACCEPTANCE ROWS ARE THE PROBE

`cargo nextest run --release -E 'test(probe_arc296_keys_on_aggregates)'` — un-ignore the three and
make them green **without touching rows 1-2**:

```
the_binder_first_control_still_destructures_a_record   GREEN now, must STAY green
keys_destructures_a_defstruct                          GREEN now, must STAY green (no regression)
keys_destructures_a_defrecord                          RED
keys_destructures_a_holon_defrecord                    RED
keys_destructures_a_defholon                           RED
```

Every bar is `check("binder_first_control")` run in the same test — the SAME aggregate, the SAME
fields, a form that already works, so the `{:keys}` reader is the only variable. Do not replace it
with a literal.

## STOP TRIGGERS

- **STOP-1 — the guard becomes a list of natures.** `matches!(a.nature, Struct | Record | HolonRecord)`
  is the hand-list this arc keeps deleting, and it goes stale the next time a nature is added —
  exactly as the current one went stale when a type was REMOVED. Ask "is this an aggregate?".
- **STOP-2 — `Nature::Peer` is admitted without a decision.** Peer is off-ladder (`rank() = i8::MIN`)
  and holds a live channel. If the widened predicate would let a Peer be destructured, say so and
  rule on it explicitly; do not let it in silently.
- **STOP-3 — the error message still says "expects a struct type".** Once the rule is not about
  structs, a message that says struct sends the next reader to the wrong place.
- **STOP-4 — only the checker is fixed.** If `runtime.rs`'s destructure reader gates on nature too,
  a green `--check` and a runtime failure is worse than today's honest refusal. Verify by RUNNING a
  fixture, not just checking it.
- **STOP-5 — arc 257.2's probe is edited to cover the gap.** It is the historical record of how this
  survived. The new probe is the coverage.
