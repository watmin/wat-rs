# Arc 126 Slice 2 — Sonnet Brief

**Goal:** convert the 6 deadlock-class tests from `:ignore` to
`:should-panic(expected = "channel-pair-deadlock")`. The
arc-126-slice-1 check now fires when these test bodies are
frozen; what was `:ignore`d (skip-from-execution) becomes
`:should-panic`d (run AND verify the panic substring).

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Read-in-order anchor docs

1. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/REALIZATIONS.md`
   — the failure-engineering discipline this arc is the worked
   example for. Sets the frame: artifacts teach.
2. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/SCORE-SLICE-1-RELAND.md`
   — slice 1's clean ship; the shipped check now fires on the
   deadlock pattern.
3. `docs/arc/2026/05/126-channel-pair-deadlock-prevention/DESIGN.md`
   § "Sequencing" — slice 2's role in the chain.
4. `docs/arc/2026/05/122-per-test-attributes/DESIGN.md` — the
   `:should-panic` annotation mechanism. Cargo's libtest
   matches by SUBSTRING.
5. `wat/test.wat` lines 416-440 — the wat-side
   `:wat::test::should-panic` no-op define and the proc-macro
   scanner's substring lock.

## What to change

The 6 deadlock-class test sites currently carry both
`:wat::test::ignore` and `:wat::test::time-limit "200ms"`
annotations. Convert each one as follows:

- **Replace** `(:wat::test::ignore "<reason>")` with
  `(:wat::test::should-panic "channel-pair-deadlock")`.
- **Keep** `(:wat::test::time-limit "200ms")` as a safety net.
  It guarantees a 200ms cap if the panic doesn't fire as
  expected (defense-in-depth; rare but cheap).

The substring **`channel-pair-deadlock`** is the load-bearing
contract slice 1 emitted (verified at `src/check.rs:401`).

### The 6 sites

| # | File | Test name |
|---|---|---|
| 1 | `crates/wat-lru/wat-tests/lru/CacheService.wat` | `test-cache-service-put-then-get-round-trip` |
| 2 | `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` | `test-step3-put-only` |
| 3 | same file | `test-step4-put-get-roundtrip` |
| 4 | same file | `test-step5-multi-client-via-constructor` |
| 5 | same file | `test-step6-lru-eviction-via-service` |
| 6 | `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` | `step_B_single_put` |

Find each via `grep -n "wat::test::ignore"` across `crates/`.
Update the comment block above each annotation pair to reflect
the new role: it's no longer "we know this hangs"; it's "we
EXPECT this to panic with the substring." Update the comments
honestly.

## Constraints

- Three files change: the ones in the table above. No other
  files. No Rust changes. No DESIGN/INSCRIPTION updates (those
  ship in slice 3).
- The substring **MUST** be the literal `channel-pair-deadlock`
  (lowercase, hyphenated, single identifier). Slice 1's Display
  impl emits this; libtest matches by substring.
- All other tests stay green.
- No commits, no pushes.
- Stop if you find any of the 6 sites missing or already in a
  different shape (i.e. someone pre-edited them); report what
  you found.

## What success looks like

After your changes:

```bash
cargo test --release --workspace 2>&1 | tail -20
```

Expected:
- exit=0
- 6 tests that were `... ignored` are now `... ok` (because the
  panic matched their `:should-panic` expected substring).
- One arc-122 mechanism test stays `... ignored` (it's an
  intentional `:ignore` that verifies the mechanism — different
  from the deadlock class).

Per-crate sanity:
```bash
cargo test --release -p wat-lru --test test 2>&1 | tail -15
cargo test --release -p wat-holon-lru --test test 2>&1 | tail -20
```

Both should show the previously-ignored tests now passing.

## Honest unknown — surface if it fires

The `:should-panic` mechanism matches cargo test's libtest
panic. The chain from arc 126's `ChannelPairDeadlock` Display →
inner freeze error → `run-sandboxed-hermetic-ast` Result::Err →
deftest TestResult::Failure → `run_single_deftest` panic message
crosses several layers. Each layer should preserve the substring,
but it has not been runtime-verified end-to-end.

**If the substring does NOT propagate** through the chain (e.g.
the runner wraps the inner error message in a way that loses
the substring, or the inner freeze error is rephrased
somewhere):
- The 6 tests will fail with `note: test panicked, but did not
  contain expected string ...`
- Stop, capture the actual panic message reported, and report
  back. The brief assumes substring propagation; if the assumption
  is wrong, this is a substrate gap to surface (not your job to
  fix in slice 2).

This is a calculated risk per failure-engineering: spawn the
work, treat the outcome as data. If the chain works, slice 2
ships clean. If it doesn't, sweep 2's report becomes the brief
for the substrate fix arc.

## Reporting back

Target ~150 words:

1. The 3 files modified, with the 6 site:line refs.
2. The exact final form of one annotation block (so the
   orchestrator can verify the conversion shape).
3. Per-crate test results (wat-lru: X passed Y failed Z ignored;
   wat-holon-lru: same shape).
4. Workspace test totals (passed / failed / ignored).
5. **If `:should-panic` matched**: confirm the chain works
   end-to-end. Note the runtime: how long did each previously-
   ignored test take to fail-with-expected-panic?
6. **If `:should-panic` did NOT match**: capture the exact panic
   message libtest reported as "did not contain expected string."
   This is the substrate gap surface for the next arc.

## What this brief is testing (meta)

Per `REALIZATIONS.md`, the user wrote: *"us delegating to sonnet
is proof our discipline is sound - i have taught you to teach
others."* This brief tests whether the artifacts (slice 1's
DESIGN + SCORE + arc 128 + arc 122 substrate) carry enough
teaching that you can convert annotations correctly with no
extra context.

The conversion is mechanical (~6 sites). The unknown is the
runtime chain. Both shapes — clean ship OR substrate-gap-
surfaced — produce useful data.

Begin by reading the read-in-order anchors. Then do the
conversion. Then run the verification commands. Then report.
