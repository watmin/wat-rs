# Arc 170 Stone B EXPECTATIONS

**BRIEF:** `BRIEF-STONE-B-WALKER-COLLAPSE.md`

## Independent prediction

**Runtime band:** 90-120 minutes sonnet.

Reasoning:
- New walker check is a small structural pattern-match (~30-50 LOC in `src/check.rs`)
- 4 new tests (~80-120 LOC)
- Caller sweep: existing usage of `*_join-result` is mostly in substrate namespace already (per design); user-namespace migrations probably 3-10 sites
- The hardest part is finding the right hook point in check.rs's existing walker traversal — sonnet may need to read significant context first

**Time-box:** 180 min hard stop.

## SCORE methodology

6 rows YES/NO per BRIEF; per-row evidence patterns:

- **Row A** (new walker check): `grep -nA 30 "check_join_result\|join.result.*namespace" src/check.rs` shows the new fn.
- **Row B** (walker hooked): grep shows the call point in the main check fn / traversal.
- **Row C** (tests pass): `cargo test --release -p wat --test wat_arc170_stone_b_walker_collapse` → 4 passed.
- **Row D** (caller migration): `grep -rn "Thread/join-result\|Process/join-result" wat-tests/ tests/` shows zero user-namespace direct calls (or fully accounted for via SCORE notes).
- **Row E** (build clean): cargo build Finished.
- **Row F** (workspace baseline maintained): cargo test summed failed ≤ 4 (Stone A baseline).

## Honest deltas to watch for

- **Walker hook point may not be obvious.** The arc 117/133 walker fires during `infer_let` post-processing. The new check might need to fire during fn-body traversal or call-site visiting. Sonnet may discover the exact hook is somewhere unexpected — surface in SCORE.

- **Namespace classification — exact mechanism.** The simplest check: enclosing def's full FQDN starts with `:wat::`. But what if a wat fn is defined at top-level WITHOUT a namespace prefix? Per arc 109 the substrate is FQDN-first, so this should be rare; verify.

- **Error message format.** Existing arc 117/133 errors have specific structure (named verb + caller location + suggested fix + reference to convention doc). The new error should match that voice — teaching, not punishing. The phrase "use the bracket (run-threads / run-processes)" should land WHEN those exist (Stones D/E); for now, point to `*_drain-and-join`.

- **Existing test fixture sweep.** Many tests embed wat source — sonnet should grep both `tests/wat_*.rs` (Rust files) AND `wat-tests/` (wat files) for occurrences. The migration is uniform per call site.

- **Substrate-namespace exemption.** `:wat::kernel::*`, `:wat::test::*`, `:wat::std::*` all need to be allowed (collectively `:wat::*` prefix check). Verify the check handles all three.

- **Pre-existing test failures.** 4 pre-existing failures from Stone A baseline are unrelated to this stone. NEW failures from this stone should be zero unless a migration is incomplete.

## Workspace baseline (commit `2a198bd`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 pre-existing target failures (lifeline flake, t6 unquote, totally_bogus, startup_error)

Post-Stone-B target:
- ≥ baseline + 4 passed (4 new Stone B tests)
- ≤ baseline failed (no regressions; existing user-namespace `*_join-result` calls all migrate to `*_drain-and-join`)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 90-120 min | TBD |
| Scorecard rows | 6/6 PASS | TBD |
| Workspace fail count | ≤ baseline (4) | TBD |
| New test count | 4+ | TBD |
| User-namespace callers migrated | 3-10 | TBD |
| Substrate-discovery surprises | 0-3 | TBD |
| Mode | Additive walker rule + caller migration sweep | TBD |
