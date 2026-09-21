# SCORE-AMEND — STONE 255.4: the rename is HALF-APPLIED

Folded into `46cc976a5` (not a repair commit). Parent: `AMEND-255.4-the-rename-is-half-applied.md`.
The 28 floor reds are gone. Delta **held at 81**. Wat-side work was not undone.

## The third join — two consumers of the emitter, not a second `is_known_type`

`method_wat_path` already emitted `/`. The call sites already spelled `/`. Lookup still
failed because two **consumers** rebuilt or required `::`:

**1. `UseDeclarations::covers`** (`src/rust_deps/mod.rs`) — 25 `wat_dispatch_*` reds.

A `use!` of `:rust::test::MathUtils` covered a head only when the rest started with
`::`. Live call `:rust::test::MathUtils/add` was "not covered by any use!". This is
the consumer of the emitter: coverage is prefix + **member join**, and the join is
`/` (retired `::` still accepted so an old spelling reaches the retirement table
rather than dying as "not covered"). Not a second `is_known_type`.

The `.wat.bad` fixture still had `MathUtils::add` (git `*.wat` missed `*.wat.bad`).
Codemod applied. The type-check test now sees `/add` as the callee, matching the
assertion the first land already updated.

**2. `check.rs` surface-method arm** — `wat_scripts_fixes_load` /
`:my::Counter/surface-forms`.

`k.contains('/')` + receiver is a `TypeDef::Surface` + name is **not** a surface
member used to `UnknownCallee` immediately. Runtime already fell through to
`sym.get(other)`. Check did not. `<S>/surface-forms` is a generated 0-arg **fn**
under the surface type (S4c carrier), not a surface method. After 255.4 `/` is
the member join for both, so a non-member `/name` is an ordinary function.

The **emitter** of that carrier was still `format!("{}::surface-forms")` in
`types.rs` and `"{proto-base}::surface-forms"` in `wat/service.wat`. Both now `/`.
Same class as `method_wat_path`, not a second door.

## The other 3

| test | fix |
|---|---|
| `no_bootstrap_path_in_committed_rust` | Removed the `bootstrap/` string. Corpus is `git ls-files` of `tests/`, `wat/`, `wat-scripts/`, `wat-tests/` (`.wat` and `.wat.bad`). Ignored paths are already omitted. |
| `wat_scripts_fixes_load` / `:my::Counter/surface-forms` | Same root as (2) — user type, generated carrier. Not a second family. |
| `probe_arc255_ivb2a_examples_seam` | Expectation `wat::core::Bytes/to-hex`. No other seam/ledger row still names `Bytes::to-hex` as a live fqdn (remaining hits are comments). |

## Final derived set

Asked of the registries and the two consumers, not of grep:

| source | names |
|---|---|
| `wat_intrinsic` | 5: `Bytes/{to-hex,from-hex}`, `HandlePool/{new,pop,finish}` |
| `wat_dispatch` (`method_wat_path`) | `rust::cache::Lru/{new,put,get,len,is_empty}`, `rust::sqlite::{Connection,ReadConnection}/…`, test shims |
| wat-side `defn` | 8: `wat::cache::{Lru,HolographicLru}/{new,put,get,len}` |
| generated carrier | `<S>/surface-forms` (emitter in `types.rs` + `wat/service.wat` interpolate) |
| corpus exact (codemod) | the above + `rust::test::*` methods + `Reject/{bundle-or-fail,project-bundle-or-fail}` + `.wat.bad` |

Do **not** rename `Record::def` (retired to `defrecord`). Nested type names
(`Cache::GetRequest`) and `::Op`/`::Reply` stay `::`.

## Delta (same tree)

| | 255.4 first land | this fold |
|---|---|---|
| originals clean | 161 / 179 | **161 / 179** |
| still clean | 80 | **80** |
| REGRESSIONS | **81** | **81** |

Held. The third-join fix does not close more converted files — those 25 reds
were rust-test shims, not the 179 census.

## Walls

The 28 named tests pass (25 `wat_dispatch_*` + 3). Crate clippy `-p wat` 0.
`one_member_join` / `one_variant_separator` / `no_bootstrap_path` pass.
Floor / workspace clippy / census: orchestrator. Do not push.
