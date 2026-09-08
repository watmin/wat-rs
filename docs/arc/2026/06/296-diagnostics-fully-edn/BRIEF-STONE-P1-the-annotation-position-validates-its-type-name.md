# BRIEF — STONE P-1: an annotation may not name a type that does not exist

## The work, in one paragraph

A type annotation whose name contains `::` or `.` names a **named type**, and that type must exist.
Today it does not have to: `(:wat::core::defn :user::f [s <- :usr::TotallyMadeUp] -> :usr::TotallyMadeUp s)`
type-checks clean and runs. Add the refusal at declaration-registration time, using the type-variable
discriminator that already exists, so the two sibling walls (type variables accepted, bare legacy
primitives refused) keep behaving exactly as they do now.

## Read in order

1. `tests/types/probe_arc296_p1_annotation_names_a_type.rs` — **the committed probe. Read it first.**
   Five controls are green and stay green; two subjects are `#[ignore]`d and this stone un-ignores
   them. Its header carries the full measurement and the three-lexical-classes table.
2. `src/declare/parse.rs:505-540` — **the room.** The block that builds a `Function`'s `type_params`
   by unioning the `<T>` suffix, the `:- [T…]` binder, and `collect_free_type_vars(&param_types,
   &ret_type)`. At this point `param_types`, `ret_type` and the complete bound-variable set are all
   in hand at once. This is why the check goes here and not in the type parser.
3. `src/declare/typevar.rs` — `collect_free_type_vars`, `collect_free_type_vars_in`, and the shared
   recursion `walk_free_type_vars`. Its header records that stone 251.8a collapsed four hand-rolled
   versions of this walk into one door. **Share this recursion; do not write a fifth walker.**
4. `src/declare/parse.rs:1039` — `is_type_var_path`, the three-lexical-classes rule. This is the
   class assignment. It is already load-bearing; the stone consumes it, it does not restate it.
5. `src/types.rs:601` — `TypeEnv::contains` = `types ∪ builtin_names`. Paired with
   `crate::runtime::is_builtin_primitive`, this is the membership union arc 296 Stone Q established.
   `src/reflect/verbs.rs:1546` (`eval_is_type`) shows the exact pairing to mirror.

## Implementation sketch

Walk each annotation's `TypeExpr` with the SAME recursion `walk_free_type_vars` uses. For every
`TypeExpr::Path(p)`:

```
if is_type_var_path(p)                      -> a type VARIABLE. Accept. (Already handled.)
else if bound in raw_type_params            -> accept.
else if TypeEnv::contains(p)
        || is_builtin_primitive(strip(p))   -> accept.
else                                        -> REFUSE, naming p and its span.
```

Recurse into `Parametric { args }`, `Fn { args, ret }`, and `Tuple` elements exactly as
`walk_free_type_vars` does. `TypeExpr::Var(_)` is synthetic — ignore it, as that walk does.

The refusal is its own named error variant carrying the offending path, in the shape the sibling
`BareLegacyPrimitive` uses — read that variant and mirror its structure so the two walls of the
family report alike.

Aggregate FIELD annotations (`defrecord` / `defstruct` / `defenum` variant fields) take the same
walk. `tests/types/…__phantom_record_field.wat` is the fixture that proves that half.

## Acceptance — what "done" means

```
cargo nextest run --release -E 'test(p1_annotation)'      7 tests, 7 passed
```

All seven, with the two `#[ignore]` attributes REMOVED. Five of the seven are green today; if any
of those five goes red, the wall over-reached, and that is a finding to report rather than a bar to
adjust.

## The corpus census — the second half of the deliverable

This wall is expected to go red across the corpus, and **that red is the deliverable, not a
problem.** After the wall stands, run `--check` over every `.wat` under `wat/`, `wat-scripts/` and
`wat-tests/` with the freshly built `target/release/wat`, and report:

- how many files refuse,
- the DEDUPED list of distinct offending type names, in frequency order,
- and for the top five, one verbatim error block each.

That list is the worklist for whoever fixes the corpus. Group it by whether the name looks like a
typo, a retired spelling, or a type that genuinely was never declared — the three groups get
different treatment and only the census can tell them apart.

## Blast radius

`src/declare/` and one error-variant declaration. No change to `parse_type_expr` or any of its five
entry points. No change to `resolve`. No new walker.

## STOP triggers — each is a REJECTION: ship nothing, report the gap

**STOP-1.** If making the wall stand requires threading a `&TypeEnv` into `parse_type_expr` or any
of its entry points — STOP. That path was measured and rejected (its own doc says the public entry
exists for callers with no span in scope, and `wat_record_from!` parses at Rust-compile time where
no `TypeEnv` exists). Report what forced it.

**STOP-2.** If any of the five green controls goes red — STOP. In particular
`a_bare_uppercase_name_is_a_type_variable_even_with_no_binder` and
`a_bound_type_parameter_is_not_a_phantom` are the over-reach detectors, and
`a_bare_legacy_primitive_is_already_refused_by_its_own_wall` catches a generic refusal replacing a
named one. Report which, with its verbatim output.

**STOP-3.** If the corpus census comes back with more than ~40 distinct offending names — STOP after
the census and report it. That size means the class is not "phantoms" but something structural the
design did not see, and it is the orchestrator's to re-plan.

**STOP-4.** If a fixture you need does not exist, or a fixture appears to contradict the design —
STOP and report. Do not add a fixture that makes a bar easier to clear.

## Tier

You edit and report. Run the seven targeted probe tests and the corpus `--check` census — those are
cheap and scoped. **The orchestrator runs the floor and clippy centrally, once, after the tree is
quiescent.** Your numbers are the ones the orchestrator cannot reconstruct: which sites you
inspected, what the census found, what surprised you.
