# SCORE-AMEND — STONE 255.1: the membership list is NARROWER than the alias it replaced

Folded into `cf94c3a93` (amended, not a repair commit). **Not pushed.**
Parent: `AMEND-255.1-the-membership-list-is-narrower-than-the-alias.md`.
Floor / workspace clippy / census / 20-name sweep / 179-file delta: orchestrator
re-runs uncontended. Crate clippy + the three walls run here.
Do not start 8d-ii. No `.wat` converted.

## CAUSE 1 — membership is derived from denotation, not a hand-list

The old `canonicalize_type_kw` forwarded **every** `:wat::type::X` to `:wat::core::X`
and then asked `types ∪ builtin_names ∪ is_builtin_primitive`. Dual-inserting
wat.type tails only for `register_builtin_leaf` of `:wat::core::` leaves was
narrower than that — `Tuple` `char` `Record` `Struct` `Error` `Infer` (and
`Bogus` as a non-member) screamed.

**The door:** `contains` / `get` / `classify` ask the **denotation**
(`type_denotation`: wat.type/X → wat.core/X) of the same stores the alias
consulted, plus `INFER_TYPE_PATH` (wat.type-only sentinel). No hand-list of
tails. A future core builtin is a wat.type member automatically.

Dropped the dual-insert in `register_builtin_leaf` and the extra
`:wat::type::nil` seed. `nil` is a member because `:wat::core::nil` is.

`wat.type/Bogus` / `wat.type/nope` stay non-members (non-vacuity holds).

## CAUSE 2 — the three walls

| wall | fix |
|---|---|
| `one_variant_separator` | rune `namespace` on rust-scheme `::` detectors / clojure→key `format!`s in `canonical_identity` (edn/render + wat-source-derive). Not enum/variant. |
| `one_param_spec` | **not runed.** `is_binder_marker` lives in `wat-reader` (both `types.rs` and `wat-source-derive` call it; derive cannot import `wat`). `peel_param_spec` stays in `types.rs`. Lint skips `crates/wat-reader/src/ast.rs` as the shared node test. |
| `no_loose_string_assert` | membership tests use `assert!(env.is_known_type(…))` (not `TypeEnv::contains("…")` inside `assert!`). Probe diagnostic still `contains` the refusal prose with a same-line `loose-assert` rune — the dump is a freeze error, not a golden. |

`cargo test --release -p wat --test lint` filters:
`only_identifier_rs_spells_the_variant_separator`, `only_types_rs_peels_a_param_spec`,
`tests_carry_no_loose_string_assert` — all pass.

## One position, not two — re-attempted, freeze held

After CAUSE 1, `normalize`'s existing `:wat::type::` + `is_known_type` block
accepts members as names. `is_resolvable_call_head` now consults TypeEnv for
`:wat::type::` members (the same question). 

| | |
|---|---|
| `wat.type/Vector` annotates | `--check` rc=0 |
| `(wat.type/Vector 1 2)` call | `--check` rc=0 |
| `(wat.type/nope)` call | `UnresolvedReference` (non-member) |
| original `counter-actor-proof-process.wat` | rc=0 (freeze held) |

Not smuggled: the consult is `starts_with(":wat::type::") && is_known_type`.
A complete membership list was what the predicate needed.

## Identity must not prefix a rendered form

`canonical_identity` of a rust-scheme path without a leading `:` gains `:`.
A **rendered** parametric `(:wat::capability::Dialable :- […])` contains `::`
but starts with `(`. Prefixing it made `type_of_answers_every_is_type_name`
classify Unknown. Early-return if the string starts with `(` .

## Walls I ran

- `cargo clippy --release --all-targets -p wat --offline -- -D warnings` — **exit 0**
- three wall tests above — pass
- `stone_255_1_*`, `identity_is_the_pair_not_the_spelling`, `probe_255_1_identity` — pass
- `probe_arc251_keyword_to_type_form` — **9/9** (was 9 of the 18 CAUSE-1 reds)
- `type_of_answers_every_is_type_name` — pass

Floor + workspace clippy + census + 20-name sweep + 179-file delta: **not run**.
Do not push. Do not start 8d-ii.
