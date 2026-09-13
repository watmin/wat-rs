# SCORE 2a1 REFUTE 2 — one-step walk; no name list; tracked ground

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `101626eea` SCORE 2a1 REFUTE.

```
floor  scripts/floor.sh   .floor/2026-09-13T08-55-13Z
       Summary [ 235.099s] 5428 tests run: 5428 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

No STOP. No name list remains in the door.

## R5

| pin | result |
|---|---|
| walk | After `register_defmacros` + `preregister_acronyms` on the whole rest: `classify_type_decl` keeps; do/let walk BODY via `container_body_start` (pub(crate)); a registered macro is `expand_once` then walked; else dropped, never expanded. Then `expand_all` → register. |
| holon records | `tests/types/probe_arc234_stone2a_record_primitives.wat`: `:myapp::Voltage [magnitude]`, `:myapp::Point [x y]`. |
| kwargs | Inline `:t::work [x & [n]]` returns `:t::work::Kwargs [n]`. |
| let / macro | `tests/macros/probe_let_splice_enum_via_macro.wat`: `:my::probe::Event` `Created [id]`, `Deleted [id]`, `NoOp`. |
| c3 | still `:demo::Req [n]`, `:demo::Op` `Go [req]`. |
| body never expanded | `defn` body holds `(:wat::core::defstruct)`; door returns `Ok` with `:t::Ok`. |
| deleted | `PRE_EXPANSION_TYPE_FORMS`, `keeps_declared_types_form`, the two filter tests. |

## R6

| pin | result |
|---|---|
| timing | Tracked `tests/services/probe_arc278_sift_rules.wat`: **16.2 ms**. `SiftRulesResponse` still registers. |
| stale body | R5's inline `declared_types_body_is_never_expanded`. |
| lint | `tests/lint/no_bootstrap_path_in_committed_rust.rs`. Went RED once on `src/check.rs:23532` and `:23563` (verbatim below). Those literals removed. |

## R7

Oracle (`type-of` after `startup_from_source` == door `type_info_value`) extended with `:myapp::Voltage`, `:myapp::Point`, and `:t::work::Kwargs`. All three load.

## R6 — the red, verbatim

```
thread 'no_bootstrap_path_in_committed_rust::committed_rust_does_not_literal_bootstrap_paths' (1801216) panicked at tests/lint/no_bootstrap_path_in_committed_rust.rs:60:5:
committed Rust under src/ or tests/ contains a string literal beginning "bootstrap/" (gitignored; empty on a clone): ["src/check.rs:23532", "src/check.rs:23563"]
```

## Blast radius

`src/freeze/env.rs` (one-step walk) · `src/macros/expand.rs` (`container_body_start` pub(crate)) · `src/check.rs` (tracked pins) · `tests/lint/no_bootstrap_path_in_committed_rust.rs`. Not pushed.
