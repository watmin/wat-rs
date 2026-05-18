# EXPECTATIONS — Arc 209 Stone A

## Mode prediction

**Mode A — clean atomic ship (~65%).** Sonnet:
1. Reads BRIEF + DESIGN + production precedent (`wat/runtime.wat:17-32`)
2. Drafts `wat/kernel/spawn_program.wat` using keyword/of + computed-unquote (precedent pattern)
3. Reshapes walker — moves the `:wat::kernel::spawn-program` detection from `WatAST::Keyword` arm into `WatAST::List` arm with args-count dispatch
4. Updates stdlib loader (one new entry)
5. Writes test file with positive/negative/edge cases
6. Verifies cargo test green

~90-130 min wall-clock. SCORE 4/4 YES.

**Mode B — small substrate gap (~25%).** Likely shapes:
- `keyword/of` doesn't accept the call shape sketched (needs different args or different helper); sonnet finds via grep
- Walker reshape requires deeper refactor than List-arm detection (e.g., the walker recurses into Vector children before checking shape; need to restructure recursion)
- `string::concat` / `keyword/to-string` need different calls than sketched

Sonnet surfaces, orchestrator decides (most likely: adjust macro body using the correct primitive; ship; SCORE notes the delta honestly).

**Mode C — STOP-trigger fires (~10%).** Most likely:
- Walker's bare-keyword arm fires before the List arm reaches the call site (mechanism gap — would need broader refactor than this stone's scope)
- A required substrate primitive doesn't exist + needs minting (would be Stone A precursor)
- Stdlib load order matters more than expected (e.g., spawn_program.wat depends on something loaded later)

Sonnet stops, surfaces, orchestrator decides.

## Strategy-decision prediction

Orchestrator's guess: BRIEF strategy LOCKED (pure-wat defmacro mirroring `wat/runtime.wat:17-32` pattern; walker reshape via List-arm context detection). Sonnet's discovery may:
- Find a cleaner walker reshape (e.g., shape-aware detection function shared with other walker arms)
- Find an existing helper for keyword construction that's simpler than `keyword/of + string::concat + keyword/to-string` chain
- Surface an additional test scenario worth covering

Sonnet picks the cleaner shape; orchestrator scores against the verified result.

## Pre-flight greps sonnet should re-run

Before code edits, sonnet verifies:
1. `grep -n "keyword/of\|keyword/to-string\|string::concat" src/macros.rs src/check.rs` — confirm these primitives exist with the call signatures the BRIEF sketches
2. `grep -n "WatAST::List\|WatAST::Keyword" src/check.rs | grep -i "spawn\|fork"` — understand current walker dispatch
3. `cat wat/runtime.wat | head -35` — re-read production precedent for expand-time substrate-call

If pre-flight surfaces gaps, the relevant STOP trigger fires.

## Out-of-scope findings (surface, don't act)

- **Tier whitelist enforcement at expand time** (e.g., explicit error if `:tier` is not `:thread`/`:process`). DEFER — type-checker catches via fn-resolution. If sonnet thinks this should ship in this stone, surface as honest delta.
- **`:wat::kernel::spawn-program-ast` reclaim** — DEFER. Stays rejected always per BRIEF.
- **Defservice-style code generation** — DEFER. Stone A is dispatch only; Stone C mints defservice.
- **Walker-arm consolidation** for OTHER `BareLegacy*` variants (BareLegacyForkProgram, BareLegacyMainSignature) — DEFER. Stone A reshapes only the spawn-program arm.

## Failure-mode catches

- FM 1 (grep before claiming): every substrate fact sonnet asserts cites file:line
- FM 9 (baseline pre-flight): sonnet runs workspace cargo test BEFORE editing; reports baseline failure count
- FM 14 (surface-retirement-leaving-leftovers): sonnet greps for spawn-program references in comments/docs that should reference the new shape; surface any found
- FM 16 (no tool preamble): BRIEF doesn't mention Bash/cargo availability
- `feedback_assertion_demands_evidence`: every substrate claim cites file:line

## What "done" means for this stone

4 SCORE rows YES; workspace cargo test green (= baseline ± 0); new file `wat/kernel/spawn_program.wat` exists with defmacro; walker shape-aware (legacy 2-arg rejected, new 3-arg accepted); stdlib loader updated; new test file with positive/negative/edge cases.

## Atomic commit shape

NO commit by sonnet. Orchestrator commits ALL files atomically when sonnet returns + scoring done. One commit: `wat/kernel/spawn_program.wat` + `src/check.rs` + `src/stdlib.rs` + `tests/wat_arc209_stone_a_spawn_program.rs`.

## Calibration record

Arc 170 Stone D1 (run-threads minimal bracket macro): similar shape (pure-wat defmacro using keyword/of); ~60 min sonnet. Arc 210 slice 1 (def-restricted keyword tag): smaller scope; predicted 45-75 min. Stone A predicted 90-150 min — bigger than D1 because adds walker reshape; smaller than full slice with consumer ripple.

If sonnet returns under 60 min, calibration data for over-specification.

## Connection to arc 209 forward chain

Stone A unblocks → Stone B (restricted_to application; uses arc 210's keyword shape if 210 shipped first) → Stone C (defservice defmacro; uses Stone A's spawn-program from generated start-fn wrappers) → Stone D (counter migration proof) → arc 209 closure paperwork → arc 203 closure unblocks → arc 170 closure unblocks → lab reconstruction.

## Dependency note on arc 210

This BRIEF was drafted while arc 210 slice 1 was in flight. Stone A does NOT directly depend on arc 210 (Stone A doesn't use defn-restricted). If arc 210 ships clean before Stone A spawns, Stone A's documentation can mention the new `:restricted-to` keyword shape; if arc 210 doesn't ship in time, Stone A's docs reference the OLD positional shape (which Stone C will need to update later when defservice generates defn-restricted forms).

Stone A's macro body uses `keyword/of` + `string::concat` + `keyword/to-string` (per BRIEF); none of these are touched by arc 210. Stone A and arc 210 are TRULY parallel substrate changes; either can ship first without blocking the other.

Sonnet: trust the substrate; trust the precedent at wat/runtime.wat:17-32; trust the disk; ship the atomic stone; return.
