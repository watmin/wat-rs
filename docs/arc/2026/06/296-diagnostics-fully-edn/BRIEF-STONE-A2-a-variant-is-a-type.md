# BRIEF — STONE A-2: a variant is a type

## The work, in one paragraph

An enum's variant becomes a type you can name in an annotation, that its constructor produces, and
that widens freely to its enum. Two programs that cannot be written today then can be: a function
that takes **only** one variant and destructures it directly, and a `let` that constructs a variant
and passes it to such a function. `match` on a variant already works and is not touched.

## Read in order

1. `tests/types/probe_arc296_a2_a_variant_is_a_type.rs` — **the committed probe, first.** Three
   controls green, three subjects `#[ignore]`d. Its header carries every measurement and states what
   has already been settled so this stone does not re-derive it.
2. `src/declare/register.rs`, `register_enum_methods` — **the erasure, and it is ONE LINE:**
   `ret_type: enum_type.clone()`, just above `sym.register_function(constructor_path, …)`.
   ⚠ `infer_enum_map_ctor` in `src/check.rs` is **NOT** the live path — measured: editing its
   fallback changed nothing, because `env.get(k)` finds this registered scheme first.
3. `src/types.rs` — `register_subtype` / `subtype_edges` / `is_subtype`. A variant edge is
   **head-level**, and head-level edges already carry values through `assignable`.
4. `src/check.rs`, the `{:keys}` path (`MapDestructureKind::Keys`) — its comment says *"HashMap /
   enum / surface are other TypeDef arms and stay out."* That predicate is what must widen.
5. `docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-STONE-P-a-variant-is-a-type.md` — the original
   notes. They name the payoff (`defclause` routing per variant, from the router that already exists
   at 72 sites) and the question this stone answers.

## Implementation sketch

Three pieces, and they must land together:

```
1  register each variant as a type carrying its OWN declared fields, plus a head-level
   subtype edge  Variant <: Enum.
2  the constructor stops erasing: register_enum_methods' `ret_type` becomes the VARIANT,
   not the enum. Widening then carries every existing call site.
3  {:keys} widens its predicate to "does this carry named fields?" — a tagged variant does.
```

⛔ **Do NOT register a variant as a `TypeDef::Aggregate` to make `{:keys}` work.** That is shaping
the TYPE to fit a PREDICATE. The predicate is what is too narrow — exactly as Stone O's guard asked
`nature == Struct` when the question was "has named fields". **A variant stays a variant.** This is
the builder's ruling and it is not open.

## Acceptance

```
cargo nextest run --release -E 'test(a2_a_variant)'  6 passed, 0 skipped   (all three un-ignored)
```

```
…__process_full_box.wat          --check EXIT 0     the builder's function
…__ctor_carries_the_variant.wat  --check EXIT 0     the builder's posterity example
…__variant_widens_to_enum.wat    --check EXIT 0     ⛔ the widest control
…__match_still_works.wat         --check EXIT 0     regression control
…__nonexistent_variant.wat       --check EXIT 1     over-reach detector
…__enum_does_not_narrow.wat      --check EXIT 1, and the message must NOT contain
                                                    "UnknownNamedType"
```

Earlier stones must not move:

```
-E 'test(p1_annotation)' 10 · -E 'test(p1b_a_parametric)' 4 · -E 'test(p2prereq)' 4
-E 'test(p3_one_question)' 5 · -E 'test(a1_one_rule)' 4
```

## Blast radius

`src/declare/register.rs`, `src/types.rs`, and the `{:keys}` predicate in `src/check.rs`.

**The corpus WILL speak, and the builder has ruled that this is fine**: *"if its a hard migration…
we've done those many times over."* Every enum construction now carries a narrower type. The
fail-count is the progress meter, not a gate — but **classify the remainder BY REASON before
reporting any delta**, and read one verbatim failure per cluster. A falling total says nothing about
kind.

## STOP triggers — each is a REJECTION: ship nothing, report the gap

**STOP-1.** If `variant_widens_to_enum` goes red — STOP and report the verbatim block. Widening is
the foundation; without it every existing construction in the corpus breaks and nothing else matters.

**STOP-2.** If making `{:keys}` work needs the variant registered as an Aggregate — STOP. Ruled out.
Report what the predicate could not see.

**STOP-3.** If `enum_does_not_narrow` goes green — STOP. Variants became ALIASES of their enum,
which passes most rows and destroys the only reason the type is worth having.

**STOP-4.** If the ctor change requires touching the runtime VALUE's shape (`src/record/construct.rs`)
rather than the checker's view — STOP and report. The wire form already carries the variant; a
runtime change ripples into EDN, comms and goldens and is a different stone.

**STOP-5.** If a corpus site needs its ANNOTATION widened from a variant to the enum to keep
compiling — STOP and report the list. That would mean widening is not carrying values the way this
design claims.

## Tier

You edit and report. Run the six fixtures and the six targeted probe binaries above. **You do NOT
run the floor or clippy — the orchestrator runs those centrally, once, after the tree is quiescent.**
Report what only you can: which registration site you used, whether the runtime twin needed
touching, and what the corpus said classified by reason.
