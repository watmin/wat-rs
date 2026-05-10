# Arc 170 slice 1f-β-ii — SCORE

**Result:** Mode A clean. 15/15 rows pass.
**Runtime:** ~8 min sonnet (well under predicted 30-45 band; well under 90 hard cap).
**Files:** 2 new + 1 modified — `wat/kernel/services/stdout.wat`, `wat-tests/kernel/services/stdout.wat`, `src/stdlib.rs`.

## Calibration

- **Predicted runtime band:** 30-45 min (sonnet pattern-apply post-1f-β-i)
- **Actual:** ~8 min — 3-5× faster than predicted
- **Why faster:** Pattern was well-trodden post-1f-β-i. BRIEF supplied verbatim Event-enum shape + side-by-side comparison table + substrate-grep citations. Sonnet had to swap one verb (`read-line` → `writeln`), one channel value type (`HolonAST` → `nil`), and the data-channel dispatch shape; everything else mechanical mirror.
- **Calibration update:** future pattern-apply slices post-1f-β-i should predict 10-15 min sonnet, not 30-45.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `wat/kernel/services/stdout.wat` parses, type-checks | ✓ `cargo check --release` green |
| B | `Event` enum declared with Write/Add/Remove variants | ✓ |
| C | Channel typealiases (EventTx/EventRx/Routing/Spawn) | ✓ all four present |
| D | `spawn` fn returns tuple `::Spawn` | ✓ |
| E | TCO driver loop with Vector routing | ✓ `StdOutService/loop` |
| F | `:wat::kernel::select` over routing values + control-rx | ✓ |
| G | `Event::Add` / `Event::Remove` mutate routing via conj/filter | ✓ |
| H | `Event::Write` triggers `IOWriter/writeln` + ack-tx send | ✓ handle-write does both |
| I | `wat-tests/kernel/services/stdout.wat` exists with ≥ 4 deftest-hermetic rows | ✓ 5 rows; all 5 in `--list` |
| J | Workspace within ±5 of post-1f-β-i baseline (1339/859) | ✓ 1339/864 — exactly +5 failed matches +5 new tests at § Row K blocker |
| K | `cargo check --release` green | ✓ clean (1 pre-existing `dead_code` warning) |
| L | Exactly 3 files modified | ✓ git status confirms |
| M | Zero new dependencies | ✓ Cargo.toml unchanged |
| N | Zero new Mutex / RwLock / CondVar | ✓ grep clean |
| O | Honest deltas surfaced | ✓ 3 categories |

**15/15 rows pass.** Mode A clean.

## Workspace state

- **Pre-1f-β-ii baseline:** 1339 passed / 859 failed (post-1f-β-i V2)
- **Post-1f-β-ii:** 1339 passed / 864 failed
- **Delta:** +0 passed / +5 failed — the 5 new stdout hermetic tests architecturally blocked by the same § Row K (deftest-hermetic → run-sandboxed-hermetic-ast phase-B migration). No regression of any pre-existing test.

The blocker continues to be tracked for closure adjacent to slice 1f-δ's INSCRIPTION; this is expected behavior for slices 1f-β-i/ii/iii.

## Honest deltas surfaced

1. **HashMap → Vector routing-state** (anticipated, mirrors slice 1f-β-i): `Routing` typealias documents HashMap intent; driver carries `RoutingVec` for stable select-index ordering. Documented inline at file header and in typealias comments. **Classification:** design adapter; same rationale as slice 1f-β-i.

2. **ack-tx zero-payload send** (anticipated): `(:wat::kernel::send ack-tx ())` is the correct shape for `Sender<wat::core::nil>`; unit literal `()` is the nil value. **Classification:** no friction; BRIEF prediction held.

3. **Dispatch Write arm on data channel** (new for this slice): The stdin.wat pattern dispatched all data-channel events as a single `Read` case (since data-rx only ever carried `Event::Read`). For stdout, data-rx carries a full `Event` enum, so the dispatch arm needed an explicit `match event` on the data-channel path to extract the `Write` payload. Sonnet added two "unexpected" arms (Add/Remove on data channel) that ignore and recurse — making the pattern explicit and complete rather than partial. **Classification:** defensive completeness; appropriate.

## Implementation choices (locked)

- **Routing state shape:** `Vector<RoutingEntry>` (mirrors slice 1f-β-i)
- **Event shape:** verbatim per BRIEF + Rust mirror in `src/thread_io.rs`
- **Data-channel dispatch:** explicit `match event` with three arms (Write handled; Add/Remove are no-ops on data channel)
- **Unit-send idiom:** `(:wat::kernel::send ack-tx ())`
- **Test harness:** `:wat::test::deftest-hermetic` (architecturally blocked per § Row K from slice 1f-β-i)

## Files modified

- `wat/kernel/services/stdout.wat` (new, 325 lines) — wat-side StdOutService
- `wat-tests/kernel/services/stdout.wat` (new, 339 lines) — 5 hermetic test rows
- `src/stdlib.rs` (+7 lines) — registration entry after stdin.wat entry

## Lessons captured

1. **Pattern apply post-mint is sub-15-min sonnet work.** Slice 1f-β-i minted the pattern at ~25 min; slice 1f-β-ii applied it at ~8 min (3× speedup). Calibration: future BRIEFs in this family should predict 10-15 min sonnet.

2. **The data-channel dispatch shape difference (Read-only vs full Event) is the only structural delta.** Sonnet handled it correctly with defensive completeness (Add/Remove no-op arms on data channel). Future BRIEFs in this family should call out the data-channel-event-shape decision explicitly so sonnet doesn't have to surface it as a delta.

3. **Workspace blocker accounting is predictable.** Each new hermetic test ships +1 failure against the § Row K baseline. Slice 1f-β-iii will ship +5 more (total 869 failed), all clearing when `deftest-hermetic` migrates to `spawn-process` adjacent to slice 1f-δ.

## What's next

1. **Atomic-commit slice 1f-β-ii** (this turn) — bundle the 3 modified files + this SCORE doc
2. **Author slice 1f-β-iii BRIEF** — `wat/kernel/services/stderr.wat` StdErrService (final service mint; pattern-apply mirror of StdOut with identical shape modulo fd-2 routing)
3. **Spawn slice 1f-β-iii** (sonnet, mechanical pattern-apply; predicted 10-15 min)

## Cross-references

- BRIEF: [`BRIEF-SLICE-1F-B-II.md`](./BRIEF-SLICE-1F-B-II.md)
- Pattern source: `wat/kernel/services/stdin.wat` (slice 1f-β-i, committed `e898c7a`)
- Predecessor SCORE: [`SCORE-SLICE-1F-B-I-V2.md`](./SCORE-SLICE-1F-B-I-V2.md)
- Successor: slice 1f-β-iii (StdErrService — final wat-side service mint)
- Architecture: REALIZATIONS pass 18 (unified Event protocol); TIERS.md § OS-boundary handling
- § Row K (architectural blocker): tracked for closure adjacent to slice 1f-δ INSCRIPTION
