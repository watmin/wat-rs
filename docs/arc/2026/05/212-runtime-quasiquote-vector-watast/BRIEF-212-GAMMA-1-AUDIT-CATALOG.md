# Arc 212 stone γ-1 — Audit catalog (READ-ONLY)

**Your ONE concern this spawn:** produce a markdown table cataloging every site in `src/*.rs` and `crates/*/src/*.rs` that pattern-matches on `WatAST` enum variants. Classify each site. Write the table to a new file. Nothing else.

**You are not migrating anything. You are not editing code. You are not running tests for failure investigation. You are producing a catalog.**

---

## Background context (read so you know WHY this stone exists; do not act on this section)

Arc 212 eliminates the class of "walker silently skips Vector" bugs at progressively stricter enforcement layers (L0 → L4). The class was first surfaced by t6 (`tests/wat_arc170_program_contracts.rs`). The substrate primitive `WatAST::children()` already shipped at slice β (commit `bc31342`). A prior spawn already migrated 12 walkers to use it (committed atomically with this BRIEF). Two walkers (`validate_comm_positions`, `collect_process_calls`) are inscribed as sharpening targets for later stones (δ-comm-positions, δ-process-scope). This stone — γ-1 — produces the comprehensive audit catalog that confirms coverage and surfaces any remaining unmigrated walkers.

**Per arc 212 DESIGN § "Locked stone chain":** γ-1 is the audit-only stone. δ-bare-primitives, δ-comm-positions, δ-process-scope, ζ-newtype-wall, η-visitor are FUTURE stones. They are not your concern this spawn.

---

## The catalog you produce

**File to write:** `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-GAMMA-1-AUDIT-CATALOG.md`

**Format:** ONE markdown table with these columns:

| file:line | fn name | classification | reason |

**Classifications:**

- **Walker (already migrated)** — function uses `node.children()` for generic recursion; walker-specific List-head logic stays in `if let WatAST::List(items, _)`. Already in the children() shape per arc 212 doctrine.
- **Walker (pending migration)** — function still recurses through `WatAST::List(items, _)` items only, without calling `node.children()`. Needs a future δ-N stone.
- **Walker (sharpening target)** — function tried to migrate but breaks under naive children() recursion; needs walker-rule sharpening (not exemption). Two known: `validate_comm_positions` + `collect_process_calls`.
- **Leaf-decomposition** — function pattern-matches on `WatAST` to extract ONE shape's parts but does NOT recurse on children. Examples: parsers, classifiers, single-shape handlers. Stays as-is forever (no migration applies; not a walker).

**Reason column:** ONE sentence. For Walker (pending migration), name the recursion site. For Leaf-decomposition, say what shape it decomposes and why no recursion.

---

## How to find sites

```bash
cd /home/watmin/work/holon/wat-rs
grep -rn "WatAST::List(" src/ crates/*/src/ 2>/dev/null | grep -v "^[^:]*:[0-9]*://" > /tmp/audit-watast-list-sites.txt
grep -rn "WatAST::Vector(" src/ crates/*/src/ 2>/dev/null >> /tmp/audit-watast-list-sites.txt
grep -rn "WatAST::StructPattern(" src/ crates/*/src/ 2>/dev/null >> /tmp/audit-watast-list-sites.txt
```

For each site: locate the enclosing function (read surrounding lines). Classify per the four classifications above.

**The 12 already-migrated walkers** (you'll find these classified as "Walker (already migrated)"):
- `src/resolve.rs::check_form`
- `src/resolve.rs::check_quasiquote_template`
- `src/check.rs::validate_sandbox_scope_leak`
- `src/check.rs::check_calls_for_sandbox_leak`
- `src/check.rs::walk_for_legacy_stream`
- `src/check.rs::walk_for_legacy_telemetry_service`
- `src/check.rs::walk_for_legacy_lru_cache_service`
- `src/check.rs::walk_for_legacy_kernel_queue`
- `src/check.rs::walk_for_deadlock`
- `src/check.rs::contains_join_on_thread`
- `src/check.rs::walk_for_pair_deadlock`
- `src/check.rs::node_contains_recv`

**Already-correct walkers** (verify they use children() OR explicit List+Vector arms; classify accordingly):
- `src/macros.rs::walk_template`
- `src/macros.rs::substitute_bindings`
- `src/runtime.rs::walk_quasiquote`
- `src/check.rs::walk_for_arc170_legacy`
- `src/check.rs::walk_for_bare_legacy_console`
- `src/check.rs::walk_for_def_restricted_call`
- Various in `src/closure_extract.rs`

**Known sharpening targets** (classify as Walker — sharpening target):
- `src/check.rs::validate_comm_positions` (line ~2137; see inscribed comment)
- `src/check.rs::collect_process_calls` (line ~3596; see inscribed comment)

**Known pending walker** (BRIEF flagged but not yet migrated):
- `src/check.rs::walk_for_bare_primitives` (~line 2705)

**Likely sources of Leaf-decomposition sites:** parsers in `src/macros.rs`, `src/closure_extract.rs`, `src/types.rs`, `src/hash.rs`, `src/dispatch.rs`, `src/lower.rs`, `src/load.rs`, `src/config.rs`, `src/freeze.rs`, `src/form_match.rs`. Most of these are NOT walkers — they decompose one shape for one purpose without recursing.

---

## STOP triggers — VERBATIM

The following triggers are non-negotiable. If any fires, STOP IMMEDIATELY. Do not investigate. Do not theorize. Do not open the file. Return what you have.

1. **You see a failing test.** STOP. This is a read-only audit. Do not investigate test failures. Workspace failure count is NOT your concern.
2. **You feel the urge to migrate a walker you found unmigrated.** STOP. This stone is read-only. Migration happens in future δ-N stones. Catalog the site; do not edit it.
3. **You feel the urge to investigate why a walker breaks under children().** STOP. The two known sharpening targets (`validate_comm_positions`, `collect_process_calls`) already have their reasoning inscribed in code comments. Do not re-discover.
4. **You feel the urge to look at any test file.** STOP. The audit is in `src/*.rs` + `crates/*/src/*.rs`. Test files (`tests/`, `wat-tests/`) are out of scope.
5. **Anything outside this concern surfaces.** STOP. Return what you have. The orchestrator handles the surface.

If you hit a STOP trigger, report what triggered it in your SCORE — that IS valuable information. Returning early with a partial catalog + a clean STOP-trigger report is a Mode A outcome.

---

## What success looks like

A markdown file at `docs/arc/2026/05/212-runtime-quasiquote-vector-watast/SCORE-212-GAMMA-1-AUDIT-CATALOG.md` containing:

1. Header: `# Arc 212 stone γ-1 — SCORE: walker audit catalog`
2. Summary line: total sites inspected, count per classification
3. The markdown table (all sites classified)
4. (Optional) Notes section: anything surprising you found that future stones should know about (e.g., a Walker (pending migration) the orchestrator didn't pre-name; a site where classification was ambiguous and you picked one — explain why)

No SCORE-formatted scorecard rows. No verification commands. No "I ran cargo test." No mention of workspace test status. Just the catalog.

---

## Constraints

- Zero code edits anywhere
- Zero new files anywhere except the SCORE file above
- Zero commits (orchestrator commits)
- No git operations
- No cargo invocations
- No test runs

---

## Time prediction

20-40 min. Audit is bounded (~50 grep hits); per-site classification is fast.

---

## Mode classification

- **Mode A:** catalog complete; all sites classified; SCORE file written; report returned
- **Mode B:** catalog partial; you hit a STOP trigger and stopped honestly; SCORE captures what's classified + names what triggered
- **Mode C:** you broke a STOP rule (started investigating something out of scope); the work is invalid and the orchestrator will discard

The substrate teaches; you listen; you catalog; nothing else.
