# Arc 170 slice 1f-β-iii — SCORE

**Result:** Mode A clean. 15/15 rows pass.
**Runtime:** ~5 min sonnet (well under predicted 10-15 band; well under 30 hard cap).
**Files:** 2 new + 1 modified — `wat/kernel/services/stderr.wat`, `wat-tests/kernel/services/stderr.wat`, `src/stdlib.rs`.

**Trio complete — all three wat-side substrate stdio services minted.**

## Calibration

- **Predicted runtime band:** 10-15 min (sonnet pattern-apply third in the family)
- **Actual:** ~5 min — 2-3× faster than predicted
- **Why faster:** Pattern was a near-clone of β-ii with namespace swap only. BRIEF's explicit call-out of the data-channel-Event dispatch shape eliminated the one delta β-ii surfaced.
- **Calibration loop:** β-i (~25 min mint) → β-ii (~8 min first apply) → β-iii (~5 min near-clone). The asymptote for pattern-apply slices in a settled family is ≤5 min sonnet.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `stderr.wat` parses, type-checks | ✓ `cargo check --release` green |
| B | `Event` enum declared with Write/Add/Remove variants | ✓ |
| C | Channel typealiases (EventTx/EventRx/Routing/Spawn) | ✓ |
| D | `spawn` fn returns tuple `::Spawn` | ✓ |
| E | TCO driver loop with Vector routing | ✓ |
| F | `:wat::kernel::select` over routing + control-rx | ✓ |
| G | Add/Remove mutate routing via conj/filter | ✓ |
| H | Write triggers `IOWriter/writeln` + ack-tx send `()` | ✓ |
| I | Data-channel match has Write/Add/Remove arms (defensive completeness) | ✓ — explicit per BRIEF |
| J | `wat-tests/kernel/services/stderr.wat` exists with 5 deftest-hermetic rows | ✓ |
| K | Workspace 1339/869 (baseline 1339/864 +5) | ✓ confirmed |
| L | `cargo check --release` green | ✓ clean (1 pre-existing dead_code warning) |
| M | Exactly 3 files modified | ✓ git status confirms |
| N | Zero new deps; zero Mutex/RwLock/CondVar | ✓ |
| O | Honest deltas surfaced | ✓ none beyond β-ii's |

**15/15 rows pass.** Mode A clean.

## Workspace state

- **Pre-1f-β-iii baseline:** 1339 passed / 864 failed (post-1f-β-ii)
- **Post-1f-β-iii:** 1339 passed / 869 failed
- **Delta:** +0 passed / +5 failed — 5 new stderr hermetic tests at § Row K architectural blocker
- **Cumulative § Row K cost from the trio:** +15 failed (5 per service × 3 services)

All 15 of these clear when `deftest-hermetic` migrates to `spawn-process` adjacent to slice 1f-δ's INSCRIPTION.

## Honest deltas

None beyond what β-ii already surfaced. The BRIEF's explicit call-out of the data-channel-Event dispatch shape (Write / Add / Remove with defensive Add/Remove no-op arms on data channel) eliminated the one delta that surfaced in β-ii. Pattern is now fully captured.

## Implementation choices (locked)

Identical to slice 1f-β-ii. Search-and-replace `StdOutService → StdErrService`. No structural changes.

## Files modified

- `wat/kernel/services/stderr.wat` (new, 335 lines) — wat-side StdErrService
- `wat-tests/kernel/services/stderr.wat` (new, 339 lines) — 5 hermetic test rows
- `src/stdlib.rs` (+8 lines) — registration entry after stdout.wat

## Lessons captured

1. **Pattern-apply asymptote.** β-i mint at ~25 min; β-ii first apply at ~8 min; β-iii near-clone at ~5 min. Family settled; future pattern-applies in this shape should predict ≤5 min sonnet.

2. **BRIEF call-outs of prior-slice deltas eliminate the re-surface.** Slice 1f-β-ii surfaced the data-channel-dispatch shape as a delta. β-iii's BRIEF spelled out the three-arm match shape explicitly; sonnet shipped it verbatim with no friction. Discipline: when a slice surfaces a delta, the next slice's BRIEF should call it out so it isn't a delta there.

3. **Trio milestone.** All three wat-side substrate stdio services exist. Next architectural step (slice 1f-γ) is the substrate Rust orchestrator that wires `spawn-thread` to boot each service program and threads ThreadIO through the per-thread cell.

## What's next

1. **Atomic-commit slice 1f-β-iii** (this turn) — 3 modified files + this SCORE
2. **Slice 1f-γ** — runtime orchestrator (substrate Rust work):
   - Spawn each service program at runtime boot
   - Generate ThreadIO per-thread on `spawn-thread`
   - Send Add events to each service to register the new thread
   - Send Remove events on thread reap
   - Install ThreadIO into the spawned thread's thread-local cell
3. Then slice 1f-δ — boot integration + shutdown cascade
4. Adjacent to 1f-δ INSCRIPTION: `deftest-hermetic` migrates to `spawn-process`; § Row K closes; the +15 trio failures (and the broader 854 baseline) resolve

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-B-III.md`](./BRIEF-SLICE-1F-B-III.md)
- Pattern source: `wat/kernel/services/stdout.wat` (slice 1f-β-ii, committed `fe9b9e9`)
- Predecessor SCORE: [`SCORE-SLICE-1F-B-II.md`](./SCORE-SLICE-1F-B-II.md)
- Successor: slice 1f-γ — substrate runtime orchestrator
- Architecture: REALIZATIONS pass 18 (unified Event protocol); TIERS.md § OS-boundary handling
- § Row K (architectural blocker): tracked for closure adjacent to slice 1f-δ INSCRIPTION
