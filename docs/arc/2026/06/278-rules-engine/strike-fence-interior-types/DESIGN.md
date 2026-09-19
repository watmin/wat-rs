# DESIGN — type-check the `where` fence's interior

## Why

`src/rete/validate/mod.rs:282` has an empty arm:

```rust
// Design call 3 — a `where` fence's outer shape is already confirmed by the
// classifier (2-item, `:wat::rete::where` head); its interior expr is out of scope.
ReteClauseShape::Where(_) => {}
```

The comment reads as a scoping decision about SHAPE. In practice the interior receives **no type
checking whatsoever**. Driven 2026-09-09 at `aefaad5c1`, one predicate in two positions:

| position | result |
|---|---|
| inside a `(:wat::rete::where …)` fence | ⛔ **ACCEPTED SILENTLY.** Starts up clean, rc=0 |
| the same predicate written inline | ✅ `ConstraintTypeMismatch`, rc=3 |

Already in the corpus: `wat-scripts/fixes/to-faithful-clojure-net.wat`'s `g3-genuine` carries a
`string::=` over `i64` fields inside a fence, **in a recorded-migration exemplar**, and surfaced
only because a codemod happened to hoist it inline.

The probe is committed at `d8068e26a` —
`tests/rete/probe_arc278_fence_interior_types.rs`, three tests, the gap banked.
Full finding: `../the-fence-says-what-the-clause-cannot/FINDING-the-fence-is-a-hole-in-the-type-system.md`.

## What it delivers

1. A fence's interior is type-checked exactly as the same predicate written inline.
2. ⭐ **The census.** Nothing has ever looked at the other fence interiors. Filling the arm makes
   `every_wat_scripts_file_loads` and the floor enumerate the population in one run.

## ⛔ THE ONE CONTRACT DECISION

> **A fence's type error is its OWN error variant, carrying NO `fact_type`, because a fence is not
> scoped to a fact.**

All thirteen existing `ReteCheckErrorKind` variants carry `rule` + `fact_type`. A fence sits at
`:when` top level, after the conditions, bound to none of them — so any fact type printed there
would name a location the reader must then discover is not where the mistake is.

The precedent is stated in this same file, on `NonReteConstraint`: *"its own variant rather than
`MalformedClause` deliberately … saying 'malformed' would teach the wrong fix (R29 `RVINA ERVDIT`
— the ruin IS the lesson, so it must name the right one)."* Same argument, same answer.

⚠ **The rejected alternative, named so it is not re-litigated:** reusing `ConstraintTypeMismatch`
with a sentinel `fact_type` (`""`, `"<where>"`). It compiles, it is one line smaller, and it puts a
false location in a diagnostic — the one thing this repo's error doctrine exists to prevent.

## The algorithm

At the `Where(expr)` arm, walk `expr` recursively. At every `WatAST::List` whose head
`classify_constraint_head` recognises, resolve its operands and report a mismatch.

⭐ **No new plumbing is needed.** `validate_when_entry` already receives `binds` and `types`, and
`src/rete/validate/mod.rs:197`/`:230` show `binds = collect_rule_bind_types(when_conds, types)` — **rule-wide**,
built that way precisely because join variables cross conditions. That is exactly the map a fence's
operands resolve through.

The walk is shape-agnostic on purpose: it visits every list node rather than modelling
`and`/`or`/`not`/`if`/`let`/`match` (the shapes `src/rete/clause.rs:242`'s `expr_is_provably_boolean`
enumerates). Modelling them would be a second hand-rolled grammar to drift.

### ⛔ The two traps in reusing `check_constraint_types` verbatim

**1. `field_names`/`field_types` must be EMPTY, and that is correct, not a shortcut.**
`resolve_operand_type`'s source 1 sends a keyword naming no declared field to
`classify_keyword_constant` — *"Not a declared field -> a CONSTANT, and its type is the constant's
own."* A fence has no fact in scope, so no operand can be a `:field` reference and every keyword
there **is** a constant. Empty arrays give the right answer by construction.

**2. `is_non_field_keyword` must NOT suppress inside a fence.** Its own doc:

> *"Used only to suppress a second, misleading diagnostic: when such a keyword does not type-check
> at the comparator, `check_operand_field_ref` has already reported the located `UnknownField` …
> this only answers 'has that already been reported?'."*

In a fence **`check_operand_field_ref` never runs**, so nothing has been reported and nothing
should be suppressed. Reused as written — with empty `field_names`, where the predicate returns
`true` for *every* keyword — it would silently exempt the entire keyword-constant operand class
from the check being added. **The cure would ship a hole of the same shape as the one it cures.**

## Files

| file | change |
|---|---|
| `src/rete/validate/error.rs` | the new fence-scoped variant(s) + `Display` |
| `src/rete/validate/typing.rs` | the interior walk + operand check |
| `src/rete/validate/mod.rs:282` | the arm fills; its comment is rewritten, not deleted |
| `tests/rete/probe_arc278_fence_interior_types.rs` | `#[ignore]` comes off |
| `tests/rete/probe_arc278_fence_interior_types_fence.wat.bad` | the `bad-is-banked` rune comes off |

## Out of scope = REJECTED

These are affirmative cuts, not deferrals.

- **LAW A inside a fence** (a generic `:wat::core::>` where a rete comparator belongs →
  `NonReteConstraint`). That is an **expressivity** question, and expressivity is parked until
  after the merge by `../the-fence-says-what-the-clause-cannot/DESIGN-widen-the-clause-then-refuse-the-fence.md`.
  This strike types what is written; it does not relitigate what may be written.
- **Migrating the corpus.** A `.wat` structural sweep is a `wat-fix` codemod strike of its own
  (`wat/fix.wat`, `wat-scripts/fixes/`), never hand-edits. See STOP-3 in the BRIEF.
- **Widening `expr_is_provably_boolean`.** Same parked design; touching it changes what compiles.
- **The perf grid's 29 recorded baselines.** Nothing here rewrites a rule, so nothing invalidates
  them. If a change would, it belongs to the parked design instead.
