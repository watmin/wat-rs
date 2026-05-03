# Arc 143 Slice 1 — Pre-handoff expectations

**Drafted 2026-05-02 (evening)** for the substrate-side
introspection-bridge slice.

**Brief:** `BRIEF-SLICE-1.md`
**Output:** TWO Rust files modified (`src/runtime.rs` +
`src/check.rs`) + 9-12 new tests + ~250-word written report.

## Setup — workspace state pre-spawn

- Substrate has all the prerequisites: `Function` struct
  (runtime.rs:499) preserves name/params/types/body;
  `Environment::lookup` (runtime.rs:563) returns
  `Option<Value>`; TypeScheme registry at `env.schemes`
  (check.rs:1149-1150) holds substrate-primitive metadata.
- `eval_struct_to_form` (runtime.rs:5891+) is the worked
  precedent for "convert runtime data to WatAST."
- No prior `lookup-define` / `signature-of` / `body-of`
  primitives exist (verified via filesystem grep).
- DESIGN.md's Findings section resolves Q1 + Q2 with file:line
  pointers.
- Workspace test green: `cargo test --release --workspace`
  exit=0, 1820 passed, 1 ignored. 4 currently-failing LRU
  tests from arc 130 prior sweep STILL exist on disk (the
  `:wat::core::reduce` gap is the proximate cause and what
  this arc enables fixing in arc 130 slice 4).

## Hard scorecard (12 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 1 | Two-file Rust diff | Exactly 2 Rust files modified: `src/runtime.rs` + `src/check.rs`. No other Rust files. No wat files. No tests/* files except the new test file (counted separately as +1 file). No documentation. |
| 2 | New test file added | ONE new test file added — either `tests/wat_lookup.rs` (Rust integration) or `wat-tests/lookup.wat` (wat). Pick whichever matches existing patterns most cleanly. |
| 3 | `eval_lookup_define` present | Function present in runtime.rs. Validates 1 arg. Returns `Value::holon__HolonAST(Arc<HolonAST>)` wrapped via the project's `Some/None` Value construction (likely `Value::wat__core__lambda` enum dispatch — match existing `Option<T>` returns). |
| 4 | `eval_signature_of` present | Same shape; returns just the head AST. |
| 5 | `eval_body_of` present | Same shape; returns body for user defines, `:None` for substrate primitives. |
| 6 | 4 helper Rust functions present | `function_to_define_ast`, `function_to_signature_ast`, `type_scheme_to_signature_ast`, `primitive_to_define_ast`. Each takes the relevant input + returns `WatAST`. Sentinel body for primitives is `(:wat::core::__internal/primitive <name>)`. |
| 7 | 3 dispatch arms in runtime.rs | New match arms in the dispatch around line 2406-2410: `:wat::core::lookup-define`, `:wat::core::signature-of`, `:wat::core::body-of`. |
| 8 | 3 scheme registrations in check.rs | New `env.register(...)` calls for the three primitives. Each takes `wat::core::Symbol -> Option<wat::holon::HolonAST>`. |
| 9 | **Lookup precedence** | Each primitive's lookup checks env FIRST, then schemes, then returns `:None`. User defines shadow primitive registrations (matches normal call dispatch). |
| 10 | **`cargo test --release --workspace`** | Exit=0; **1820 + N passed** (where N is the count of new tests, 9-12); 0 new failures; 1 ignored (arc-122 mechanism); the 4 LRU `reply channel closed` failures STAY (arc 130 slice 4 will fix those AFTER arc 143 ships — slice 1 of arc 143 doesn't address them yet). The PRE-EXISTING failure count is unchanged; the NEW test additions all pass. |
| 11 | Test coverage of all 9 cases | For each of the three primitives: user-define lookup (3 tests), substrate-primitive lookup (3 tests), unknown-name returns `:None` (3 tests). For `body-of` specifically: substrate-primitive returns `:None` (1 additional test). Total: 9-10 minimum, 12 max if you add edge cases (e.g., a name that's both a user define AND a primitive — the user define wins). |
| 12 | Honest report | 250-word report includes: file:line refs for the 3 eval funcs + 4 helpers + dispatch arms + scheme registrations; the verbatim synthesized AST shape for `signature-of :wat::core::foldl`; test totals; honest deltas; LOC delta. |

**Hard verdict:** all 12 must pass. Row 9 is the load-bearing
precedence rule. Row 10 is load-bearing for runtime correctness
(no regressions). Row 11 is load-bearing for coverage breadth.

## Soft scorecard (5 rows)

| # | Criterion | Pass condition |
|---|---|---|
| 13 | LOC budget | runtime.rs additions: ~80-150 LOC (3 eval funcs + 4 helpers + dispatch). check.rs additions: ~30-60 LOC (3 scheme registrations + any imports). Test file: ~80-150 LOC (9-12 tests). Total slice diff: 200-400 LOC. >500 LOC = re-evaluate (likely over-engineered). |
| 14 | Style consistency | The new eval funcs follow the existing pattern in runtime.rs (arg count check → arg type check → execute → return Value with proper Span). The helpers are in a section adjacent to other Value→AST helpers (near `eval_struct_to_form` at line 5891+). |
| 15 | Sentinel placement | The `(:wat::core::__internal/primitive <name>)` sentinel is documented inline (rustdoc on `primitive_to_define_ast`) noting it's NEVER evaluated — substrate primitives use Rust dispatch, not the sentinel body. |
| 16 | Span discipline | Either consistent `Span::unknown()` for synthesized AST, OR the user-define case copies the original body's span. Pick one and stick with it across the 4 helpers. |
| 17 | Dispatch ordering | The 3 new arms in the runtime dispatch sit near other introspection primitives (next to `:wat::core::struct->form` at line 2408). Don't scatter them across the dispatch table. |

## Independent prediction

Before reading the agent's output, the orchestrator predicts:

- **Most likely (~55%):** all 12 hard + 4-5 soft pass cleanly.
  The substrate has the worked precedent (`struct->form`); the
  Function struct preserves all needed data; the TypeScheme
  registry is queryable. Sonnet ships in 15-25 min. Tests for
  user-define lookup work straight off; substrate-primitive
  tests need careful AST construction but the helpers reduce
  it to a recipe.

- **Second-most-likely (~20%):** 11-12 hard pass + soft drift
  on the sentinel form's exact representation OR on how
  `Option<HolonAST>` is constructed in the dispatch return
  path. Outcome still committable; minor rework or reland.

- **TypeScheme→AST converter complexity (~12%):** synthesizing
  `:_a0`, `:_a1` etc. names + rendering `TypeExpr` back to
  keyword form is more involved than the brief implies.
  Sonnet may need to discover the existing TypeExpr→keyword
  rendering path (likely in a `Display` impl). If the
  rendering path doesn't exist, sonnet writes one. Surfaces
  in honest deltas; row 4 + 5 still pass.

- **Test placement gap (~8%):** sonnet picks the wrong test
  location (e.g., adds tests to a Rust integration file
  when wat-tests/ would be cleaner, or vice versa). Cosmetic;
  reland not needed.

- **Lookup precedence error (~5%):** sonnet checks schemes
  before env, OR doesn't handle the both-present case
  correctly. Row 9 fails; reland with sharper precedence
  example.

## Methodology

After the agent reports back, the orchestrator MUST:

1. Read this file FIRST.
2. Score each row of both scorecards explicitly.
3. Diff via `git diff --stat` (expect 2 Rust files modified
   + 1 test file added).
4. Verify hard rows 3-8 by `grep -n` for the new function
   names + dispatch arms + scheme registrations.
5. Verify hard row 10 by running `cargo test --release
   --workspace` locally; confirm sonnet's reported totals.
6. Verify hard row 11 by reading the test file's deftest /
   `#[test]` count + their names.
7. Verify hard row 12 by reading the report.
8. Verify row 9 (precedence) by reading the eval func
   bodies — does each check env before schemes?
9. Score; commit `SCORE-SLICE-1.md` as a sibling.

## Why this slice matters for the chain

Slice 1 is the ONLY Rust work in arc 143. Once it ships,
slices 2-5 are pure wat. The substrate gains an
introspection bridge that's been missing since macros
shipped — every future userland macro that needs to ASK
about an existing binding gets these three primitives for
free.

This slice ALSO continues the failure-engineering chain:
- Arc 130 slice 1 RELAND surfaced the reduce gap (Mode B
  at Layer 1 — clean diagnostic)
- Arc 143 ships the introspection primitives that enable
  the userland fix
- Arc 143 slices 2-5 ship the userland fix itself
- Arc 130 slice 1 RELAND v2 picks up at Layer 2+ against
  the corrected substrate

This is the substrate-as-teacher pattern playing out in
real time. The reshape removed a structural shield → runtime
gap surfaced → diagnostic was clean → fix is targeted → the
substrate gets stronger.

## What we learn

- **All hard pass:** the introspection bridge is sound; arc
  143 slices 2-5 are unblocked; pure wat from here. Slice
  2 brief drafts off this score's calibration.
- **Row 10 fails (workspace not green):** the new primitives
  broke something. Diagnose; reland with the breakage as the
  failing-row diagnostic.
- **Row 9 fails (lookup precedence wrong):** the substrate-
  primitive shadows the user define instead of the other
  way. Reland brief quotes the precedence rule verbatim.
- **Row 11 fails (test coverage gap):** sonnet missed a
  variant. Reland adds the missing tests; substrate code
  is fine.
- **Soft drift (row 13 LOC over budget):** likely sonnet
  invented helpers we didn't anticipate. Score notes them;
  may reveal a pattern worth keeping.
