# BRIEF — build the rete `defrule` wall (S1 classifier + S2 validator + S3 reorder)

**Design:** `docs/arc/2026/06/294-holon-returns-to-vsa/DESIGN-rete-defrule-wall.md` (read it first — the WHY, the
three grounded silent holes, the three ratified design calls). This brief is the HOW.

## The work, in one paragraph

A `defrule`'s `:when` patterns and `:then` inserts are quoted DATA the type-checker never validates; the runtime
matcher classifies clauses by shape and treats any unrecognized shape / unknown field-ref as a silent `None` (Clara's
lenient no-match), and `build_insert_fact` takes `:then` kwargs POSITIONALLY without name-validation or reorder. The
9a codemod corrupted many rule fixtures and NOTHING screamed. Build **one post-register freeze pass** that walks every
`defrule`'s quoted `:when`/`:then` and validates + normalizes them against the type registry, so a malformed rule is a
LOCATED freeze error and an out-of-order `:then` kwargs is silently reordered to declaration order. The wat oracle
(`wat/rete.wat`) and the native kernel (`src/rete/kernel.rs`) stay UNMOVED — this is freeze-time validation, not an
engine change.

## PROVEN FOUNDATION (a committed disconfirming probe — copy its shape)

`src/freeze/env.rs` `#[cfg(test)] mod rete_wall_probe` (already on the disk, green) proves:
- **`build_env(forms)` → `EnvBundle { types: TypeEnv, symbols, residue: Vec<WatAST> }`**; `residue` = post-register,
  post-resolve USER forms.
- A `defrule` expands to `(:wat::core::defn <name> [] -> :wat::rete::Rule (:wat::rete::make-rule "<name>" (:wat::core::quote [<when>…]) (:wat::core::quote [<then>…])))`. The `make-rule` call is reachable in `residue` by recursive descent for a `WatAST::List` headed by `WatAST::Keyword(":wat::rete::make-rule")`; the quoted `:when` is `mr[2]` = `(:wat::core::quote <Vector>)`, `:then` is `mr[3]`.
- **The quote survives `resolve` UN-MANGLED** — head keyword + clause list intact; `resolve` does NOT choke on the
  free `?loc`/`<-` inside the quote.
- **Field order reads from the registry via the COLON-PREFIXED key**: `env.types.get(":weather::Temperature")` →
  `TypeDef::Aggregate(a)` → `a.field_names()` = `["celsius","location"]` (declaration order). The runtime matcher
  uses the same key + `sym.types()` (`matcher.rs:126-135`) — use the same accessor for consistency.

Keep or fold this probe into the S2 test file; do not delete it (it is the reachability contract).

## Rooms (read in order)

1. `src/rete/matcher.rs:178-306` — `eval_clause`, the **single rete-DSL clause grammar** (bind `(?v <- :field)`,
   FQDN constraints `(:wat::core::= a b)` / `not=`/`<`/`>`/`<=`/`>=`, combinators `:wat::rete::and`/`or`/`not`, the
   `:wat::rete::where` STOP arm, unknown → `None`). **S1 extracts the shape recognition from here.**
2. `src/rete/matcher.rs:379-479` — `build_insert_fact`; the `:then` kwargs handling at `:445-461` (the positional
   strip the reorder replaces) and the `matcher.rs:451` follow-up comment this retires.
3. `src/rete/matcher.rs:108-135` — the proven `:{class}` key + `field_names()` accessor.
4. `src/freeze/env.rs:80-357` — `build_env`; hook the pass at the END, after `resolve_references`, on the resolved
   user residue + `types`, before constructing `EnvBundle`. `EnvBundle` fields at `:40-47`.
5. `src/types.rs:238-250` — `AggregateDef::field_names()` / `field_types()` (declaration order); `:440` `TypeEnv::get`.
6. `wat/rete.wat:1948-1965` — the `defrule` macro (confirms the `make-rule` + double-`quote` expansion).
7. `src/rete/exists`/`accumulate` clause shapes — grep `:wat::rete::exists` / `accumulate` in `matcher.rs` +
   `wat/rete.wat` to enumerate ALL recognized `:when` clause heads so the classifier is COMPLETE (an incomplete
   classifier would false-positive a legitimate clause — see STOP-2).

## Implementation sketch

**S1 — the shared classifier (`src/rete/matcher.rs`):**
```rust
pub(crate) enum ReteClauseShape<'a> {
    Bind { var: &'a str, field: &'a str },              // (?v <- :field)
    Constraint { op: &'a str, operands: &'a [WatAST] }, // (:wat::core::= a b), field-refs inside
    Not(&'a WatAST), And(&'a [WatAST]), Or(&'a [WatAST]),
    Exists(&'a WatAST), Where(&'a WatAST), Accumulate(/* … */),
    Unrecognized,                                        // ← the wall rejects this; the matcher None's it
}
pub(crate) fn classify_rete_clause(clause: &WatAST) -> ReteClauseShape<'_> { … }
```
Re-point `eval_clause` to `match classify_rete_clause(clause) { … }` — behavior-IDENTICAL (Unrecognized → `None`).
Prove with the full rete suite 0-new BEFORE S2.

**S2 — the validator (`src/rete/validate.rs`, new warded home):**
```rust
pub(crate) fn validate_rete_rules(residue: &mut [WatAST], types: &TypeEnv) -> Result<(), CheckErrors> { … }
// recursive descent (mirror find_make_rule in the probe) → for each make-rule:
//   :when quote (mr[2]) → each condition (:Type …clauses…):
//     - Type (colon-keyed) registered record? else CheckError #wat.rete/UnknownFactType (span).
//     - each clause: classify_rete_clause; Unrecognized → #wat.rete/MalformedClause (span, the bad form).
//     - Bind.field / Constraint field-ref operands: a real field of Type? else #wat.rete/UnknownField
//       (span, type, bad field, available field_names). The free ?v stays free.
//     - Not/And/Or/Exists recurse.  Where/Accumulate: validate the OUTER shape + fact-type head only
//       (design call 3 — do NOT deep-walk a where predicate's arbitrary interior).
//   :then quote (mr[3]) → each (:wat::rete::insert (:Type …args…)):  ← see S3.
```
Hook in `build_env` post-resolve; errors flow through the existing `CheckError`/`CheckErrors` freeze channel
(`env.rs:22`). Add the new `#wat.rete/*` variants to the same error enum (`conformare` — each variant diagnostic-
complete: names the rule, the span, the type, the field, the available fields).

**S3 — the shared reorder + `:then` normalize (`src/rete/validate.rs`, helper in a shared spot):**
```rust
// ONE helper, single-sourced (the (C) spliced-construction pass calls it too — do NOT inline it):
pub(crate) fn reorder_kwargs_by_field_name(
    field_order: &[&str], kv_pairs: &[(&str, &WatAST)],
) -> Result<Vec<WatAST>, /* UnknownField */ …> { … }
// :then (:insert (:Type k1 v1 k2 v2 …)) kwargs (matcher.rs:454-456 shape test):
//   validate each :field ∈ Type; reorder value-ASTs to declaration order; REWRITE the quoted form in
//   the residue in place → build_insert_fact receives declaration order at fire time.
// positional (:Type v1 v2 …): arg count == field count else #wat.rete/RhsArityMismatch.
```

## Blast radius

`src/rete/matcher.rs` (extract classifier — no behavior change), `src/rete/validate.rs` (NEW), `src/freeze/env.rs`
(one hook + keep the probe), the freeze error enum (new `#wat.rete/*` variants). **DO NOT touch** `wat/rete.wat`,
`src/rete/kernel.rs`, or any `.wat` rule fixture — fixing the fixtures the wall reveals is a SEPARATE fan-out
(the census), not this strike.

## STOP triggers (halt + report; do not improvise)

1. **If the post-register hook cannot reach the quoted forms or `types` as the probe shows** — STOP, report. (It
   can; the probe proves it. If your build sees otherwise, something moved.)
2. **If a `:when` clause head you don't recognize appears in a CURRENTLY-GREEN rete fixture** — that means the
   classifier is INCOMPLETE, not the fixture wrong. STOP, report the clause; do NOT reject it. (Enumerate every
   `:when` head from `matcher.rs` + `wat/rete.wat` first — room 7 — so the classifier is complete before it walls.)
3. **If validating a `where`/`accumulate` interior is needed to pass a green fixture** — STOP; the design bounds the
   wall to outer-shape + alpha field-refs. Report; do not deep-walk arbitrary exprs.
4. **Do NOT reorder or "fix" any `.wat` fixture** — the wall makes them scream; a human/fan-out fixes them.

## Gate (I re-run ALL of this myself — your report is a hypothesis)

- `cargo build --release` clean.
- The `rete_wall_probe` test still green.
- A NEW probe test: the corrupt defrule (injected `:celsius` clause) → a LOCATED `#wat.rete/MalformedClause` (or
  `UnknownField`) at freeze; a CORRECT defrule → freezes clean; an out-of-declaration-order `:then` kwargs →
  reordered, fires correct.
- `cargo nextest run --release -E 'test(/rete/)'` — the rete suite: the SHARED-classifier S1 adds 0-new; the S2/S3
  wall RE-SHAPES the ~17 corrupt fixtures from wrong-count FAILs into located-error FAILs (that is the census,
  EXPECTED — not a regression). Report the exact list of now-screaming fixtures.
- Whole floor `cargo nextest run --release` — NO NEW failures OUTSIDE the rete-rule fixtures (the wall must not
  break non-rule code). The known `no_inlined_wat` lint stays the one allowed failure.

## Method

- Build ONCE to a temp file, grep the FILE; use targeted `-p wat --lib` / `--test <name>` runs; never re-run the
  full 5-min suite to re-grep.
- A rust-analyzer / rustc diagnostic on a MID-EDIT file is a PHANTOM — a suite that RAN N tests compiled. Ground a
  cascade against a real `cargo build` before believing it.
- Commit nothing; leave the tree for the orchestrator to weigh. Report: the diff summary, the new-screaming-fixture
  list, and your own gate results.
