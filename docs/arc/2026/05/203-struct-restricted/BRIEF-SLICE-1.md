# BRIEF — Arc 203 Slice 1: substrate primitive minting (struct-restricted)

**Phase:** First implementation slice of arc 203. Mints the wat-side `:wat::core::struct-restricted` substrate primitive.

**Predecessor:** Arc 198 (def-restricted / defn-restricted). Same mechanism family — adding one declaration surface; reusing arc 198's walker and storage unchanged.

**Successor:** Slice 2 — Counter/Client capability struct using struct-restricted as the ServiceWithProvisioning demo's first consumer.

## Goal

Ship the substrate primitive that lets wat-side struct declarations carry per-constructor + per-accessor whitelists. Walker enforcement is arc 198's existing machinery — no walker changes.

## Required form

```scheme
(:wat::core::struct-restricted :Name
  [<constructor-whitelist-prefixes>...]            ;; ctor whitelist for Name/new
  ([<wlist>] field <- :T, ...)                     ;; restricted attrs section (variadic)
  (field <- :T, ...))                              ;; public attrs section (variadic)
```

Four positional slots after head. All explicit; no inheritance; sections present even when empty.

### Required tests (in `tests/wat_arc203_struct_restricted.rs`)

1. **Form parses + structurally valid** — a struct-restricted declaration compiles cleanly; struct accessors synthesized; constructor + accessors callable from whitelisted prefix
2. **Constructor restriction fires** — caller outside ctor whitelist calling `Name/new` → `DefRestrictedCallerNotAllowed` (the arc 198 error variant; we reuse it; do not mint a new error)
3. **Per-field restriction fires** — caller outside a field's whitelist calling `Name/<restricted-field>` → `DefRestrictedCallerNotAllowed`; caller outside the ctor whitelist but inside a field's whitelist can still read that field
4. **Public accessors unrestricted** — any caller can read `Name/<public-field>` regardless of namespace
5. **Empty sections honored** — `()` for restricted section (all fields public, ctor still restricted); `()` for public section (everything restricted)
6. **Malformed shapes rejected** — wrong arity, non-keyword whitelist entries, wrong section structure → MalformedForm with clear reasons

Tests follow the shape of `tests/wat_arc198_def_restricted.rs` (5 tests) as a template.

## Required code path

### Type-check side (src/check.rs)

- Recognize `:wat::core::struct-restricted` as a keyword head (parallel to `:wat::core::struct` at src/check.rs:5260)
- Mint `infer_struct_restricted` mirroring `infer_def_restricted` (src/check.rs:7478) for shape validation:
  - 4 positional args after head: name keyword + ctor whitelist Vector + restricted-section List + public-section List
  - Validate ctor whitelist entries are keywords (per arc 198's `extract_def_restricted_binding` extraction pattern)
  - Validate restricted-section entries: each entry is a `[wlist] field <- :T` triple-shape; wlist is Vector of keywords
  - Validate public-section entries: each entry is a `field <- :T` pair-shape
  - Returns `Some(TypeExpr::Tuple([]))` — declaration form, no value type
- Register the type declaration into `CheckEnv.types` (same path as `:wat::core::struct`)
- Register restrictions into `CheckEnv.defined_value_restrictions` (per arc 198 pattern) for:
  - `Name/new` → ctor whitelist
  - `Name/<field>` for each field in restricted section → its own whitelist
  - (Public-section fields get NO entry — they're unrestricted)

### Runtime side (src/runtime.rs)

- Detect `:wat::core::struct-restricted` keyword head in `register_runtime_defs_form` (src/runtime.rs:2224+); parallel arm to existing struct detection at src/runtime.rs:2410
- Register the type into `TypeEnv` per existing struct path
- Extend `register_struct_methods` (src/runtime.rs:1879) OR fork a `register_struct_methods_with_restrictions` companion that:
  - Synthesizes `Type/new` Function + `Type/<field>` Functions (per existing register_struct_methods)
  - Populates `SymbolTable.defined_value_restrictions` with the parsed whitelists (mirroring the CheckEnv side)

### What you do NOT need to do

- **Walker** — `walk_for_def_restricted_call` (src/check.rs:3152+) is the existing arc 198 walker; it reads from `defined_value_restrictions` regardless of which declaration surface populated the HashMap. It will fire automatically once restrictions are registered. DO NOT modify the walker.
- **CheckError variant** — reuse `DefRestrictedCallerNotAllowed`. DO NOT mint a new variant.
- **Inventory wiring** — arc 198 Stone 1 (slice 2) wired Rust-side restrictions via `inventory::submit!`. Arc 203 is wat-side only; no inventory work needed. The form is parsed at freeze time and registers directly.

## STOP triggers (true emergencies — surface, do not paper over)

1. **`register_struct_methods` signature change needed** beyond adding a restrictions parameter — surface; we may need to fork rather than extend
2. **Walker doesn't catch the new accessor sites** — would mean arc 198 walker is binding-name-specific rather than HashMap-generic; surface immediately (would invalidate arc 203's core assumption)
3. **TypeEnv registration path requires a new variant on `TypeDef`** — surface; we may need to treat struct-restricted as a wrapper around struct rather than a parallel form
4. **Form shape parsing requires changes to the parser** beyond `infer_struct_restricted` — surface; the parser already handles keyword-headed lists and Vectors generically per arc 198, but if there's a new structural pattern (e.g., per-field whitelist nesting), it might need parser-level work
5. **Workspace baseline regresses** beyond the pre-existing 3 failures (deftest_wat_tests_tmp_totally_bogus, startup_error_bubbles_up_as_exit_3, t6_spawn_process_factory_with_capture_round_trips) — STOP

## HARD constraints

- **DO NOT commit.** Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Operate on real repo, not `.claude/worktrees/`.
- DO NOT mint new walker code — arc 198's walker is reused.
- DO NOT mint new CheckError variants — reuse `DefRestrictedCallerNotAllowed`.
- DO NOT use `--no-verify` / `--no-gpg-sign`.
- DO NOT touch arc 198's files (`src/check.rs:3152+` walker, etc.) — extend, don't modify.

## Decay disclosure (orchestrator)

The substrate touchpoints (line numbers, fn names) are accurate as of 2026-05-16 — verified during DESIGN drafting. Sonnet operates from current disk state; if any signature or location has shifted since the timestamp, sonnet's grep-first investigation should surface and adapt without re-asking.

The exact mechanism for `register_struct_methods` extension is sonnet's discovery — orchestrator suggests "extend or fork"; sonnet picks based on the actual function shape. Either is honest.

## SCORE methodology

6 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Form parses cleanly for a worked example | One positive test compiles + runs |
| B | Constructor restriction fires on illegal caller | Negative test produces `DefRestrictedCallerNotAllowed` |
| C | Per-field restriction fires on illegal accessor caller | Negative test produces `DefRestrictedCallerNotAllowed` per restricted field |
| D | Public accessors unrestricted | Positive test: caller outside any whitelist successfully reads public fields |
| E | Empty sections work (both `()` for restricted, `()` for public) | Two positive tests covering each degenerate shape |
| F | Workspace failure count ≤ baseline (3 pre-existing) | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures |

## Time-box

Predicted: 60-90 min sonnet. Hard stop: 120 min.

## Workspace baseline

Pre-arc-203 baseline (verified 2026-05-16 just prior to spawn): clean except 3 pre-existing failures:
- `deftest_wat_tests_tmp_totally_bogus`
- `startup_error_bubbles_up_as_exit_3`
- `t6_spawn_process_factory_with_capture_round_trips`

Post-slice-1 target: pass count ≥ baseline + 6 (new arc 203 tests); fail count ≤ 3 (no regressions).

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-1.md` per § SCORE methodology.
2. Return final summary to orchestrator:
   - Rows passed/failed
   - Workspace delta
   - File paths touched
   - Any honest deltas surfaced (substrate shape that differed from BRIEF assumption)
   - Suggested INTERSTITIAL or DESIGN corrections (if any)

You are launching now. T-minus 0.
