# SCORE — STONE P-2a: a monomorphic variant is a type

No commit. Floor and clippy left to the orchestrator. Lands on P-1 / P-2 prereq / P-3.
Nothing reverted of those. `src/record/construct.rs` was not touched.

## What landed

For every enum with empty `type_params`, each variant FQDN is registered as a
`TypeDef::Enum` whose structure is that variant's own fields (not a
membership-only leaf — `type-of` would otherwise answer `None` for something
that has fields), plus a subtype edge `Variant <: Enum`. Generic enums are
untouched.

Site: `TypeEnv::register_monomorphic_variant_types`, called from
`freeze/env.rs` after both stdlib and user types are registered and before
the annotation wall. `register_enum_methods` and `build_unit_variant_map`
skip the singleton TypeDefs so they do not mint `:Enum::Variant::Variant`.

Ordinary function parameters **do** consult `assignable` → Path-Path
`is_subtype` (`check.rs:16992`). Arc 209's marker bound is the same door.
That is why the probe's `takes-colour` / `takes-red` rows work.

## STOP-1 — the ctor does not stop erasing

Both halves, or neither. The map-ctor result type was changed to the variant
for the monomorphic case. Stdlib then refused to load. **299** type-check
errors. First cluster, verbatim:

```
#wat.check/TypeMismatch {:message ":wat::kernel::send: parameter payload expects :wat::query::Store::Op; got :wat::query::Store::Op::ScanIndex"
 :location #wat.core/Span {:file "wat/query/sqlite-store.wat" :line 243 :col 1
 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 336 :col 109}}}
 :callee ":wat::kernel::send" :param "payload"
 :expected ":wat::query::Store::Op" :got ":wat::query::Store::Op::ScanIndex"}
```

`infer_send_prime` (`check.rs:11619`) **unifies** payload against I. It does
not call `assignable`. The brief said if the argument position does not
consult `is_subtype`, that is a STOP, not a site to patch. Ctor retyping was
reverted. `infer_enum_map_ctor` still returns the enum.

Second cluster, same run: `if` else-branch
`RecvOutcome :- [ScanResponse::RequestTooLarge]` vs
`RecvOutcome :- [ScanResponse]` — parametric args are invariant. That is the
argument-correspondence class P-2b was fenced for; it showed up wrapping a
monomorphic variant.

Until send (and RecvOutcome's type args) accept a subtype, **no expression
types as `:usr::Colour::Red`**. The annotation is accepted. That is the
uninhabitable parameter the DESIGN said is worse than today's refusal. It is
the remainder. The probe's five bars still pass because they never construct
a value into a variant-typed parameter.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 subsumption at the ctor's new type | **fired** on send-unify. Ctor retyping reverted. Not patched. |
| STOP-2 alias / mutually assignable | **held.** Singleton Enum + one-way edge. Direction row is TypeMismatch, not 0. |
| STOP-3 generic fence moved | **held.** EXIT=1, still `UnknownNamedType`. |
| STOP-4 runtime VALUE shape | **held.** `construct.rs` not touched. |
| STOP-5 annotation widened in corpus | **held.** No corpus wat edited. |

## Expectations

| # | result |
|---|---|
| 1 | variant annotation EXIT=0 |
| 2 | flows to enum param EXIT=0 (ctor still erases, so this is the old green) |
| 3 | direction EXIT=1, names `Colour::Red`, not `UnknownNamedType` |
| 4 | `is-type? :usr::Colour::Red` stdout `true` |
| 5 | generic fence EXIT=1, `UnknownNamedType` |
| 6 | `test(p2a_a_monomorphic)` **5 passed, 0 skipped** |
| 7 | `test(p1_annotation)` **10 passed, 0 skipped** |
| 8 | `test(p2prereq)` **4 passed, 0 skipped** |
| 9 | `test(p3_one_question)` **5 passed, 0 skipped** |
| 10 | runtime unchanged | `src/record/construct.rs` not in the diff |
| 11 | floor | orchestrator |

## Targeted checks

```
./target/release/wat --check …__variant_annotation.wat                  EXIT=0
./target/release/wat --check …__variant_flows_to_enum_param.wat         EXIT=0
./target/release/wat --check …__enum_does_not_flow_to_variant_param.wat EXIT=1  TypeMismatch Colour::Red
./target/release/wat …__is_type_on_a_variant.wat                        stdout true
./target/release/wat --check …__generic_variant_stays_refused.wat       EXIT=1  UnknownNamedType
cargo nextest run --release -E 'test(p2a_a_monomorphic)'  5 passed, 0 skipped
```

Floor **orchestrator**. Clippy **orchestrator**.

## Working tree

```
src/types.rs              register_monomorphic_variant_types; is_monomorphic_variant_type; EnumVariant::name
src/freeze/env.rs         call after validate_aggregate_containment
src/declare/register.rs   skip singleton variant-types in register_enum_methods
tests/types/probe_arc296_p2a_a_monomorphic_variant_is_a_type.rs  three subjects un-ignored
```

Do not commit unless a later brief says to.
