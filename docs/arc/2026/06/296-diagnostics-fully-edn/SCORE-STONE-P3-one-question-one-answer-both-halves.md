# SCORE — STONE P-3: one question, one answer — both halves

No commit. Floor and clippy left to the orchestrator. Lands on P-1 RELAND-1
and the P-2 prereq. Nothing reverted. Neither half changes what is a type;
both change who is asked.

## Half 1 — coverage is per declaring scope

`stdlib_use` and `user_use` stay unmerged. The wall picks per declaration:

```
src/check.rs:15266
    // SCOPE SELECTION, not the reserved-prefix SKIP RELAND-1 deleted.
    // Every declaration is still validated; only the reference set differs.
```

`is_reserved_prefix(name)` selects the set. No `continue`. TypeDef and
Function store no origin/`Privilege` — I looked; prefix IS the scope because
user source cannot define under `:wat::*` / `:rust::*` (an existing wall).

A user annotation of `:rust::sqlite::Connection` without a user `use!` is
now refused, matching resolve's call-head answer. Stdlib annotations still
see stdlib `use!`.

## Half 2 — `is-type?` asks store 4

One disjunct on `eval_is_type`: `|| types.is_subtype_parent(&type_kw)`.
`derive` was not taught to validate its marker (STOP-5 held).

The wall and the verb still do not share one predicate. The wall also asks
`covers(scope)`; the verb must not, or stdlib `use!` would leak into
`is-type?` (P-2 STOP-1). A fifth store would have to change both sites.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 `the_stdlib_still_loads` red | **held.** stdout `"loaded"` |
| STOP-2 skip / allow-list / continue | **held.** No `continue` in the validator. Selection, not skip. |
| STOP-3 non-marker becomes a type | **held.** stdout `false` |
| STOP-4 corpus refuses | **held.** 845 files, 0 UnknownNamedType |
| STOP-5 derive must validate its marker | **held.** Not touched. |

## Expectations

| # | result |
|---|---|
| 1 | without user `use!`: EXIT=1, path `:rust::sqlite::Connection` |
| 2 | with user `use!`: EXIT=0 |
| 3 | stdlib still loads: stdout `"loaded"` |
| 4 | `is-type? :wat::spawn::Spawned`: stdout `true` |
| 5 | `is-type? :usr::NeverDerivedTo`: stdout `false` |
| 6 | `test(p3_one_question)` **5 passed, 0 skipped** |
| 7 | `test(p1_annotation)` **10 passed, 0 skipped** |
| 8 | `test(p2prereq)` **4 passed, 0 skipped** |
| 9 | no skip arm; SCOPE SELECTION comment at `src/check.rs:15266` |
| 10 | corpus `wat/` `wat-scripts/` `wat-tests/`: **845 / 0** |
| 11 | refuse together without a `use!`; accept together with one |

## Targeted checks

```
./target/release/wat --check …__user_annotation_without_user_use.wat   EXIT=1  :rust::sqlite::Connection
./target/release/wat --check …__user_annotation_with_user_use.wat      EXIT=0
./target/release/wat …__stdlib_annotation_still_loads.wat              stdout "loaded"
./target/release/wat …__is_type_on_a_derive_marker.wat                 stdout true
./target/release/wat …__is_type_on_a_non_marker.wat                    stdout false
cargo nextest run --release -E 'test(p3_one_question)'   5 passed, 0 skipped
cargo nextest run --release -E 'test(p1_annotation)'     10 passed, 0 skipped
cargo nextest run --release -E 'test(p2prereq)'          4 passed, 0 skipped
```

Floor **orchestrator**. Clippy **orchestrator**.

## Sites inspected

- Probe + five fixtures
- `src/freeze/env.rs` stdlib collection vs user collection (already distinct)
- `src/check.rs` wall — two loops, each entry's NAME is the scope
- `src/resolve/walk.rs:104-124` call-head coverage (user residue only)
- `src/resolve/reserved.rs` `is_reserved_prefix`
- `src/types.rs` TypeDef — no origin field; Privilege is a registration gate, not stored
- `src/value/environment.rs` Function — no origin field; `synthesized_for` is a companion mark, not a scope
- `src/reflect/verbs.rs` `eval_is_type`
- `src/types.rs:847` `is_subtype_parent`

Trap-door 2 (generated `:wat::*` names whose annotations came from user source):
not observed. Generated companions take the type's own FQDN (`:usr::Point/x`),
which is not reserved, so they take `user_use`.

## What surprised

Nothing about the two halves. The prefix-as-scope reading is load-bearing
exactly because origin is not stored — if a generated function ever landed
under `:wat::*` with a user annotation, the wall would ask `stdlib_use` for
it. That shape was not found.

## Working tree

```
src/freeze/env.rs     stdlib_use / user_use unmerged; wall takes both
src/check.rs          scope_decls per declaration; SCOPE SELECTION comment
src/reflect/verbs.rs  is_subtype_parent disjunct
tests/types/probe_arc296_p3_one_question_one_answer.rs  both subjects un-ignored
```

Do not commit unless a later brief says to.
