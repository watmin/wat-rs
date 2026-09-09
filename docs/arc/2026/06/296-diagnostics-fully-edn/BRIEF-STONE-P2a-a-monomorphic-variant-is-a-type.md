# BRIEF — STONE P-2a: a monomorphic variant is a type

## The work, in one paragraph

For an enum with **no type parameters**, each variant becomes a registered type that is a SUBTYPE of
its enum, and the variant constructor stops erasing: `(:usr::Colour::Red {:shade 7})` types as
`:usr::Colour::Red`, which flows anywhere `:usr::Colour` is wanted. A `:usr::Colour` does **not**
flow where `:usr::Colour::Red` is wanted — that direction is the whole point. Generic enums are
untouched and stay refused.

## Read in order

1. `tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type.rs` — **the committed probe, first.**
   Two controls green, three subjects `#[ignore]`d; its header carries every measurement.
2. `docs/.../DESIGN-STONE-P2a-a-monomorphic-variant-is-a-type.md` — why both halves ship together,
   and where the fence is.
3. `src/types.rs` — `register_subtype` / `is_subtype` / `subtype_edges` (`:542`, `:847`), and
   `register_builtin_leaf` / `register_use_declared_leaf` for how a name gains membership.
   `EnumDef` and `EnumVariant` are the declaration shapes; `type_params` empty ⇒ monomorphic.
4. `src/check.rs` `infer_enum_map_ctor` (`:14084`) — **where the erasure happens.** This is the
   function that decides the ctor's result type.
5. `src/record/construct.rs` `try_eval_enum_map_ctor` (`:254`) — the runtime twin. Read it to see
   whether the runtime VALUE needs to change at all, or only the checker's view.
6. `tests/services/probe_arc209_spawned_marker.rs` — the worked precedent for subsumption at a
   parameter position, including the parametric-child case, and the `assignable` head-arm it names.

## Implementation sketch

At enum registration, for an enum whose `type_params` is empty, for each variant:

```
register the variant FQDN as a type       (membership; structure is the variant's own fields)
register_subtype(variant_fqdn, enum_fqdn) (the edge that makes subsumption carry it)
```

Then in `infer_enum_map_ctor`, return the VARIANT type rather than the enum type — for the
monomorphic case only. Subsumption then does the rest: an existing call passing a `Colour::Red`
where `Colour` is expected keeps working because the edge is walked by `is_subtype`.

⚠ **Check whether `is_subtype` is consulted at the argument position you need**, rather than
assuming it. Arc 209 proves it for a marker bound; confirm the same path serves an ordinary
parameter, and if it does not, that is a STOP, not a thing to route around.

## Acceptance

```
cargo nextest run --release -E 'test(p2a_a_monomorphic)'   5 passed, 0 skipped   (all un-ignored)
cargo nextest run --release -E 'test(p3_one_question)'     5 passed, 0 skipped
cargo nextest run --release -E 'test(p1_annotation)'      10 passed, 0 skipped
cargo nextest run --release -E 'test(p2prereq)'            4 passed, 0 skipped
```

Per fixture:

```
…__variant_annotation.wat                    --check EXIT 0
…__variant_flows_to_enum_param.wat           --check EXIT 0        ← the widest control
…__enum_does_not_flow_to_variant_param.wat   --check EXIT 1, message names Colour::Red and is
                                                                   NOT UnknownNamedType
…__is_type_on_a_variant.wat                  stdout true
…__generic_variant_stays_refused.wat         --check EXIT 1, message IS UnknownNamedType
```

## Blast radius

`src/types.rs` (variant registration + edges), `src/check.rs` (`infer_enum_map_ctor`'s result type),
possibly `src/record/construct.rs`. **The corpus is the real surface**: 104 monomorphic enums and
every construction of their variants now carry a narrower type. Expect the floor to speak. If it
does, the fail-count is the progress meter and each error names the next site — but classify the
remainder BY REASON before reporting a delta, never by count.

## STOP triggers — each is a REJECTION

**STOP-1.** If `a_variant_value_still_flows_to_an_enum_parameter` goes red — STOP and report the
verbatim block. Subsumption is not carrying the narrowed ctor type, and that is the stone's
foundation, not a site to patch.

**STOP-2.** If making the direction row pass requires registering the variant as an ALIAS,
`typealias`, or anything where the two types are mutually assignable — STOP. A synonym passes four
of the five rows and destroys the only reason the type is worth having.

**STOP-3.** If `a_generic_enums_variant_stays_refused` changes at all — STOP. Generic enums are out
of scope and the fence must stay exactly where it is, including its diagnostic.

**STOP-4.** If the erasure fix requires changing the runtime VALUE's shape (not just the checker's
view of it) — STOP and report what forced it. The wire form `#wat.core/Option.Some {…}` already
carries the variant; a runtime change would ripple into EDN, comms, and goldens, and that is a
different stone.

**STOP-5.** If a corpus site needs its ANNOTATION widened from the variant to the enum to keep
compiling — STOP and report the site list. That would mean subsumption is not carrying values the
way this design claims, and the count is a finding, not a worklist.

## Tier

You edit and report. Run the four targeted probe binaries and the five per-fixture checks.
**The orchestrator runs the floor and clippy centrally, once, after the tree is quiescent.**
Report what only you can: which registration site you used, whether the runtime twin needed
touching, and what the corpus said.
