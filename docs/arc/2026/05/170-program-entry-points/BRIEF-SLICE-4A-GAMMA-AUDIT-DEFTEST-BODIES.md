# Arc 170 Slice 4a-γ-audit BRIEF — scan all deftest bodies; produce three-rule worklist

**Task:** #317
**Phase:** Slice 4a-γ first sub-stone (audit → decorate → flip). See `INTERSTITIAL-REALIZATIONS.md` § 2026-05-14 "Mid-session breadcrumb" for the rescope rationale; see `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 7-ter for the three-rule classification (canonical substrate fact).
**Predecessors:** 4a-α ✓ (`ddb3cad`) + 4a-β ✓ (`3536f12`). The mint + caller sweep are settled. This stone is the prerequisite analysis for the deftest macro flip — produces a per-site worklist that drives #318 (decorate) safely.

## Goal

Produce a structured audit report of every `:wat::test::deftest` body in the codebase, classified by the three-rule check. The report's worklist tells #318 (decorate) which deftests need `-hermetic` decoration BEFORE the macro flip at #314 lands. **NO code edits in this slice.** Pure information; visibility before semantic shift.

## The three-rule classification (from FM 7-ter; restated for this BRIEF)

Any deftest body exhibiting ANY of these traits MUST keep hermetic semantics (after the flip lands, that means renaming the deftest to `:wat::test::deftest-hermetic`):

1. **Reads `RunResult.stdout` / `RunResult.stderr` slots.** Threads return empty stdio Vecs; tests asserting on captured output need process pipes (run-hermetic). Look for: `RunResult/stdout`, `RunResult/stderr`, `assert-stdout-is`, `assert-stderr-matches`, `assert-stdout-contains`, etc.

2. **Calls `:wat::kernel::println` / `eprintln` / `readln` in the body.** Stdio verbs in thread context route to ambient services that share parent's fd 0/1/2 (pollution; no per-test capture). Process context gives the child its own captured fd. Look for: `:wat::kernel::println`, `:wat::kernel::eprintln`, `:wat::kernel::readln`.

3. **Calls `:wat::config::set-*!` family verbs in the body.** Per-runtime config mutation. Threads share the parent's runtime — `set-*!` from a thread mutates state the parent reads. ILLEGAL cross-thread. Look for: `:wat::config::set-capacity-mode!`, `:wat::config::set-dim-router!`, `:wat::config::set-redef!`, `:wat::config::set-eval-redef!`, and any other `:wat::config::set-*!` form.

If a body exhibits NONE of the three: safe for `run-thread` after the flip; stays as plain `:wat::test::deftest`.

## Survey scope

The deftest population (`grep -rEc ":wat::test::deftest\b"` across the codebase, filtered):

| Location | Site count (approx) | Notes |
|---|---|---|
| `wat-tests/` | ~190 | bulk of population — holon/ (~60), test.wat (16), time.wat (38), edn/ (10), stream.wat (13), core/ (~9), kernel/ etc. |
| `tests/` | ~7 | tests/wat_make_deftest.rs has 7; mostly Rust files with embedded wat strings |
| `crates/` | ~12 | wat-sqlite, wat-lru, wat-holon-lru, wat-macros |
| `examples/` | ~1 | with-loader |

**Estimated total: ~224 active `:wat::test::deftest` sites.** The audit covers all of them.

ALSO classify `:wat::test::deftest-hermetic` callers — they're ALREADY hermetic and don't need decoration, but the audit report should confirm the rule-firing rate among them (sanity check: most deftest-hermetic sites SHOULD exhibit at least one rule, otherwise they're using hermetic unnecessarily).

## Output: structured audit doc

Write `docs/arc/2026/05/170-program-entry-points/AUDIT-SLICE-4A-GAMMA-DEFTEST-BODIES.md`. Format:

```markdown
# Arc 170 Slice 4a-γ-audit AUDIT — deftest body three-rule classification

**Task:** #317
**BRIEF:** BRIEF-SLICE-4A-GAMMA-AUDIT-DEFTEST-BODIES.md
**Substrate rule reference:** docs/COMPACTION-AMNESIA-RECOVERY.md § FM 7-ter

## Summary counts

| Total deftest sites | Safe for thread | Rule R1 (stdio reads) | Rule R2 (stdio verbs) | Rule R3 (set-! family) | Multiple rules | Total flagged |
|---|---|---|---|---|---|---|
| TBD | TBD | TBD | TBD | TBD | TBD | TBD |

## Flagged sites (must become deftest-hermetic after flip)

(One row per flagged site.)

| File:line | Test name | Rules fired | Rationale |
|---|---|---|---|
| ... | ... | R1, R3 | reads RunResult.stdout + calls set-capacity-mode! |

## Safe sites (stay as plain deftest after flip)

(Same shape; OR a count + "see grep output for full list" if too many to enumerate; orchestrator's call. Aim to enumerate by file at minimum.)

## deftest-hermetic sanity check

(Confirms each existing deftest-hermetic site exhibits at least one rule.)

| File:line | Test name | Rules fired | Notes |
|---|---|---|---|
| ... | ... | R2 | calls :wat::kernel::println in body (legitimate hermetic) |

(If any deftest-hermetic site exhibits ZERO rules → flag in honest deltas as "potentially over-hermetic; could downgrade after flip.")

## Honest deltas

(Anything surprising: helper-wrapped patterns, indirect rule-firing through helper calls, etc.)

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | TBD | TBD |
| Total deftest sites audited | ~224 | TBD |
| Flagged for decoration | TBD-prediction | TBD |
| deftest-hermetic over-hermetic candidates | 0–5 | TBD |
| Mode | A (clean) | TBD |
```

## Constraints (HARD)

- **NO code edits.** This is pure audit; producing a worklist for #318. Don't migrate any sites; don't rename any sites; don't touch `wat/test.wat` macros; don't touch any test bodies.
- **Operate ONLY in `/home/watmin/work/holon/wat-rs/`** per `feedback_no_worktrees` + FM 7-bis. If you see `.claude/worktrees/agent-<id>/` in your cwd: `cd /home/watmin/work/holon/wat-rs/` immediately. Use `git -C /home/watmin/work/holon/wat-rs` for git operations. Use absolute paths under `/home/watmin/work/holon/wat-rs/` for all file operations.
- DO NOT commit. Orchestrator commits atomically after independent verification.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs.
- DO NOT modify the recovery doc, INTERSTITIAL, or this BRIEF.

## Method

1. **Verify cwd** (`pwd` — must be `/home/watmin/work/holon/wat-rs/`).
2. **Enumerate** every `:wat::test::deftest` site:
   ```
   grep -rEn ":wat::test::deftest\b" /home/watmin/work/holon/wat-rs/wat-tests/ /home/watmin/work/holon/wat-rs/tests/ /home/watmin/work/holon/wat-rs/crates/ /home/watmin/work/holon/wat-rs/examples/
   ```
   Filter out comments (`;;` prefix), .md files, doc comments.
3. **For each site, read the deftest body.** The body is the third argument of the macro:
   ```
   (:wat::test::deftest :test-name (prelude) BODY)
   ```
   The body is what runs inside `run-hermetic` (today) / `run-thread` (after the flip). Surveyed depth: read the body's top-level forms + descend through `(:wat::core::do ...)`, `(:wat::core::let ...)`, `(:wat::core::match ...)`. Helper function calls in the body don't count UNLESS you can determine the helper itself fires a rule (note in honest deltas if helpers obscure the classification).
4. **Apply three-rule check.** Per-body, note which rules fire (R1 / R2 / R3 / none).
5. **Same for `:wat::test::deftest-hermetic`** sites — they don't need decoration; the audit confirms they're legitimately hermetic. Flag any with zero rules firing as potential over-hermetic.
6. **Build the audit doc.** Tables per the output spec above. Each flagged site enumerated with file:line, test name, rules fired, and one-line rationale.
7. **STOP-at-first-red:** if any classification is genuinely ambiguous (e.g., helper-wrapped indirect rule-firing without clear answer), surface in honest deltas; don't force a YES/NO call. Orchestrator decides during #318 BRIEF drafting.

## STOP triggers

- A deftest body's body argument can't be unambiguously identified (e.g., parses oddly, macro-of-macro, etc.) → STOP that site, surface in honest deltas.
- Helper function call obscures classification (helper called from body that itself fires a rule, but the deftest body's lexical text doesn't) → flag the helper; surface; orchestrator decides whether to follow the indirection or treat lexically.
- > 30% of deftest sites flag → STOP and surface; the classification rule may need refinement before mass decoration.

## Time-box

EXPECTATIONS will set the band. Hard stop at 2× upper-bound; surface partial audit if reached.

## On completion

Write `AUDIT-SLICE-4A-GAMMA-DEFTEST-BODIES.md` per the format above. Return: doc path + total-flagged count + most surprising delta (if any).

Do NOT commit. Orchestrator commits atomically after independent verification.
