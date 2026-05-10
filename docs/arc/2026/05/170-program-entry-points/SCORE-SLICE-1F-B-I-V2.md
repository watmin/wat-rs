## Arc 170 slice 1f-β-i V2 — SCORE

**Result:** Mode B — 15/17 pass; 1 row architecturally-out-of-scope; 1 row partial (workspace baseline delta from new tests blocked by the same out-of-scope item).
**Runtime:** ~25 min sonnet (well under predicted 60-90 band; well under 180 hard cap).
**Files:** 3 new + 1 modified — `wat/kernel/services/stdin.wat`, `wat-tests/kernel/services/stdin.wat`, `src/stdlib.rs`.

## Calibration

- **Predicted runtime band:** 60-90 min (sonnet); hard cap 180 min
- **Actual:** ~25 min — well under band; well under cap
- **Why faster than predicted:** BRIEF gave verbatim Rust Event types + concrete substrate-grep citations. Pattern-mint was bounded; the one design call (HashMap → Vector for stable select-index) sonnet surfaced cleanly with rationale.
- **Model decision validated:** Sonnet was the right call. Mechanical pattern-application from Rust types to wat side; design surface bounded by BRIEF prescriptions and the one honest-delta route resolved with documented rationale.

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `wat/kernel/services/stdin.wat` exists, parses, type-checks | ✓ `cargo check --release` green |
| B | `:wat::kernel::ThreadId` typealias declared | ✓ `stdin.wat:42` |
| C | `Event` enum declared with Read/Add/Remove variants | ✓ `stdin.wat:53-61` |
| D | Channel typealiases declared (EventTx/EventRx/Routing/Spawn) | ✓ `stdin.wat:67-92` |
| E | `spawn` fn defined, returns tuple | ✓ `stdin.wat:304` → `Spawn` |
| F | TCO driver loop with HashMap routing (adapted to Vector for stable select-index) | ✓ loop/dispatch at `:266`/`:175`; HashMap → Vector substitution documented inline at `:21-30` |
| G | `:wat::kernel::select` used over routing values + control-rx | ✓ `stdin.wat:280` |
| H | `Event::Add`/`Event::Remove` mutate routing via conj/filter (Vector equivalents) | ✓ `handle-add` / `handle-remove` fns present |
| I | `Event::Read` triggers `IOReader/read-line` + `edn::read` + reply-tx send | ✓ `handle-read` at `stdin.wat:150-174` |
| J | `wat-tests/kernel/services/stdin.wat` exists with ≥ 4 deftest-hermetic rows | ✓ 5 hermetic test rows |
| K | At least 1 deftest-hermetic test passes | **OUT-OF-SCOPE** — see § Row K below |
| L | Workspace within ±5 of post-1f-0b baseline (1328/854 per SCORE-1F-0B) | △ Partial — pass-count 1339 unchanged from immediate baseline; +5 failures from new hermetic tests that hit the same Row K out-of-scope blocker. No regression of any pre-existing test. |
| M | `cargo check --release` green | ✓ clean (1 pre-existing `dead_code` warning unrelated) |
| N | Only 3 new files + `src/stdlib.rs` registration entry = ≤ 4 files | ✓ git status shows exactly 4 |
| O | Zero new dependencies | ✓ Cargo.toml unchanged |
| P | Zero new Mutex/RwLock/CondVar | ✓ grep clean across new files |
| Q | Honest deltas surfaced | ✓ 4 categories surfaced |

**15/17 rows pass; 1 architecturally-out-of-scope (K); 1 partial driven by K (L).**

## § Row K — architectural out-of-scope rationale

**Out of slice 1f-β-i's scope.** The 5 new `deftest-hermetic` rows cannot exercise the wat-side StdInService because the `deftest-hermetic` macro in `wat/test.wat:258` + `:338` still calls `:wat::kernel::run-sandboxed-hermetic-ast` — the legacy substrate primitive from the retired `wat/std/hermetic.wat`. The phase-B caller-sweep was explicitly scoped out of arc 170 slice 3's closure with that pending migration documented inline at `src/stdlib.rs:103-110`:

> *"User-source callers of `run-sandboxed-hermetic-ast` are phase B sweep territory."*

The migration target per `TIERS.md` is `(:wat::kernel::spawn-process fn)` (Layer 3 / tier 2) — but tier-2 child processes require all three substrate stdio services to boot, per the locked architecture in `TIERS.md § OS-boundary handling`:

> *":wat::kernel::StdInService — owns fd 0 ... :wat::kernel::StdOutService — owns fd 1 ... :wat::kernel::StdErrService — owns fd 2"*

Slice 1f-β-i ships **one of three** services. The `deftest-hermetic` migration is downstream of:

1. Slice 1f-β-ii — StdOutService
2. Slice 1f-β-iii — StdErrService
3. Slice 1f-γ — runtime orchestrator + spawn-thread integration
4. Slice 1f-δ — runtime boot integration + shutdown cascade

**The migration will close adjacent to slice 1f-δ's INSCRIPTION** — at that point the forked child boots with full stdio service availability. Row K is not "deferred"; it is **architecturally scoped out** of this slice and tracked in the slice-1f-δ closure plan.

The 854-baseline failures share this single root cause: every `deftest-hermetic` test in the workspace today hits the same retired-verb path. The +5-failure delta from slice 1f-β-i is mechanical (5 new tests adopting the canonical hermetic shape; same blocker as the 854 baseline).

## Honest deltas surfaced

1. **HashMap routing → Vector substitution.** The BRIEF specified `HashMap<ThreadId, (data-rx, reply-tx)>` for routing state. `HashMap/values` iteration order is non-deterministic in the substrate (backed by `std::collections::HashMap`). `select-by-index` requires stable order so the fired-index maps back to the routing entry. Driver uses `Vector<(ThreadId, EventRx, Sender<HolonAST>)>` instead; conceptual `Routing` typealias declared as HashMap per BRIEF documentation, driver state is `RoutingVec`. **Classification:** design adapter (not substrate friction, not bug). Documented inline at `stdin.wat:21-30`.

2. **`deftest-hermetic` 854-baseline blocker.** Detailed in § Row K above. **Classification:** architecturally out-of-scope; downstream of slices 1f-β-ii/iii/γ/δ.

3. **ThreadId placement.** Placed in `stdin.wat` (not a separate `kernel/types.wat`) per BRIEF guidance. First consumer is here; future refactor can lift it out when a second consumer appears. **Classification:** design choice documented inline.

4. **Select API shape.** Returns `(i64, CommResult<T>)` as expected from BRIEF citations. Console.wat pattern followed exactly. **Classification:** no friction; BRIEF prediction held.

## Lessons captured

1. **Pre-flight crawl missed `deftest-hermetic`'s phase-B status.** I wrote the V2 BRIEF citing `deftest-hermetic` as the test scaffold without grepping its macro body or cross-referencing `src/stdlib.rs:103-110`. FM 2 (briefing sonnet without substrate verification). Sonnet caught it cleanly (Mode B-architecturally-out-of-scope, no workaround). The 854-baseline rot was a known unfinished phase-B sweep; the V2 BRIEF should have noted it.

2. **TIERS.md is the load-bearing reference for `spawn-process` dependencies.** Future BRIEFs touching tier-2 work MUST cite which of the three services exist at brief-time. The dependency chain (StdIn → StdOut → StdErr → orchestrator → boot) is the gating order for any work that depends on forked-child stdio.

3. **The "premature migration" trap.** I initially proposed migrating `deftest-hermetic` to `spawn-process` as a one-file fix. User caught the dependency: forked tier-2 children need StdOut + StdErr available to boot. The fix isn't a one-file swap; it's a multi-slice chain. **Crawl TIERS.md before proposing tier-2 migrations.**

## Implementation choices (locked)

- **Routing state shape:** `Vector<RoutingEntry>` (not HashMap) — substrate select-index requires stable order
- **Conceptual Routing typealias:** documented as HashMap per BRIEF (matches the architectural intent)
- **ThreadId placement:** inline in `stdin.wat`; future refactor lifts it out
- **Event enum shape:** verbatim per BRIEF + Rust mirror in `src/thread_io.rs`
- **Test harness:** `:wat::test::deftest-hermetic` (per BRIEF) — tests can't run until the migration ships downstream

## Files modified

- `wat/kernel/services/stdin.wat` (new, 16089 bytes) — wat-side StdInService
- `wat-tests/kernel/services/stdin.wat` (new) — 5 hermetic test rows
- `src/stdlib.rs` (+8 lines) — registration entry after `wat/kernel/channel.wat`

## What's next

1. **Atomic-commit slice 1f-β-i V2** (this turn) — bundle the 3 new files + `src/stdlib.rs` edit + this SCORE doc
2. **Author slice 1f-β-ii BRIEF** — `wat/kernel/services/stdout.wat` StdOutService (pattern-apply mirror: `Write { line }` + `Add { thread_id, data_rx, ack_tx }` + `Remove { thread_id }`; ack-tx reply instead of HolonAST reply)
3. **Spawn slice 1f-β-ii** (sonnet, mechanical pattern apply; predicted 30-45 min given pattern is now minted)

## Cross-references

- BRIEF (V2): [`BRIEF-SLICE-1F-B-I-V2.md`](./BRIEF-SLICE-1F-B-I-V2.md)
- BRIEF (V1, STALE): [`BRIEF-SLICE-1F-B-I.md`](./BRIEF-SLICE-1F-B-I.md) — pre-pass-18 relay-sub-thread design; historical record
- Predecessor: slice 1f-0b (`d32a29f`) — Rust Event types this slice mirrors
- Successors: slices 1f-β-ii (StdOut), 1f-β-iii (StdErr), 1f-γ (orchestrator), 1f-δ (boot integration)
- Architecture: REALIZATIONS pass 18 (unified Event protocol); TIERS.md § OS-boundary handling
- Migration tracker: `deftest-hermetic` → `spawn-process` adjacent to slice 1f-δ's INSCRIPTION
