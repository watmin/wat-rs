# Arc 158 — EXPECTATIONS (slice 1b)

**Drafted 2026-05-07 by orchestrator before sonnet spawn.**
Slice 1b = wat-rs consumer sweep (~951 sites; mechanical
transform of legacy let bindings to untyped form).

## Independent prediction

**Predicted runtime:** 25-40 min Mode A. **Time-box:** 60 min wall-
clock.

**Why this estimate:**
- Mechanical transform per binding (`((NAME TYPE) EXPR)` → `(NAME EXPR)`)
- ~951 sites distributed across many files (stdlib + tests + crates + examples + embedded Rust)
- Type-expr can be complex (parametric, function, tuple) — sed regex risky; AST-aware sweep or careful per-file tooling recommended
- Comparable to arc 155 sweep 1b (~476 sites in 12.5 min) but ~2× volume; 25-40 min is reasonable

**Mode classification:**
- **Mode A** (clean ship): workspace 73 failed → 0 failed; no other regressions; uniform mechanical transform.
- **Mode B**: 1-2 sites need manual handling because of unusual type-expr shape; sonnet patches.
- **Mode C**: tooling claim doesn't survive empirical check; sonnet's first sweep approach can't reach all sites; orchestrator decides on path forward.

## Expected scorecard rows

| Row | Expectation | Verification |
|---|---|---|
| **Workspace count** | 73 failed → 0 failed; passed climbs by ~73 to ~2039 | `cargo test --release --workspace` final count |
| **No new failures** | All regression failures must be NEW transforms going wrong (sonnet should self-detect) | `cargo test --release --workspace` shows 0 unexpected red |
| **`LegacyTypedLetBinding` count post-sweep** | 0 | `grep -cE "LegacyTypedLetBinding" $output` |
| **Excluded file** | `tests/wat_arc158_let_bindings.rs` UNTOUCHED | `git diff --stat tests/wat_arc158_let_bindings.rs` shows no change |
| **Atomic state** | Sonnet does NOT commit | `git log --oneline -3` |
| **Sweep coverage** | All buckets touched per BRIEF: stdlib + wat-tests + crates + examples + embedded Rust | Sonnet's report enumerates buckets |

## Honest delta candidates

- **Tooling choice:** sed regex CAN handle most cases but the type-expr can include parens (`Fn(...)` etc.) which fight regex. Sonnet may need ast-aware tool (python with sexp parser?) or per-file manual care. Per memory `feedback_collapse_to_llm_in_loop.md`: empirically verify the tool works before claiming it's the only option.
- **Multi-line bindings:** some legacy bindings span lines for readability. Sweep tool should preserve formatting where possible.
- **Comments referencing legacy shape:** `;; (((x :T) ...))` style comments are documentation, not code. Leave alone (or update to canonical form for consistency — sonnet's call).
- **Edge case: bindings that pun:** `((let-x :wat::core::let) ...)` — name is `let-x`, type is `:wat::core::let` (the keyword). Unlikely but possible; sweep should handle.

## SCORE methodology

After 1b returns:
- Verify workspace = 2039 / 0 (or whatever the math says)
- Spot-check 3-5 swept files for clean transform
- If clean: orchestrator commits 1a + 1b atomically with combined message; pushes
- If 1-2 patch sites: orchestrator patches OR re-spawns sonnet for the patch

## Pre-flight checklist (orchestrator runs BEFORE spawn)

- [x] DESIGN.md current
- [x] BRIEF-SLICE-1b.md drafted
- [x] EXPECTATIONS-SLICE-1b.md drafted (this commit)
- [x] 1a verified (sonnet's report; 73 failures all expected)
- [ ] Commit BRIEF + EXPECTATIONS for 1b
- [ ] `model: "sonnet"` set on Agent call (FM 12)
- [ ] `run_in_background: true` set on Agent call
- [ ] ScheduleWakeup at 60 min (3600s) post-spawn

## Why slice 1b after 1a

Per recovery doc § 7: substrate (1a) leaves working tree dirty
with EXPECTED breakage (`LegacyTypedLetBinding` firing on legacy
sites). Sweep 1b runs against the dirty tree, fixes consumer
sites, brings workspace to green. Orchestrator commits 1a + 1b
atomically when workspace = 0 failed.

This is the proven pattern from arcs 154 + 155 (and
substrate-as-teacher more broadly). Mode A clean ship requires
both pieces in the same commit; partial state on disk is
not acceptable per `feedback_no_broken_commits.md`.
