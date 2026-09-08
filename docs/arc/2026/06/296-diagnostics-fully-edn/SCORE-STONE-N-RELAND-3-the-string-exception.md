# SCORE — STONE N RELAND 3: the string exception, and one site the scream named

No commit. Floor left to the orchestrator. Lands on N RELAND-2's uncommitted tree.

## Population (1) — rust-embedded wat. String exception. Hand-edited.

R21 does not govern. No wat-fix script was added. Rename and wrap landed in the same edit:

```
(:wat::core::Some 3)     ->  (:wat::core::Option::Some {:value 3})
[:wat::core::Some {:value v} …] -> [:wat::core::Option::Some {:value v} …]
[:wat::core::None {} …]  ->  [:wat::core::Option::None {} …]
(:wat::core::Ok x)       ->  (:wat::core::Result::Ok {:value x})
(:wat::core::Err e)      ->  (:wat::core::Result::Err {:error e})
:wat::core::None         ->  :wat::core::Option::None   (value position)
```

Wall intact: `retired_bare_variant`, `#[wat_intrinsic(":wat::core::Some")]`, `if k == ":wat::core::None"` unchanged.

Files: `@example` / `@example-norun` in `src/intrinsic/**`, `src/reflect/*.rs`, `src/collection/transform.rs`, `src/rete/step_payload.rs`; inline test programs in `src/runtime.rs`; `tests/process/probe_supervisor_select_lost.rs`.

Two `:wat::core::Error` tokens were almost eaten by a prefix replace of `:wat::core::Err` (`:wat::core::Result::Error`). Caught and restored in `kernel/error.rs` and `record.rs`.

## Population (2) — the scream. One authorised `.wat` hand-edit.

`tests/macros/probe_arc278_macro_generates_service.wat:43`

```
(:probe::Echo::EchoResponse::Ok c)
-> (:probe::Echo::EchoResponse::Ok {:n c})
```

Why the codemod declined: the `defenum` lives **inside the defmacro's quoted body**. `fmap-for-src` / `eval-with-defs!` never see it — same class as M2's generated unquote heads. Field name is `n` (that nested defenum: `:Ok [n <- :i64]`), not `reply`. `--check` of the fixture: EXIT=0, 0 fixture-local path hits.

`probe_arc278_macro_generates_service`: **PASS**.

## Stepper — Map is canonical

After qualification, `step_match_canonical` went red: `is_match_canonical` treated `(:Option::Some {:value 5})` as a call to reduce (`items[1]` is a Map, not a literal). Arc 296 M's ONE grammar. Added: a Map whose keys are keywords and whose values are canonical is itself canonical. `step_match_canonical` and `step_round_trip_agrees_with_eval_ast`: **PASS**.

## WalkStep — same string exception, named by the tests

Four `runtime::tests::walk_w*` programs still constructed `WalkStep::Continue` / `::Skip` positionally. The Option/Result pass did not touch them. Wrapped (`{:acc …}`, `{:terminal … :acc …}`). All four **PASS**.

## The 22 `non-exhaustive`

**None survived in the tests this strike ran.** They were the same Option match sites; qualifying the arms restores `MatchShape::Option`. 364 tests (`runtime::tests::*` + the five probe rows + the macro probe + supervisor): **364 passed**. Floor not run — if a `non-exhaustive` appears there it is a FINDING, not residue.

## Probe / clippy / check

- `probe_arc296_enum_map_ctor`: **5 passed**
- `cargo clippy --release --all-targets --workspace`: **0 errors**, 5 pre-existing dead-code warnings
- `--check tests/macros/probe_arc278_macro_generates_service.wat`: EXIT=0

## Floor

**Not run.** Orchestrator. Pre-strike (central): 5110 passed · 128 failed.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 rename without wrap | **held.** One edit, map ctor included |
| STOP-2 rust-string codemod shipped | **held.** No `wat-scripts/fixes/` added |
| STOP-3 extra `.wat` hand-edit | **held.** Only the screamed defmacro site |
| STOP-4 surviving non-exhaustive called residue | **held.** None seen in 364; floor is the remaining witness |
| STOP-5 wall weakened | **held.** |

## What landed (this stone)

- rust-embedded wat: qualified + wrapped Option/Result
- `src/runtime.rs` `is_match_canonical`: Map
- `src/runtime.rs` walk fixtures: WalkStep map ctor
- `tests/macros/probe_arc278_macro_generates_service.wat:43`: `{:n c}`
