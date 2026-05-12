# Arc 170 slice 3 — Gap C V2 EXPECTATIONS (sonnet scorecard)

**One spawn.** Extend EVERY top-level form-consuming substrate pass to recurse into `(:wat::core::do ...)` — completing arc 136's design that arc 157 partially extended. Three probes pass; workspace stays at 0 failed.

## Independent prediction

**Runtime band:** 45-120 min sonnet. Identifying passes is the bulk; each pass's `do` arm is small (~5-10 LOC).

**Hard cap:** 240 min.

## Scorecard (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | All top-level form-consuming passes identified + extended | grep + SCORE inventory of ALL passes touched |
| B | Probe 1 (`do` of two `def` forms) passes | cargo test |
| C | Probe 2 (`do` of two `defn` forms via expansion) passes | cargo test |
| D | Probe 3 (defmacro-emitted `do` wrapping `defn`) passes | cargo test |
| E | Workspace at 0 failed | full cargo test |
| F | `cargo check --release` green | clean |

## Implementation approach

### Phase 1 — Write the three probes as failing baseline

Create `tests/probe_do_splice_def.rs` with the three tests from the BRIEF. Confirm all three FAIL with the resolve-time call-head-lookup error. The probes ARE the regression test set; they become canonical once Gap C ships.

### Phase 2 — Inventory passes

Grep + read to identify EVERY substrate pass that walks top-level forms. Pattern to find: any function that iterates through a `&[WatAST]` or similar at top level and handles `define` / `def` / `defn` / `struct` / `enum` / `typealias` / `defmacro` / etc. Check if each already recurses into `(:wat::core::do ...)`.

Known passes (verify + extend):
- `register_defines` / `register_stdlib_defines` (src/runtime.rs)
- `register_struct_methods` / `register_enum_methods` / `register_newtype_methods` (src/runtime.rs)
- `register_defmacros` / `register_stdlib_defmacros` (src/macros.rs)
- `register_types` / `register_stdlib_types` (src/types.rs)
- `resolve_references` (src/resolve.rs) — for call-head resolution
- `check_program` (src/check.rs) — verify behavior across top-level forms
- Any other top-level-form-consumer

Passes that ALREADY handle `do` (verify + don't change):
- `register_runtime_defs` (runtime.rs:2018-2023)
- `collect_splice_defs_ctx` (check.rs:6848)

### Phase 3 — Extend each missing pass

Add a `do` arm to each pass that walks top-level forms. Mirror the pattern from `register_runtime_defs`:

```rust
WatAST::List(items, _) if matches!(items.first(),
    Some(WatAST::Keyword(k, _)) if k == ":wat::core::do") => {
    for child in &items[1..] {
        // recurse with the same top-level context
        Self::recurse_call(child, ...);
    }
}
```

### Phase 4 — Verify

- All three probes pass
- Full workspace cargo test: 2199+ passed / 0 failed

### Phase 5 — Check parallel `let` gap

The arc 157 doctrine ALSO documents `let` at top level as splicing for def. Sonnet checks: does the same gap exist for `let`? If yes, surface as Gap D follow-up (do NOT fix this slice — out of scope).

## What sonnet should produce

1. **Code changes:**
   - Multiple src/ files extended (each top-level form-consuming pass gets a `do` arm)
   - `tests/probe_do_splice_def.rs` — the three probe tests (committed as regression suite)
2. **SCORE doc:** `docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-GAP-C-V2-DO-SPLICE-DEF.md`
   - Scorecard verification
   - Complete inventory of passes extended (which ones got the do arm, with line refs)
   - Resolve pass mechanism notes (most subtle one)
   - Top-level `let` gap status (does it exist? surface only)
   - Honest deltas (≥ 3)
3. **Do NOT commit.** Orchestrator atomic-commits after scoring verification.

## What sonnet should NOT do

- Do NOT modify deftest / deftest-hermetic macros (Phase E V3 work)
- Do NOT modify Layer 1/2 macros / drivers
- Do NOT retire run-sandboxed-* substrate verbs (Phase F)
- Do NOT touch BareLegacy* / spawn.rs / Process<I,O> struct fields
- Do NOT rename `define` → `defn` workspace-wide (separate arc 109 follow-up)
- Do NOT extend `let` top-level splicing (separate concern; surface only)
- Do NOT use deferral language in SCORE
- If extending passes causes unexpected breakages, STOP and report (root cause per test, no workarounds)

## Tools required

- Read / Edit / Bash (cargo, git, grep)
- Write for probe test file + SCORE
- No Agent invocations

## Verification commands

```bash
# Baseline
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# Probe verification
cargo test --release --test probe_do_splice_def 2>&1 | tail -10

# Workspace verify after fix
cargo test --release --workspace --no-fail-fast 2>&1 | \
  grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
```

## Expected workspace delta

- Baseline: 2199 passed / 0 failed
- Post Gap C V2: 2199+3 passed / 0 failed (probes added + passing)

## Honest delta categories (anticipated)

1. **Complete pass inventory** — list every function extended with a `do` arm
2. **Resolve pass mechanism** — the subtlest; how call-head resolution sees-through `do`
3. **Top-level `let` parallel gap** — surface presence/absence; do not fix
4. **Workspace impact** — any tests behaving differently with new do splice recognition
5. **Anything unexpected**
