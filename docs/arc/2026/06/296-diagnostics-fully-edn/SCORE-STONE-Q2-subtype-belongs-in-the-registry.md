# SCORE — STONE Q2: `subtype?` belongs in the registry

No commit. Floor left to the orchestrator. Lands on Q (`is-type?`) and 255's
registration of `conforms?`. Those were not reverted.

The `@see` wall was right. The `@see` was also right. `subtype?` was live and
unregistered. 255 registered its twin.

## Census — SMALL, an omission

`TABLE-STONE-Q2-check-arms-without-registration.md`. 144 arm occurrences, 133
unique FQDNs, **23** without `#[wat_intrinsic]` / `#[wat_special_form]`.

Not 143. Not 255's special-forms bucket. Register `subtype?`. The other 22 are
a short list (declaration heads, retired constructor spellings, pre-registry
collection ops, expect/try aliases, three kernel names). Not this stone.

## Registration

`#[wat_intrinsic(":wat::core::subtype?")]` on `eval_subtype`, same
variadic-sniff / single-`@arg` shape as `conforms?` (arc 255 Stone 1c-a-ii).
Dispatch arm retired. Checker arm and TypeScheme untouched. Doctrine 1
untouched. Known-name check still `get` OR `is_builtin_primitive` (Q named
that drift; Q2 does not quietly fix it).

The `@see` on `:wat::runtime::is-type?` **stayed**. The wall is green because
`subtype?` now resolves, not because the reference moved.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 census before registration | **held.** Table first. 23, not 143. |
| STOP-2 `@see` removed or repointed | **held.** `src/reflect/verbs.rs:1545` still `@see :wat::core::subtype?`. |
| STOP-3 wall loosened | **held.** `all_see_fqdns_resolve_to_registered_intrinsics` PASS. |
| STOP-4 new registration shape | **held.** `conforms?`'s variadic / one-`@arg` mechanics, copied. |
| STOP-5 Doctrine 1 touched | **held.** Checker still skip-infers both keyword args. |

## Targeted checks

```
all_see_fqdns_resolve_to_registered_intrinsics              PASS
registry_membership_gap_a_is_named_and_frozen               PASS  (subtype? left GAP_A)
the_residues_cannot_shadow_the_registry                     PASS  (left expand-time residue)
registry_first_door_owns_every_handler_row_no_literal_arm   PASS
every_dispatched_verb_is_classified_or_disposed             PASS  (left KNOWN_UNREVIEWED)
probe_arc237_sA_hierarchy                                   10 passed  (incl. probe 10 unknown-name raise)
probe_arc296_is_type                                        1 passed
cargo test --release -p wat-doc -p wat-macros -p wat-edn -p wat-reader  ok
cargo nextest run --release --test lint                     125 passed
cargo clippy --release --all-targets --workspace            0 errors, 5 pre-existing dead-code warnings
```

Floor **orchestrator**.

## Working tree

```
docs/arc/2026/06/296-diagnostics-fully-edn/TABLE-STONE-Q2-check-arms-without-registration.md
src/runtime.rs            #[wat_intrinsic] eval_subtype; dispatch arm retired
src/intrinsic/mod.rs      subtype? deleted from GAP_A
src/macros/eval.rs        subtype? deleted from expand-time residue
src/rete/purity.rs        subtype? deleted from KNOWN_UNREVIEWED
```

Do not commit unless a later brief says to.
