# DESIGN — the rete `defrule` wall: one post-register validator that makes every silent rule-corruption LOUD

> **Directive (the builder, 2026-07-13):** *"set the heretics ablaze, the shadowdancers will find them by their
> screams — all the silent rete failures die now — we should strive for one way to enforce correctness here; a
> 'half wall' sounds like a very bad idea."*

## Why (the failure class this pulls out by the root — `extirpare`)

The 9a kwargs codemod corrupted rete `defrule` fixtures and **nothing screamed** — the floor showed wrong derived
counts (`got 0`), test-by-test, not located errors. Grounded mechanism:

- A `defrule :when` pattern is **DATA** (a quoted `WatAST`), interpreted at *fire* time by the alpha-matcher
  (`src/rete/matcher.rs:195-306`). The type-checker never sees it as a construction. The matcher classifies clauses
  by SHAPE and — mirroring Clara — treats **any unrecognized clause shape as `None` (no match, no error)**
  (`matcher.rs:201-204`, and explicitly `matcher.rs:297-301` *"Clara no-error: unhandled clause = no match"*). A
  field-ref to a **nonexistent field** is the same silent `None` (`matcher.rs:228` — `read_fact_field(...)?`).
  → The codemod's injected `:celsius` keyword became an unclassifiable clause #1 → the condition silently never
  matched → the rule silently never fired → `0`. A *dead rule* even passed one sub-test (`negation_blocks_*`),
  because `0` is coincidentally the right answer for the "blocks" case.
- A `defrule :then` insert is KWARGS read by `build_insert_fact` (`matcher.rs:379-479`). Structure is loud
  (`:388-441`), **but** the kwargs handling (`matcher.rs:445-461`) **strips the `:field` keywords and takes the
  values POSITIONALLY, in written order** — no name validation, no reorder. The comment at `:448-452` admits the
  hole: *"Follow-up: compile-time reorder-by-name (field-names-of) would make an out-of-declaration-order kwargs RHS
  correct rather than positionally mapped."* → a wrong `:then` field name is silently ignored; an out-of-order
  kwargs `:then` silently produces a **scrambled fact** (right count, wrong values).

Three silent holes, one class: **rule forms reference a fact's schema but nothing validates them against it.** This
is the magic-free floor (R28 `SOLVIMVS NE MENTIRETVR` / R29 `RVINA ERVDIT`) that the rete DSL never got, because it
inherited Clara's lenient-matcher convention. The wall closes the class and — the load-bearing consequence — makes
the floor **enumerate the entire corruption backlog by located error** (the shadowdancers find the heretics by
their screams), instead of hunting fixtures by heuristic grep.

## What — one pass, no half-wall

A single post-register Rust pass, `validate_rete_rules`, hooked in `src/freeze/env.rs` **immediately after
`register_types` (step 5)** — the phase where `env.types()` field names are authoritative (same seam as
`register_aggregate_kwargs_companions`, `env.rs:108`, and the (C) reorder pass). It recursively walks the user forms
into `defn`/`do`/`let` bodies (mirror `register_types_impl`, `src/types.rs:1847`), finds each
`(:wat::rete::make-rule "<name>" (quote [<when>…]) (quote [<then>…]))` (what `defrule` expands to,
`wat/rete.wat:1948-1965`), and validates + normalizes both quoted vectors against the registry:

### `:when` conditions (DATA — positional requirement-clauses; kwargs is a corruption here)
For each condition `(:Type …clauses…)`:
1. `Type` (head keyword, colon-stripped) must be a **registered record** → else located `#wat.rete/UnknownFactType`.
2. Each clause must classify to a **known rete-DSL shape** via the SHARED classifier (below) → else located
   `#wat.rete/MalformedClause` naming the clause + span. (Kills the injected-`:celsius` hole — a bare keyword is
   not a clause.)
3. Each `(?v <- :field)` bind clause and each field-ref inside a constraint (`(:wat::core::= (?loc <- :location) …)`)
   must name a **real field of `Type`** → else located `#wat.rete/UnknownField` (names the type + the bad field +
   the available fields, `RecordDef.field_names()`). The **free `?v` stays free** (cross-condition join —
   `matcher.rs:28`); only the `:field` side is schema-checked.
4. Combinators `(:wat::rete::not …)` / `and` / `or` / `exists` recurse (their sub-conditions get 1–3).

### `:then` inserts (KWARGS — the encouraged form; validated + reordered)
For each `(:wat::rete::insert (:Type …args…))`:
1. `Type` must be a registered record → `#wat.rete/UnknownFactType`.
2. If **kwargs** (even arity, keyword at each even index — the `matcher.rs:454-456` shape test): every `:field` must
   be a real field of `Type` → `#wat.rete/UnknownField`; then **reorder the value-ASTs to declaration order** via
   the shared reorder helper and **rewrite the quoted form in place** so `build_insert_fact` receives declaration
   order at fire time. (Kills the wrong-name hole AND the silent-scramble hole; retires the `matcher.rs:451`
   follow-up.)
3. If **positional**: arg count must equal the type's field count → `#wat.rete/RhsArityMismatch`.

## The three design calls (ratify or redirect)

1. **One grammar, shared (the anti-half-wall).** Extract clause-shape recognition into one
   `classify_rete_clause(clause) -> ReteClauseShape` (in `src/rete/matcher.rs`). `eval_clause` consumes it (runtime:
   a well-formed-but-non-matching clause is still `None`); the validator consumes it (freeze: `Unrecognized` is a
   located error). A well-formed `(?v <- :field)` whose `:field` is absent is **malformed** (loud at freeze), NOT a
   runtime non-match — the distinction is knowable statically against the schema. This single source is what makes
   "one way to enforce correctness" real; a second hand-written grammar in the validator would be the exact drift
   that bred the one-sided-arm class (arc 278 R14).

2. **(C) shares the reorder, does not merge the pass.** `reorder_kwargs_by_field_name(type, &[(kw, value_ast)]) ->
   Vec<value_ast>` is ONE helper (reads `RecordDef.field_names()`), called from two sites: this `:then` normalizer
   AND (C)'s spliced-construction reorder. Different form-shapes (rete RHS vs aggregate constructions) → two focused
   passes, one single-sourced reorder. This is the "one way" without conflating two walks.

3. **`where`/`accumulate` depth = shape + alpha refs, not interiors (v1 scope).** The wall validates the outer clause
   shape, the fact-type heads, and the alpha field-refs — the entire surface the corruption class touches. It does
   NOT deep-walk a `where` predicate's arbitrary pure expr (fence/type-checked at stone 6) or an accumulate
   reducer's body. If interior validation is wanted, it's a **named follow-on**, not a half-wall on *this* class
   (which is 100% alpha field-refs + clause shapes + `:then` order). ← the one scope bound; flag if you want it wider.

## Four-questions

| | verdict |
|---|---|
| **Obvious?** | YES — a rule form that references a field the fact type doesn't have is a located error naming the type, the field, and the available fields. A reader sees exactly what's wrong. |
| **Simple?** | YES — one pass, one grammar source (shared with the matcher), one reorder helper (shared with (C)). No new construct; it consults the registry the checker already built. |
| **Honest?** | YES — this is the magic-free floor (R28/R29) extended to the rete DSL: the malformed rule cannot compile silently. It converts Clara's lenient no-match into a structural wall exactly where the schema makes correctness decidable, while preserving the load-bearing leniency (a well-formed clause that simply doesn't match a given fact is still a runtime `None`). |
| **Good UX?** | YES — the author of a rule (human or codemod) is *forced toward* the correct form one located error at a time; and the backlog censuses itself. |

## The strike (decomposed)

- **Probe (disconfirming, first):** `scratchpad/rete-wall-probe.wat` — a corrupt defrule (injected-keyword `:when`
  clause + wrong-field bind) that TODAY freezes+fires green-but-wrong; assert the pass turns it into the located
  error. Plus a correct defrule that freezes clean, and an out-of-order kwargs `:then` that the reorder makes
  correct. Prove the hook reaches `env.types()` + the quoted forms BEFORE briefing the full build.
- **S1 — the shared classifier:** extract `classify_rete_clause` in `matcher.rs`; re-point `eval_clause` at it
  (behavior-identical: floor 0-new on the rete suite). The seam, no new behavior.
- **S2 — the validator pass:** `validate_rete_rules` in a warded home (`src/rete/validate.rs`), the recursive walk +
  the `:when`/`:then` checks + the located `#wat.rete/*` errors (`conformare`: each variant reaches diagnostic
  completeness). Hooked in `env.rs` post-register. GATE: the probe's corrupt rule → located error; correct rule →
  clean.
- **S3 — the `:then` reorder + the shared helper:** `reorder_kwargs_by_field_name`; wire `:then` normalization;
  (leave (C)'s call site for the (C) strike, but land the helper here).
- **THE CENSUS:** run the whole floor. Every corrupt rete fixture now SCREAMS a located error. Fan out shadowdancers
  to fix each by its scream (`:when` → revert to positional DATA; `:then` → the reorder accepts kwargs, or fix the
  named bad field). Weigh the floor to green by my own re-run.

## Blast radius / STOP triggers

- Touches: `src/rete/matcher.rs` (extract classifier), `src/rete/validate.rs` (new), `src/freeze/env.rs` (hook +
  the ~19 rete fixtures the census reveals). The wat oracle (`wat/rete.wat`) and the native kernel stay UNMOVED
  (this is a freeze-time validator, not an engine change) — the R22 `OCVLI NOVI ORACVLVM IMMOTVM` line holds.
- STOP if the post-register hook cannot reach the quoted `:when`/`:then` as `WatAST` (report; do not invent a
  runtime-side check). STOP if a "correct" existing rete fixture the census flags is actually a legitimate form the
  grammar doesn't yet recognize (surface it — it means the classifier is incomplete, not the fixture wrong).
- The known-good baseline is the floor at 64; the wall will re-shape those failures (wrong-count → located error)
  and must not add NEW non-rete failures.

## FOLLOW-ON (captured 2026-07-13, builder's observation — NOT this strike; after the wall returns + is weighed)

The builder saw the shape: this wall is being built as a **proper feature of wat's freeze pipeline**, not a
rete-specific bolt-on — and asked how to **expose it to other distributions of wat** (users who write their own
Rust internals / their own domain DSLs).

**The general capability it's an instance of — a pluggable `FreezeValidator` extension point.** `validate_rete_rules`
already has the right signature: `(residue: &mut [WatAST], types: &TypeEnv, symbols: &SymbolTable) -> Result<(),
LocatedErrors>` — run post-register, walk user forms, consult the registry, emit located errors, optionally
normalize (the `:then` reorder). Lift that into a trait + a compile-time registry, and any wat distribution's Rust
crate can register its OWN freeze-time validators for its OWN DSL — getting wat's magic-free floor (R29 `RVINA
ERVDIT` — the system educates the caller) at freeze, against the registry, WITHOUT forking the pipeline.

**The mechanism is already in the building.** `env.rs:97` references an `inventory` drain that registers user
`EdnSchema` TYPES into the registry at compile time (a distribution author already plugs types in this way). A
`FreezeValidator` is the exact sibling: `inventory::submit!` a validator → the freeze pipeline drains the registry
post-register and runs each. **Types + validators = a distribution author adds both their domain's data shapes AND
its correctness rules, first-class.** This is the R31/R32 lineage (wat as the substrate others build on,
decomplected) extended from *interfaces* to *validation*.

**The clean path (do NOT do it now):** build the rete wall as a clean INSTANCE first, prove it green, THEN lift the
pattern — the rete wall becomes the FIRST registered validator, dogfooding the extension point (`PRIMVS VSVS ANGVLOS
PANDIT` — the first real consumer proves the seam; the same discipline as MemStore being the first `Store`
satisfier, R26). Ground the `inventory` `EdnSchema` API (`env.rs:97`, `src/**` `inventory::submit!` sites) before
designing the registry — do not assert its shape. A named follow-on stone, its own scout + four-questions; not a
scope-creep on the wall.
