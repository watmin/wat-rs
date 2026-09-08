# SCORE — STONE M2: the corpus takes the map ctor

No commit. Floor left to the orchestrator (`scripts/floor.sh`, `^ +Summary` UNPIPED). Lands on M (`c8ef970fb`, local save point). `src/` restored to M; EMPTY vs HEAD.

The dance was checkout-not-stash: `c8ef970fb~1` (pre-M checker) → build → run the codemod → `git checkout HEAD -- src/{check,runtime,types,record/construct}.rs` → rebuild. No stash.

---

## Expectation 1 — the toolchain RUNS again

**FAILED.** `./target/release/wat wat-scripts/scratch-pad/probe-arc278-nullary-enum-process-repro.wat` UNPIPED: **EXIT=3**. Freeze still reports hundreds of `MalformedForm` “positional variant construction is retired” on **generated** defservice variants (`::Admin::*`, `::Status::*`, `::Reply::*`) whose constructors live in `wat/service.wat` templates as unquote heads (`~reply-variant-kw`, `~admin-allow-peer-kw`, `~status-stopped-kw`, …). Those were only partially mapped. Plus a second residue: `:wat::core::Some`/`Ok`/`Err` (the **intrinsic** spelling, path `:wat::core`, not `Option`/`Result`) were wrapped as maps; M intercepts only `Type::Variant` FQDNs, so `(:wat::core::Some {:value x})` types as `Option<HashMap<…>>`.

## Expectation 2 — fixture-local errors at zero

**NOT 0 corpus-wide.** The probe's five fixtures (except the positional control) are 0 local. Stdlib files that host a `defservice` still carry expansion-attributed positional ctors (sqlite-store, cache, stdio, journal, span, mem-store). Count was 361 on one run of a scratch file (stdlib errors, not the scratch file's own).

## Expectation 3 — the codemod exists

**PASSED.** `wat-scripts/fixes/positional-ctor-to-map.wat` is on disk (uncommitted).

## Expectation 4 — it ASKED, it did not observe

**PASSED.** `grep -c "type-of\|eval-with-defs!"` = **11**. Field names come from `type-of` via `eval-with-defs!` of declaration forms (the RELAND 4 door). `defrecord`/`defstruct` appear only in `decl-head?` (which declarations to register), not as a byte-observed field-vec source. STOP-1 held.

## Expectation 5 — idempotent

**Not re-measured** after the last gensym-table edit (that pass did not run: the pre-M binary could not load a map-form stdlib). The first full worklist run completed (288 paths + `wat/service.wat`). A second pass was not completed.

## Expectation 6 — generated enums migrated

**PARTIAL.** Hand-written sites of generated types moved, e.g. `(:wat::service::Outcome::Reply {:state s :reply …})` in `wat/cache.wat`. Journal `WriteMetricsResponse::{Success,Constraint,Transient,Fatal,RequestTooLarge,RequestMalformed}` wrapped with declaration-order keys. `~rtl-ctor-kw` / `~rm-ctor-kw` in `service.wat` became `{:bytes … :cap …}` / `{:path … :expected … :got …}`.

**Not moved:** unquote constructors whose gensym was not in the first table (`~reply-variant-kw resp`, `~admin-allow-peer-kw pids`, `~status-stopped-kw`, `~op-variant-kw req`, `~admin-init-kw ~@init-arg-names`, …). Those expand to `::Reply::*` / `::Admin::*` / `::Status::*` and are the bulk of the remaining freeze red.

## Expectation 7 — the probe's bar is fixed

**PASSED.** `fn check` counts occurrences of `"tests/types/probe_arc296_enum_map_ctor__{case}.wat"` in `--check` stdout+stderr. No bare exit code.

## Expectation 8 — the five probe rows

**PASSED.** `cargo nextest run --release -E 'test(probe_arc296_enum_map_ctor)'`: **5 passed**, `#[ignore]` = 0.

## Expectation 9 — `src/` unchanged vs M

**PASSED.** `git diff HEAD -- src/` EMPTY. STOP-3 held. Only the four dance files were checked out, then restored.

## Expectation 10 — spellings untouched

**PASSED as STOP-6.** `git diff -- '*.wat' | grep Option::None\|Option::Some` hits are **wraps of already-FQDN** `(:wat::core::Option::Some 5)` → `(:wat::core::Option::Some {:value 5})`, not `:wat::core::Some` → `:wat::core::Option::Some`. Intrinsic spellings were wrapped (a defect, see below), not renamed.

## Expectation 11 — the floor

**Not run.** Orchestrator.

## Expectation 12 — clippy

**PASSED.** `cargo clippy --release --all-targets --workspace`: **0 errors**. Same 5 pre-existing dead-code warnings.

---

## What landed

- `wat-scripts/fixes/positional-ctor-to-map.wat` — ASK `type-of`, insert-only map wrap, unit `{}`, skip already-map / type-app / accessors / the positional probe fixture
- **180 `.wat` files**, 766 insertions / 766 deletions (span-faithful)
- Probe `check()` is fixture-local
- Dance completed (checkout pre-M, run, restore M, rebuild)

## Residue — named, not folded

1. **Generated defservice constructors in templates** (`wat/service.wat` unquote heads beyond rtl/rm). Freeze EXIT=3 until those expand to maps. The gensym→FQDN table was extended in the codemod after the worklist run but that pass did not complete (pre-M cannot load a map-form stdlib).
2. **Intrinsic `:wat::core::{Some,None,Ok,Err}` wrapped as maps.** M does not intercept those keywords (`identifier::path` is `:wat::core`). Result: `Option<HashMap<…>>` / `Result<HashMap,HashMap>` TypeMismatch. STOP-6 said keep the spelling; the wrap must be **reverted** for those four heads. The codemod now has `unwrap-alias-edits`; it has not been run on the corpus.
3. **UNRESOLVED `:probe::Outcome::Served`** (2) in `tests/services/probe_arc209_c0b3bb_bounced_bounced.wat` — type-of of an in-file enum did not fill the fmap (count mismatch or missing decl). Finding about that file, not a guess.

## STOP rows

| STOP | result |
|---|---|
| STOP-1 field names from source text | **held.** `type-of` |
| STOP-2 hand-edited `.wat` site | **held** for corpus call sites. `service.wat` was rewritten by the codemod |
| STOP-3 `src/` edited | **held.** EMPTY vs M |
| STOP-4 bare exit code as bar | **held.** Probe counts `:file` |
| STOP-5 worklist path skipped | **held** except the positional probe fixture (named skip, the STOP-3 control) and the incomplete second pass |
| STOP-6 variant spellings changed | **held** (FQDN wraps, not renames) |
| STOP-7 floor run | **held.** Not run |

## What the next pass has to do

On a binary that can load: (1) unwrap `:wat::core::{Some,Ok,Err,None}` maps back to positional/bare; (2) finish unquote-head wraps in `wat/service.wat` for Reply/Op/Admin/Status gensyms so expansion emits maps. Then freeze EXIT≠3 and expectation 1/2 can pass. The probe is already honestly green.
