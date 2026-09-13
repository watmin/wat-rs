# SCORE 2a1b — one membership door: `type-of` answers every name `is-type?` admits

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `b928367ed` BRIEF 2a1b.

```
floor  scripts/floor.sh   .floor/2026-09-13T09-51-06Z
       Summary [ 210.449s] 5431 tests run: 5431 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

No STOP. First full floor was green. E5's required red was a targeted nextest of the
agreement wall (Builtin arm temporarily raised); captured, not a floor red, not re-run
as floor. Builtin arm restored. Clippy 0.

`:wat::core::Record` is a TypeDef (opaque zero-field Aggregate via `register_builtin`),
an `is_builtin_primitive` name, AND a typesub parent (`:wat::holon::Record` is-a it).
The classifier's order answers **Declared**, not Builtin (the trap-door expected
Builtin). `:wat::core::Option`/`Result` are Declared (defenum wins over
`BARE_CONTAINER_HEADS`).

## Shape

| # | result |
|---|---|
| 1 | **PASS.** `TypeMembership` on `TypeEnv`: Declared(`&TypeDef`) / Builtin / Marker{children} / Unknown. Order: `get` Some, then `builtin_names` ∪ `is_builtin_primitive`, then `is_subtype_parent` (children = `subtype_edges` keys listing it, sorted). Canonicalizes `:wat::type::` → `:wat::core::` first. `is_known_type` = not Unknown. Builtins keep `get` = None; no TypeDef fabricated. |
| 2 | **PASS.** `eval_type_of` asks `types.classify` then `type_info_for_membership` (ONE function beside `type_info_value`). Declared → `type_info_value` unchanged. Builtin → kind `:Builtin`, empty type-params, `TypeBody.Builtin []`. Marker → kind `:Marker`, `TypeBody.Marker [children]`. Unknown → `unknown type '{kw}'`. |
| 3 | **PASS.** `wat/runtime-typeinfo.wat`: `TypeKind` gains `:Builtin` `:Marker` (unit). `TypeBody` gains `:Builtin []` and `:Marker [children <- Vector of keyword]`. Doc: a Builtin row's type-params is empty because a builtin's structure is not declared (`builtin_names` holds names only). Rust registers via `wat_enum_register_from!`. |
| 4 | **PASS.** Checker `type-of`: literal `WatAST::Keyword` is type-position (skip infer). Non-literal inferred as today. |
| 5 | **PASS.** `subtype?` known-check is classify not Unknown. `(:wat::core::subtype? :probe::Rec :probe::Marker)` → true. |

## Pins

| pin | result |
|---|---|
| six-kinds | `tests/reflection/probe_arc296_type_of_six_kinds.wat` exhaustive match gains Builtin + Marker arms. `:user::main` appends Vector / i64 / HashMap-from-string / Marker. Exact stdout: `"Aggregate"\n"Enum"\n"Newtype"\n"Alias"\n"Union"\n"Surface"\n":left"\n":right"\n"Record"\n":alpha"\n"T"\n"Builtin"\n"Builtin"\n"Builtin"\n"Marker"\n":probe::Rec"`. Keyword `str` of an FQDN child prints `:probe/Rec` (Clojure `/`); fixture concatenates `":"` + `keyword::to-string`. |
| subtype? | `tests/reflection/probe_subtype_marker.wat` prints `"true"`. |
| unknown | `type_of_unknown_name_raises`: exact reason `unknown type ':nope::Nothing'`. |
| agreement wall | `check::tests::type_of_answers_every_is_type_name` enumerates `types.iter` keys + `builtin_leaf_names` + `BUILTIN_PRIMITIVES` (colon-prefixed) + `subtype_parent_names` from the stores, never a list. Asserts classify not Unknown and `type_info_for_membership` does not raise. Went RED once (verbatim below). GREEN on the floor. |

`declared-types` is unchanged (E7 is the orchestrator's corpus re-run).

## EXPECTATIONS

| # | result |
|---|---|
| E1 | **PASS** on tracked ground: six-kinds `type-of :wat::core::Vector` → `"Builtin"`. Orchestrator re-runs the bootstrap defect probe. |
| E2 | **PASS** on tracked ground: six-kinds `type-of :wat::core::i64` (literal) → `"Builtin"`; it passes check. |
| E3 | **PASS.** subtype? Rec Marker true; `type-of :probe::Marker` → Marker, children `[:probe::Rec]`. |
| E4 | **PASS.** `unknown type ':nope::Nothing'`. |
| E5 | **RED once, then restored.** Builtin arm raised `unknown type`; wall panicked on `:wat::stream::Stream`. Verbatim below. GREEN on the floor. |
| E6 | **PASS.** `is_known_type` delegates to classify. `eval_type_of` and `eval_subtype` ask classify. Remaining `is_builtin_primitive(` sites: `types.rs` classify, `declare/typevar.rs` (pre-existing), `runtime.rs:9828` conforms? (pre-existing). No new inline union. `BUILTIN_PRIMITIVES: &[&str]` extracted next to `is_builtin_primitive`. |
| E7 | Not run here (orchestrator: corpus.sh + verify-refute2.sh). Door unchanged. |
| E8 | **PASS.** Two new arms; four new rows appended; exact stdout green. |
| E9 | Table below. Both directions, from the stores, with counts. |
| E10 | **5431 passed, 0 failed. Clippy 0.** 5428 at 2a1 close + 3 (`type_of_unknown_name_raises`, `type_of_answers_every_is_type_name`, `subtype_marker_is_true_after_derive`). |

## E5 — the red, verbatim

Targeted: `cargo nextest run --release -E 'test(=type_of_answers_every_is_type_name)'` with the Builtin arm of `type_info_for_membership` raising `unknown type`. Captured, not a floor, not re-run as floor. Arm restored.

```
thread 'check::tests::type_of_answers_every_is_type_name' (2654197) panicked at src/check.rs:24048:33:
type-of :wat::stream::Stream: Diagnostic(#wat.runtime/MalformedForm {:message "malformed :wat::runtime::type-of form: unknown type ':wat::stream::Stream'" :location #wat.core/Span {:file "src/check.rs" :line 24035 :col 20 :end #wat.core/Option.None {}} :causes [] :head ":wat::runtime::type-of" :reason "unknown type ':wat::stream::Stream'"})
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test check::tests::type_of_answers_every_is_type_name ... FAILED

failures:

failures:
    check::tests::type_of_answers_every_is_type_name

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1223 filtered out; finished in 0.15s

        FAIL [   0.158s] (1/1) wat check::tests::type_of_answers_every_is_type_name
────────────
     Summary [   0.164s] 1 test run: 0 passed, 1 failed, 5452 skipped
        FAIL [   0.158s] (1/1) wat check::tests::type_of_answers_every_is_type_name
error: test run failed
```

## Set difference — `is_builtin_primitive` (37) vs `builtin_names` (`register_builtin_types`)

The classifier unions them, so `is-type?` and `type-of` agree whatever the difference is.
Reconciling the two stores is its own stone.

`is_builtin_primitive` not in `builtin_names` (**14**):

| name |
|---|
| `:wat::core::Record` |
| `:wat::core::Tuple` |
| `:wat::core::char` |
| `:wat::core::fn` |
| `:wat::core::nil` |
| `:wat::holon::Engram` |
| `:wat::holon::EngramLibrary` |
| `:wat::holon::OnlineSubspace` |
| `:wat::holon::Reckoner` |
| `:wat::kernel::ChildHandle` |
| `:wat::kernel::HandlePool` |
| `:wat::kernel::ProgramHandle` |
| `:wat::kernel::Receiver` |
| `:wat::kernel::Sender` |

`builtin_names` not in `is_builtin_primitive` (**12**):

| name |
|---|
| `:rust::crossbeam_channel::Receiver` |
| `:rust::crossbeam_channel::Sender` |
| `:wat::core::PersistentMap` |
| `:wat::core::PersistentVector` |
| `:wat::core::Value` |
| `:wat::kernel::Address` |
| `:wat::kernel::Listener` |
| `:wat::kernel::Peer` |
| `:wat::kernel::Process` |
| `:wat::kernel::Thread` |
| `:wat::kernel::ThreadSelfPeer` |
| `:wat::stream::Stream` |

## Blast radius

`src/types.rs` (classifier; `is_known_type` = not Unknown) · `src/reflect/verbs.rs`
(`type_info_for_membership` + `eval_type_of`) · `src/runtime.rs` (`BUILTIN_PRIMITIVES`;
`subtype?` known-check) · `src/check.rs` (type-of literal type-position; unknown pin;
agreement wall) · `wat/runtime-typeinfo.wat` (`:Builtin` `:Marker`) ·
`tests/reflection/probe_arc296_type_of_six_kinds.wat` ·
`tests/reflection/probe_arc296_reflection_answers_for_every_type_kind.rs` ·
`tests/reflection/probe_subtype_marker.wat`. Not pushed.
