# SCORE 2a4c — the stdlib door replaces a divergent MACRO; it walks as the user door walks

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent: `bb60894f7` BRIEF 2a4c. Finding 19. 2a4c: `0bdac1ccd`.

```
floor  scripts/floor.sh   .floor/2026-09-14T23-03-18Z
       Summary [ 223.851s] 5498 tests run: 5498 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

5495 + 3 new tests (chained `mk`, unexpandable `defn` body, every tracked `wat/` file) = 5498.

## The door

`register_declared_stdlib_types` (`src/freeze/env.rs`):

1. **Macro retract.** A top-level `defmacro` the file defines that the cloned registry holds
   DIVERGENTLY (`!macro_structurally_equivalent`) is retracted from the copy
   (`MacroRegistry::retract_if_divergent`) before `register_stdlib_defmacros`. Equivalent is a
   no-op. The copy only.
2. **The walk.** `collect_type_forms` (the user half's one-step walk) keeps only what can declare
   a type; a `defn` body is never expanded. Kept forms expand under `Privilege::Stdlib` so a
   companion minted mid-walk still registers.
3. Then `register_stdlib_types_replacing` → `register_variant_types`. Returns `(TypeEnv, MacroRegistry, names)`
   so a chained call 2 can use call 1's output as its snapshot.

STOP-9: `convert.sh` reporting `UNREGISTERABLE wat/…` exits 9. BRIEF-1 § One step 3 and the STOP list.

## Fixtures

| case | result |
|---|---|
| chained `mk` | `declared_stdlib_types_replaces_divergent_macro`: call 1 mints `E.V [a]`; call 2's snapshot is call 1's output; `mk` now mints `[b c]`; env2 answers the NEW fields; env1 still `[a]`. |
| mutation | RED captured: `call 2: #wat.macro/DuplicateMacro … name ":wat::probe2a4c::mk"`. Restored, GREEN. |
| unexpandable `defn` body | `declared_stdlib_types_skips_unexpandable_defn_body`: file with `Colour` plus a `defn` whose body is `(:wat::kernel::start-primed-stdio)` returns `Colour`, not a refusal. |
| every current `wat/` file | `declared_stdlib_types_accepts_every_tracked_stdlib_file` GREEN — STOP-1 does not fire. |

## Era `wat/core.wat` through the door

DuplicateMacro is gone. The door's first answer is not Ok:

`MalformedDefmacro` on `:wat::core::format` (line 1639): program-body purity check,
`keyword head :wat::core::ReadOutcome::Forms refused at macro expand time` (F5 default-deny).
The same class the brief names as expected on MA for era `format` / `defservice`. Named, not STOP-1
(STOP-1 is a **current** `wat/` file; HEAD `wat/core.wat` is Ok).

Era `wat/service.wat` through the door: `MalformedDefmacro` on `:wat::service::defservice` (line 180),
same `ReadOutcome::Forms` cause — the expected leftover.

## run5 (`bootstrap/era/probe-S/run5.sh` on `0bdac1ccd`, repo-relative paths)

| tool | wall | files LOSING | files GAINING | notes |
|---|---|---|---|---|
| match-arm vs `probe-L/wL` | 148 s | **0** | 1 | UNRESOLVED 10 |
| positional-ctor vs `probe-L/pT` | 28 s | **2** (2a2 artifact + `wat/service.wat`, skip-path) | 7 | UNRESOLVED 84 (was 173) |
| variant-separator vs the census tool | 56 s | **0** | 67 | report lines 22 (baseline 34) |

Chain vs main: identical **1370**, differs 119; CHAIN-FAILS **28**. START 23:07:55Z DONE 23:17:29Z.

`wat/` refusals:

| before (33f3ebcfb) | after 2a4c |
|---|---|
| DuplicateMacro **11** | **0** |
| ProgramBodyEvalFailed **2** | **0** on HEAD `wat/` through the door. MA/VS still print one `ProgramBodyEvalFailed wat/service.wat line=2443` — match-arm's drop-form retry after dropping the malformed era `defservice`; the door's first answer on that file is the expected `MalformedDefmacro defservice`. The nested `defn ~serve-name` lives inside that template (era line 2443), not a top-level `defn` the walk skips. |

MA's remaining `wat/` lines, both expected:

```
[match-arm] UNREGISTERABLE wat/core.wat MalformedDefmacro head=:wat::core::defmacro name=:wat::core::format line=1639
[match-arm] UNREGISTERABLE wat/service.wat MalformedDefmacro head=:wat::core::defmacro name=:wat::service::defservice line=180
```

## Census

`.census/2026-09-14T23-02-35Z.txt` (2050 files). `--diff` against the 2a4b baseline
`.census/2026-09-14T22-02-27Z.txt`: **no STOP-8**, exit 0.

## The bar

| item | result |
|---|---|
| Both fixtures GREEN; first RED under mutation (verbatim) | **PASS.** |
| Era `wat/core.wat`: no DuplicateMacro | **PASS.** First answer is the expected `MalformedDefmacro format`, not Ok — named above. |
| run5: 0 losing MA/VS; PC's 2 unchanged; chain ≥ 1370 | **PASS.** |
| DuplicateMacro 11 → 0; ProgramBodyEvalFailed 2 → 0 | **PASS** on the door. MA drop-retry leftover named. |
| census `--diff` vs latest: no STOP-8 | **PASS.** |
| Floor + clippy | **PASS.** 5498/5498, clippy 0. |

Not pushed. Batch 2 is a separate brief.
