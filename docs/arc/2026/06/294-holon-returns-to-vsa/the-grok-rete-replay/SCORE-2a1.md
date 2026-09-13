# SCORE 2a1 — the declaration door: expand, register, return, with no startup

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Binary: `./target/release/wat` (boots). Floor + clippy run.

```
floor  scripts/floor.sh   .floor/2026-09-13T07-21-27Z
       Summary [ 211.266s] 5421 tests run: 5421 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

First floor `.floor/2026-09-13T07-11-36Z` was RED on two intrinsic gates for the new verb
(no `CheckEnv` scheme; no runnable `@example`). Captured, not re-run. Fixed. Second floor
green. Clippy then refused `DeclaredTypesFail` as a large `Err`; boxed it. No STOP.

## EXPECTATIONS

| # | result |
|---|---|
| E1 | **PASS.** sift_rules `:usr::my-sift::SiftRulesResponse` variants in declaration order (`Deductions [items, cursor]`, `Fatal [err]`, `RequestTooLarge [bytes, cap]`, `RequestMalformed [path, expected, got]`). W2f `:probe::Echo::EchoResponse` (`Ok [reply]`, `RequestTooLarge`, `RequestMalformed`). No `check_program`. |
| E2 | **12.3–16.6 ms** on sift_rules (`D1 sift_rules register_declared_types` eprintln). An `eval-with-defs!` turn is ~460 ms. |
| E3 | **lazy OnceLock.** `stdlib_snapshot()` is `build_env(vec![])` behind `OnceLock`. Same pointer on two calls. Stdlib AST walk: no list headed by `:wat::runtime::declared-types` (a comment naming the verb is not a call). Construction is not inside a lock stdlib expansion can re-enter. No empty-file `--check` delta: not captured at startup. |
| E4 | **PASS.** `:wat::runtime::declared-types` is pure, no `!`. `@arg`/`@ret`/`@example`/`@example-norun`/`@see`. `type_info_value` is the one TypeInfo constructor; `eval_type_of` calls it. |
| E5 | **PASS.** Cases 1–7: plain defenum+defrecord; defsurface Op/Reply; CreateWebACL; sift-rules-defsvc enum; stale defservice; isolation; malformed → `DeclaredTypes.Refused` naming the form and cause. |
| E6 | **PASS.** Colour's `type-of` after `startup_from_source` == `type_info_value` from the door. sift_rules / W2f do not load (stale bodies); they are E1, not the oracle. |
| E7 | **PASS.** `:t::Box` with `left` vs `right`: each call sees only its own. |
| E8 | **RED, then restored.** Shared one TypeEnv across calls. Case 6 unwraps `DuplicateType :t::Box`. Verbatim below. |
| E9 | **5421 passed, 0 failed. Clippy 0.** 5411 at ingest + 10 new tests. |

## E8 — the red, verbatim

```
thread 'check::tests::declared_types_isolation_same_name_different_fields' (278471) panicked at src/check.rs:23667:85:
called `Result::unwrap()` on an `Err` value: DeclaredTypesFail { form: List([Keyword(":wat::core::defrecord", …), Keyword(":t::Box", …), Vector([Symbol(Identifier { name: "right", …}), …])], …), cause: "#wat.type/DuplicateType {:message \"duplicate type declaration: :t::Box\" :location #wat.core/Span {:file \"src/check.rs:23662\" :line 1 :col 1 :end #wat.core/Option.Some {:value #wat.core/Pos {:line 1 :col 62}}} :causes [] :name \":t::Box\"}" }
```

The copy is the isolation. Restored to `stdlib_types.clone()` per call.

## First floor (captured, not re-run)

`.floor/2026-09-13T07-11-36Z/` Summary [212.774s] 5421 tests run: 5419 passed, 2 failed, 22 skipped.

Arms:

1. `intrinsic::tests::checker_skip_debt_is_named_and_frozen` at `src/intrinsic/mod.rs:1486` —
   `NEW — registered but absent from CheckEnv: [":wat::runtime::declared-types"]`.
   Fix: rank-1 `TypeScheme` in `register_builtins` (Vector of WatAST → DeclaredTypes), plus an
   `infer_list` arm beside `type-of`. Not parked on the debt ledger.
2. `intrinsic::tests::purity_mandated_examples` at `src/intrinsic/mod.rs:2834` —
   `pure+det intrinsic ':wat::runtime::declared-types' has no runnable @example`.
   Fix: `@example (:wat::core::variant-name (:wat::runtime::declared-types (:wat::core::Vector :- [:wat::WatAST]))) #=> "Ok"`.
   `@example-norun` kept for the quoted-defenum form.

## D1 — not STOP-1

`register_declared_types` is `build_env`'s USER half on copies of the stdlib snapshot:
`register_defmacros` → `preregister_acronyms` → `expand_all` → `register_types_with_acronyms` →
`register_variant_types`. Stops before `register_defines` and `check_program`.

Whole-file expansion of sift_rules eval'd `:wat::query::mem-store/start` (`ProgramBodyEvalFailed`).
The door filters to type-registering surface forms (`defenum`/`defrecord`/`defsurface`/`defservice`/
`sift-rules-defsvc`/`declare-acronyms`/…). Not `declare::parse::is_declaration_form` (runtime
`def`/`defclause` residue).

## D2 — lazy, not STOP-2

`stdlib_snapshot` / `stdlib_loaded` share one `OnceLock`. The initializer is `build_env` itself.
The verb clones that snapshot per call. Stdlib source has no call of the verb (AST walk of list
heads). The enum `:wat::runtime::DeclaredTypes` and a comment naming the verb are not calls.

## D3 — the verb

`:wat::runtime::declared-types` → `:wat::runtime::DeclaredTypes` (`Ok [types]` / `Refused [form, cause]`).
wat is the source of truth (`wat/runtime-typeinfo.wat`); Rust registers via `wat_enum_register_from!`.
Failed declaration is a structured refusal, never a panic, never a silent drop.

## Blast radius

`src/freeze/env.rs` (snapshot + door) · `src/reflect/verbs.rs` (`type_info_value` + the verb) ·
`src/check.rs` (tests + scheme + infer arm) · `src/types.rs` (one register line) ·
`wat/runtime-typeinfo.wat` (DeclaredTypes). Not pushed.
