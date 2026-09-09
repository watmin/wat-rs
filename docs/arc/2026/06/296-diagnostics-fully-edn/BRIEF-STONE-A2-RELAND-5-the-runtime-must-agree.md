# BRIEF — STONE A-2 RELAND-5: a field-reading operation accepts a variant, in BOTH implementations

## Why this exists

A-2 passed nine `--check` rows, a green floor at 5289/5289, and the builder's own program **type-checked
and died**:

```
CHECK=0
RUN=1   ":wat::core::let: expected an aggregate type, got wat::core::Enum `(:usr::Box::Full 42)`"
```

★ **`{:keys}` was widened in `check.rs` and not in `runtime.rs`.** One question, two implementations,
disagreeing — the exact defect this whole campaign has been closing, shipped inside the campaign.
Every acceptance row I wrote asked the CHECKER, so not one of them could see it.

## The rule

> **A tagged variant carries named fields. Every field-reading operation must accept one — in the
> checker AND in the runtime.**

Same argument Stone O made for aggregates: the predicate is about SHAPE, not about which `TypeDef`
arm the value happens to live in.

## Two measured gaps, and a census that says how many more

```
{:keys} destructure     checker widened, RUNTIME not      check=0  run=1
(:field x) accessor     NEITHER widened                   check=1  "unknown callee: :inside"
                        (it checks CLEAN on a plain defrecord — so this is A-2's gap, not a
                         pre-existing limit of the accessor form)
```

**`src/runtime.rs` has 13 sites matching `Value::Aggregate(a)` and NOT ONE has a `Value::Enum` arm.**
Census them: which are field-READING operations that the checker now admits a variant to? Those must
widen. Write the table to `TABLE-STONE-A2-RELAND-5-aggregate-only-runtime-sites.md` and **report the
count before fixing** — if more than ~6 need changing, STOP and report; that is a campaign to shape,
not a sweep.

⚠ Not every `Value::Aggregate` site is field-reading. Some may be construction, transport, or
nature checks where a variant genuinely does not belong. **Classify, do not sweep.**

## Read in order

1. `tests/types/probe_arc296_a2_a_variant_is_a_type.rs` — 15 rows now, and **two of them RUN**.
   `run_program`'s doc comment explains why it had to exist.
2. `src/runtime.rs` `LetBinding::HashDestructure` (~4115) — the `{:keys}` runtime path. Its arms are
   `Value::Aggregate` (by nature) and `Value::wat__std__HashMap`; `Value::Enum` falls to
   `TypeMismatch`.
3. `src/runtime.rs` the keyword-as-accessor fall-through (~3655) — same shape, same gap.
4. `src/check.rs` — the accessor's checker side, which reports `"unknown callee"` for a variant
   receiver while accepting an aggregate one.
5. `src/value/value.rs:1150` `EnumValue` — `type_path`, `variant_name`, `names`, `fields`. **The
   field names are already carried**, in declaration order, exactly as `AggregateValue.names` are.
   Nothing needs to be looked up.

## Acceptance

```
cargo nextest run --release -E 'test(a2_a_variant)'   15 passed, 0 skipped
```

The two new rows RUN the program and assert on stdout — `42` and `7`. Earlier stones unmoved:
`p1_annotation` 10 · `p1b_a_parametric` 4 · `p2prereq` 4 · `p3_one_question` 5 · `a1_one_rule` 4.

## STOP triggers — each is a REJECTION

**STOP-1.** If widening the runtime requires changing `EnumValue`'s shape — STOP. The names are
already there; this is a read, not a representation change.

**STOP-2.** If `non_enum_container_stays_invariant` goes green — STOP. That row scopes option E and
nothing here should touch it.

**STOP-3.** If the census exceeds ~6 sites needing change — STOP after the census and report.

**STOP-4.** If a `Value::Aggregate` site needs a variant arm for a NON-field-reading reason
(construction, transport, a nature/purity check) — STOP and report which. A variant is not an
aggregate; it merely carries named fields, and the difference matters at exactly those sites.

**STOP-5.** If any fix requires the checker and runtime to encode the "has named fields" test
separately — STOP and report. Two implementations of one predicate is the defect this stone exists
to close, and re-creating it in the fix would be the campaign eating itself.

## Tier

You edit and report. Run the fixtures and the targeted `-E` filters, **including the two that
execute**. You do NOT run `scripts/floor.sh`, an unfiltered `nextest`, or clippy.
