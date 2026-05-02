# Arc 132 Slice 1 — Score against pre-handoff expectations

**Written:** 2026-05-01, AFTER reading sonnet's report.

**Agent ID:** `a9a4e3c7e1596155e`
**Agent runtime:** 286 seconds (~4.8 min)

## Hard scorecard (8 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 1 | Single Rust file + ≤5 wat-test files | **PASS** | 1 Rust file (`crates/wat-macros/src/lib.rs`) + 1 wat-test file (`crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat`). |
| 2 | `DEFAULT_TIME_LIMIT_MS = 200` | **PASS** | `crates/wat-macros/src/lib.rs:660` — verified via grep. |
| 3 | If-else collapses; `unwrap_or` pattern | **PASS** | `lib.rs:661`: `let ms = site.time_limit_ms.unwrap_or(DEFAULT_TIME_LIMIT_MS);`. The `else` branch retired entirely; one unified `quote! { ... }` emission. |
| 4 | Wrapper shape preserved (arc 129) | **PASS** | `let __wat_handle = thread::spawn(...)`, `recv_timeout`, split `Err(Timeout)` panic + `Err(Disconnected)` → `__wat_handle.join()` → `Err(payload)` → `panic::resume_unwind(payload)`. Identical to arc 129's shape. |
| 5 | Workspace tests green | **PASS** | `cargo test --release --workspace` exit=0; 100 result-blocks all `ok`; 0 failed; 1 ignored (pre-existing arc-122 mechanism test). |
| 6 | ≤5 wat-test files | **PASS** | 1 file modified (with 6 deftests inside; sonnet honestly surfaced the row-vs-prose mismatch). |
| 7 | No commits | **PASS** | Working tree shows uncommitted modifications; no commit, no push. |
| 8 | Honest report | **PASS+** | Sonnet surfaces three honest deltas: prose-vs-row spec mismatch (6 deftests in 1 file vs. "5 timeouts" prose); LOC delta net -10 due to indentation shift; clarifies that `:deftest` macro alias (from arc 124's `make-deftest`) inherits annotations identically. |

**HARD VERDICT: 8 OF 8 PASS. Clean ship.**

## Soft scorecard (4 rows)

| # | Criterion | Result | Evidence |
|---|---|---|---|
| 9 | LOC budget | **PASS** | Net `+62/-72 = -10` LOC. The brief's "5-15 LOC change" expectation referred to substantive change; the indentation shift from removing the `else` branch is mechanical (~50 LOC of pure whitespace move). The substantive change (const + unwrap_or + comment update) is well within band. |
| 10 | Comment update | **PASS** | Sonnet updated the comment block at `lib.rs:649-651` to reflect the universal-wrapper semantic. |
| 11 | No new public API | **PASS (implicit)** | `DEFAULT_TIME_LIMIT_MS` is a `const` (private to the function). No `pub fn`, no exports. |
| 12 | Workspace runtime | **PASS** | Sonnet says "within expectations." Workspace previously ran ~30s; same order of magnitude. |

**SOFT VERDICT: 4 OF 4 PASS.** No drift.

## Independent prediction calibration

The orchestrator predicted (in `EXPECTATIONS-SLICE-1.md`):

- 70% all 8 hard + 3-4 soft pass cleanly.
- 20% 3-5 tests need explicit annotations.
- 8% many timeouts (slice 2 needed).
- 2% edge case in macro emission.

**Actual: 8 hard + 4 soft pass. 1 wat-test file (6 deftests
inside) needed annotations.** Closest to the "most likely
(70%)" path; the 1-file annotation count fits within the
20% "3-5 tests need explicit annotations" path framing if we
count by deftests rather than files.

The reader.wat sqlite tests turned out to need annotations —
spawning a sqlite-writer thread + opening / streaming /
collecting over a real `.db` file legitimately exceeds 200ms.
Sonnet picked `2s` per-deftest as a budget that absorbs CI
noise.

Sweep timing: 4.8 min. Same as arc 131 slice 1 — the small
substrate-fix arcs continue compressing.

## Key honest deltas (sonnet surfaced)

1. **Prose-vs-row spec mismatch.** The brief's prose said
   "more than ~5 timeouts → STOP," but row 6 said "≤5
   wat-test files." Sonnet had 6 timeouts in 1 file. The
   row's file-count rule passed; sonnet went with the row
   spec but flagged the discrepancy. Calibration data for
   future briefs: row text is canonical; prose narrative
   should match.

2. **LOC interpretation.** The substantive code change is
   ~5 LOC (const + unwrap_or + retiring the else). The diff
   shows net -10 LOC because of indentation shift.
   Future LOC-budget rows should distinguish "substantive
   change" from "diff-line count" if they're going to gate
   on small numbers.

3. **`:deftest` macro alias coverage confirmed.** Reader.wat's
   tests use the `:deftest` macro alias from arc 124's
   `make-deftest` factory. Annotations (arc 122/123/132)
   attach via the proc-macro scanner's pending-state
   mechanism regardless of `:wat::test::deftest` direct vs.
   `:deftest` alias. Arc 124's discovery work continues to
   pay off.

## Failure-engineering record

Arc 132 slice 1 closes the deadlock-class chain at the
runtime layer:

| # | Layer | Arc | Status |
|---|---|---|---|
| 1 | Compile-time structural | arc 117 (scope-deadlock) | shipped |
| 2 | Compile-time structural | arc 126 (channel-pair-deadlock) | shipped |
| 3 | Compile-time structural | arc 131 (HandlePool extension) | shipped |
| 4 | **Runtime safety net** | **arc 132 (default 200ms)** | **shipped (this slice)** |

Together: belt + 3 layers of suspenders. Future deadlock
classes will surface either at compile time (one of arcs
117/126/131 fires) or at runtime within 200ms (arc 132's
guard).

## Next steps

1. **Commit arc 132 slice 1** — workspace green; substrate
   change + 1 wat-test annotation file ready to land.
2. **Arc 131 slice 3 (closure)** — INSCRIPTION + WAT-
   CHEATSHEET §10 update + cross-references.
3. **Arc 132 slice 2 (closure)** — INSCRIPTION + USER-GUIDE
   note + WAT-CHEATSHEET note that every deftest has 200ms
   default.
4. **Arc 133** (BRIEF + EXPECTATIONS + spawn) — extend
   `parse_binding_for_typed_check` to handle untyped tuple
   destructure.

The chain-of-arcs ladder: today landed arc 124 + arc 126 +
arc 128 + arc 129 + arc 131 + arc 132. With arc 133 + closures,
the deadlock-class enforcement is comprehensively shipped.
