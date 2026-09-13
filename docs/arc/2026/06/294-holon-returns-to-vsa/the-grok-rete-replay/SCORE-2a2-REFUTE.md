# SCORE 2a2-REFUTE — a nested program's own types; EnumFields; skip defn only

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `9b3846713` VERIFY+REFUTE 2a2. Committed on `replay/grok-rete`.

```
floor  scripts/floor.sh   .floor/2026-09-13T22-27-58Z
       Summary [ 234.072s] 5431 tests run: 5431 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

No STOP. R10 RED-once was a targeted nextest of `stdlib_snapshot_is_once_and_stdlib_does_not_call_the_verb` (not the floor); captured, then the temporary `defmacro` was reverted before this floor.

## R-items

| # | result |
|---|---|
| R8 | **PASS.** Each `(:wat::core::forms …)` is its own program: `enum-fields-for-forms` asks the door on the children and `walk-edits` applies that map inside the subtree only. Parent map never merges. Fixture: parent `:t::M::A` → `{:x x}`; child `:t::M::A` → `{:y y :z z}`. A type declared only inside forms and used outside stays UNRESOLVED (`:inner::E::V`). pre24 `probe-m1-ann-erase.wat`: nested `[:probe::CMsg::Setup {:addr addr}` / `Work` now KEY-FIRST (was the one real loss). |
| R9 | **PASS.** `macro-decl-head?` / `plain-type-head?` / `has-macro-decl?` / `plain-type-names` / `keep-local-eps` / `skip-local-eps` deleted. `fmap-for-src` already asked the door for every `local-eps-of` candidate after 2a2, so the filter was dead. **Site delta: 0 conversions** (run5 PC GAINING=7 / LOSING=1, same as 2a2). UNRESOLVED 173 vs 2a2's 209 (nested-program reports gone). |
| R10 | **PASS.** `form_calls_declared_types` skips `defn` bodies only. RED-once: temporary stdlib `defmacro :wat::fix::__r10-deadlock-probe` whose body calls `declared-types`; nextest `stdlib_snapshot_is_once_and_stdlib_does_not_call_the_verb` panicked `wat/fix.wat must not call declared-types (OnceLock re-entry / deadlock)` (thread 741508, `src/check.rs:23683`). Reverted. Floor after revert is green. |
| R11 | **PASS.** `[<tag>] UNREGISTERABLE <path> <cause> head=<head> name=<name> line=<n>[ fallback]`. Fixture: `MalformedDecl head=:wat::core::defenum name= line=11` (bare `(:wat::core::defenum)`, span matched — no fallback). Corpus: 0 fallback firings. |
| R12 | **PASS.** variant-separator SHAPE / IDEMPOTENCE / NO CASCADE rewritten to the door: per-program `enum-fields`, pairs from `fields`/`known-enum?`, substring prefilter, `rename-keyword-exact`. NEW is parent+"."+leaf, OLD is parent+"::"+leaf — order-independence without a census intersection. Nested `forms` collect their own pairs. |
| R13 | **PASS.** Door returns `:wat::fix::EnumFields {fields answered}`. `""` sentinel gone. Consumers use `enum-fields-get` / `known-enum?`. Smoke: `:usr::E::Variant` → `["a" "b"]`, `:wat::core::Option::Some` → `["value"]`, answered `[:usr::E :wat::core::Option]`. |

## run5 (`bootstrap/era/probe-S/run5.sh` on the implementation tree, SCORE-only amend after; tools identical)

| tool | wall (one process) | files LOSING | files GAINING | UNRESOLVED |
|---|---|---|---|---|
| match-arm vs `probe-L/wL` | 153 s | **0** (was 1) | 1 (`probe-m1-ann-erase2.wat` nested PoolMsg) | 10 (ladder 15) |
| positional-ctor vs `probe-L/pT` | 29 s | 1 (same artifact: `probe_arc278_macro_generates_service.wat` deeper `{:n c}`) | 7 | 173 (today 7663; was 209) |
| variant-separator vs the census tool | 58 s | 0 | 67 (was 62) | 38 UNREGISTERABLE (named) |

Chain vs main: identical **1370** (2a2: 1367), differs 119; classifier CHAIN-FAILS **28** (same).

`probe-m1-ann-erase.wat` nested match arms are KEY-FIRST. Remaining 4-line vs-main diffs are not losses: nested `CMsg::Setup` → `CMsg.Setup` (child door now flips the separator; main left `::`) and `probe-m1-ann-erase2.wat` child fields `{:addr addr}` / `{:s s}` vs main's parent-leaked `{:deps addr}` / `{:pair s}`. Door-correct; STOP-1 did not fire.

## Fixtures

| case | result |
|---|---|
| match-arm existing | byte-identical to `after.post` |
| nested used-outside | `[match-arm] UNRESOLVED :inner::E::V`; inner arm unconverted; wildcard delimiter-flipped |
| parent/child `:t::M` different fields | parent `[:t::M::A {:x x} x]`; child `[:t::M::A {:y y :z z} y]` |
| positional-ctor existing + different-fields | Shape still map-ctors; parent `(:t::M::A {:x 1})`; child `(:t::M::A {:y 1 :z 2})`; UNREGISTERABLE names the bare defenum |
| variant-separator | NodeKind `.Symbol`; Color.Red flipped; Color::NotAThing reported |
| E9 | `every_recorded_migration` 18/18 PASS |

## Blast radius

`wat/fix.wat` (EnumFields, named UNREGISTERABLE, forms-children) · `src/check.rs` (skip defn only) · match-arm / positional-ctor / variant-separator · their replay fixtures · scratch `probe-2a2-enum-fields.wat`. Not pushed.
