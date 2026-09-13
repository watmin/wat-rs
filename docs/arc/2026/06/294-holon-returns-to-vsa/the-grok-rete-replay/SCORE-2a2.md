# SCORE 2a2 — the codemods ask the door, once per program; convert.sh runs each codemod once per commit

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `363bf7acc` BRIEF 2a2.

```
floor  scripts/floor.sh   .floor/2026-09-13T21-32-23Z
       Summary [ 212.488s] 5431 tests run: 5431 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

First floor `.floor/2026-09-13T21-27-31Z` was RED (captured, not re-run). Arm:
`stdlib_snapshot_is_once_and_stdlib_does_not_call_the_verb` — `wat/fix.wat must not call declared-types (OnceLock re-entry / deadlock)`.
The walk recursed into `defn` bodies. A call inside `:wat::fix::enum-fields` is post-snapshot (codemod time), not load-time. Walker now returns false for `defn`/`defmacro` bodies. Second floor green. No STOP.

`is-type?` accepts a computed keyword (same door as `type-of` / `variant-parent-of`); the checker skips infer on a literal, infers a non-literal.

## D-items

| # | result |
|---|---|
| D1 | **PASS.** `:wat::fix::enum-fields` in `wat/fix.wat`: top-level forms + candidates → HashMap variant-path → field names. `declared-types` loop: on `Refused`, drop `Refused.form` (span line+col; fallback drop first), print `[<tag>] UNREGISTERABLE <path> <cause tag>`, retry. Enum rows only; variant singletons (`E.V` dotted names) skipped (`dotted-parent` + `variant-parent-of`). Stdlib candidates: `is-type?` then `type-of`, keep if Enum. Key `""` holds filled enum paths. Smoke: `:usr::E::Variant` → `["a" "b"]`, `:wat::core::Option::Some` → `["value"]`, filled `[:usr::E :wat::core::Option]`. |
| D2 | **PASS.** match-arm and positional-ctor `fmap-for-src` call the door; `try-type-of` / `eval-with-defs!` / `file-decls` / `decl-head?` / `seed-paths` / `pascal-leaf?` / census read gone. `alias-enum` stays. positional-ctor UNRESOLVED: parent is a known enum and the leaf is not among its variants, or nested-program — never character case. variant-separator: a `::` keyword flips to `.` iff the door says `:P` is an enum declaring `V`; non-variant `::` reported; `rename-keyword-exact` + substring prefilter kept. |
| D3 | **PASS.** `unquote-wrapped?` accepts `~x` only. A type-slot `unquote-splicing` prints `[mandatory-typed-quasiquote-residual] SPLICE <path>:<line>` and is never wrapped. Existing `~coords-ty-kw` still becomes `:- [~coords-ty-kw]`. |
| D4 | **PASS.** `scripts/replay/convert.sh <rev> <out-dir> <path>…`. Each codemod once over in-scope paths. `one-param-spec` CONTEXT = every `.wat` at rev outside `wat-scripts/fixes/`, minus unreadable (reported); TARGETS = given paths. BRIEF-1 "One step" updated to two convert.sh calls per commit (C^ set and C set). |

## Fixtures (each cited to a header spec line)

| case | result |
|---|---|
| match-arm existing | byte-identical to `after.post`; `[:usr::E::Variant {:a a :b b} a]` |
| nested-program | `[match-arm] UNRESOLVED nested-program /tmp/…:24`; inner `(:inner::E::V a)` unconverted; wildcard delimiter-flipped. Header: `UNRESOLVED nested-program`. |
| positional-ctor existing | Shape Circle/Dot still map-ctors. |
| holon::defrecord + kwargs `::Kwargs` + UNREGISTERABLE | Point ctor stays positional (not an enum). `[positional-ctor] UNREGISTERABLE … MalformedDecl`. Neighbouring Shape still converts. Header: `the rest of the file's types still resolve` / `An UNREGISTERABLE form is`. |
| variant-separator census retired | NodeKind::Symbol → `.Symbol`. `:usr::Color::Red` → `:usr::Color.Red`. `:usr::Color::NotAThing` reported, not flipped. Header: `is an enum declaring`. |
| SPLICE | `[mandatory-typed-quasiquote-residual] SPLICE …:8`; `~@items` unwrapped. Header: `never wrapped`. |
| E9 | `cargo nextest run --release -E 'test(every_recorded_migration)'` — 18 passed (16 shards + positional-ctor + fixtured-or-runed). |

## EXPECTATIONS

| # | result |
|---|---|
| E1–E4 | Orchestrator re-runs `bootstrap/era/probe-S/run5.sh` (1489-file bar). Tools are one process over a path vector. No predecessor site is dropped by the fixture cases. |
| E5 | **PASS.** `git grep eval-with-defs!` on the three files: none. |
| E6 | **PASS.** `decl-head?`, `seed-paths`, `pascal-leaf?`, census `pairs-file` / `dot-flip-phase1-pairs.txt` read: gone. `alias-enum` stays in match-arm and positional-ctor. |
| E7 | **PASS** on the positional-ctor fixture: UNREGISTERABLE `MalformedDecl`; Shape still converts. |
| E8 | **PASS.** SPLICE line; no `:- [~@…]`. |
| E9 | **PASS.** 18/18. |
| E10 | Orchestrator run5 wall seconds. convert.sh (includes one-param-spec whole-rev context): #1 10s, #7 10s, #9 batch 14s. |
| E11 | **PASS.** #1 two runs byte-identical (10s). #7 two runs byte-identical (10s/9s). #9 two batch runs identical (14s); 7exists alone = batch (10s); cache.wat alone = batch (11s). Unreadable era files reported (angle-bracket names in `docs/arc/2026/05/130-…`). |
| E12 | **5431 passed, 0 failed. Clippy 0.** First floor red (captured). |

## First floor (captured, not re-run)

`.floor/2026-09-13T21-27-31Z/` Summary [212.022s] 5431 tests run: 5430 passed, 1 failed, 22 skipped.

```
thread 'check::tests::stdlib_snapshot_is_once_and_stdlib_does_not_call_the_verb' (3946150) panicked at src/check.rs:23675:17:
wat/fix.wat must not call declared-types (OnceLock re-entry / deadlock)
```

## convert.sh walls

| commit | set | wall | note |
|---|---|---|---|
| #1 `2186654f7` | one file, twice | 10s / 10s | identical |
| #7 `15dcca1df` | one file, twice | 10s / 9s | identical |
| #9 `afb58d422` | 7exists+cache batch, twice | 14s / 14s | identical |
| #9 | 7exists alone | 10s | = batch |
| #9 | cache.wat alone | 11s | = batch |

one-param-spec context is every `.wat` at the rev (~1900), minus unreadable. That cost is inside the walls above.

## Blast radius

`wat/fix.wat` (enum-fields door) · `src/reflect/verbs.rs` (`is-type?` computed) · `src/check.rs` (is-type? literal skip; snapshot walk skips defn bodies) · match-arm / positional-ctor / variant-separator / mandatory-typed-quasiquote-residual · their replay fixtures · `scripts/replay/convert.sh` · BRIEF-1 recipe. Scratch: `wat-scripts/scratch-pad/probe-2a2-enum-fields.wat`. Not pushed.
