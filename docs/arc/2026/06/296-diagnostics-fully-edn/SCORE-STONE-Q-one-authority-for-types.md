# SCORE — STONE Q: one authority that can answer for every type

No commit. Floor left to the orchestrator. Lands on 255's TypeEnv membership door
and 296 L's `type-of` (structure). Those were not reverted.

The scalars are not missing from the registry. They are a different mechanism.
`type-of` asks `TypeEnv::get`. Membership is `contains` ∪ `is_builtin_primitive`.
The wat verb is `:wat::runtime::is-type?`.

## Verdict — (A)

One query, two stores, nothing moves.

```
is-type? name  =  TypeEnv::contains(name)  ∨  is_builtin_primitive(stripped)
```

- **(B)** not this stone. Completing 255's leaf list so `contains` equals the union
  (`char`, `Tuple`, …) is a follow-up. It would register names, not fabricate TypeDefs.
- **(C)** the trap. `get` stays `None` for leaves. Census asserts it.

`type-of` stays the structure verb. A variant name is `false` today; P-2 makes it true.

The table is `TABLE-STONE-Q-the-mechanisms.md`. The executable census is
`types::tests::stone_q_census_contains_is_membership_get_is_structure`.

## The five Doctrine 1 could not see, read from source

| keyword | get | contains | prim |
|---|---|---|---|
| `:wat::core::i64` | None | yes | yes |
| `:wat::core::f64` | None | yes | yes |
| `:wat::core::bool` | None | yes | yes |
| `:wat::core::String` | None | yes | yes |
| `:wat::core::nil` | Alias | yes | yes |

nil has structure. The other four are leaves. `type-of` cannot receive them because
its checker **infers the arg as a value**. `is-type?` skips inference (type position,
same convention as `subtype?`).

## Probe

`probe_arc296_is_type`: **1 passed**, `#[ignore]` = 0.

Stdout of `tests/reflection/probe_arc296_is_type.wat` (EXIT=0, `--check` EXIT=0):

```
true
true
true
false
```

primitive `:wat::core::i64` · builtin `:wat::core::Vector` · user `:usr::Shape` ·
nonexistent `:usr::TotallyMadeUp` **false**. That last is the row.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 shape before table | **held.** Table written first. Census test locks `contains`/`get`. Verb is the union the table named. |
| STOP-2 primitives into TypeEnv | **held.** `get(":wat::core::i64")` is still `None`. |
| STOP-3 Rust-only | **held.** A wat program asked. Four bools came back. |
| STOP-4 Doctrine 1 weakened | **held.** `probe_arc296_is_type__doctrine1.wat.bad` `--check` EXIT=1: `Doctrine 1 (arc 242): ':wat::core::i64' is a TYPE keyword, not a value`. |
| STOP-5 P-1 built | **held.** Annotation wall not touched. |

## Named, not patched

`subtype?` still asks `get` OR `is_builtin_primitive`, not `contains`.
`:wat::core::PersistentVector` is a leaf and not in the primitive table, so
`subtype?` would raise "unknown type" for a name `is-type?` answers true.
The two predicates do not share an authority yet. Out of scope; Q does not
quietly "fix" `subtype?`.

`:wat::core::Fn` is a type **shape** (`TypeExpr::Fn`), not a named member.
Both stores say no. Lowercase `:wat::core::fn` is the value type and is prim.

## Targeted checks

```
cargo test --release --lib stone_q_census…     1 passed
cargo test --release --test reflection is_type_answers…  1 passed
cargo test --release --test reflection reflection_answers_for_an_enum  ok
cargo test --release -p wat-doc -p wat-macros -p wat-edn -p wat-reader  ok
cargo nextest run --release --test lint        125 passed
cargo clippy --release --all-targets --workspace  0 errors, 5 pre-existing dead-code warnings
./target/release/wat --check tests/reflection/probe_arc296_is_type.wat  EXIT=0
./target/release/wat --check tests/reflection/probe_arc296_is_type__doctrine1.wat.bad  EXIT=1
```

Floor **orchestrator**.

## Working tree

```
docs/arc/2026/06/296-diagnostics-fully-edn/TABLE-STONE-Q-the-mechanisms.md
src/check.rs                 infer_list skip-infer + TypeScheme
src/reflect/verbs.rs         #[wat_intrinsic] is-type?
src/runtime.rs               is_builtin_primitive pub(crate); comment names the parallel set
src/types.rs                 census test
tests/reflection/probe_arc296_is_type.rs
tests/reflection/probe_arc296_is_type.wat
tests/reflection/probe_arc296_is_type__doctrine1.wat.bad
```

Do not commit unless a later brief says to.
