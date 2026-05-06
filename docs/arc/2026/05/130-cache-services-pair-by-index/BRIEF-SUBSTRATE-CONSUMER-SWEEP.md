# Arc 130 — Substrate consumer sweep BRIEF: `Vector/len` → `Vector/length`

**Drafted 2026-05-06.** Post-arc-146 substrate consumer cleanup
that was missed when arc 146 shipped its Dispatch entity for
`length` (2026-05-03). The original BRIEF-SLICE-1-RELAND.md for
the test-file rebuild is a SIBLING brief at a different scope;
this brief is the substrate consumer cleanup that unblocks the
reland's Layer 1 from currently failing.

## Context

The wat-lru and wat-holon-lru cache-service substrate files use a
primitive name `:wat::core::Vector/len` that **does not exist in
the wat-rs substrate**. The substrate's Vector-length primitive
is `:wat::core::Vector/length` (per arc 146's Dispatch entity
declaration at `wat/core.wat:13`). The 4 broken references are
the cascade's NEXT chain link after arc 143 closed the
`:wat::core::reduce` blocker — arc 143's INSCRIPTION explicitly
named this:

> *"The arc 130 RELAND v1 stepping stone now fails on a
> DIFFERENT primitive (`:wat::core::Vector/len`) — the `:reduce`
> blocker is closed; the cascade progressed to its next link."*

The arc 130 slice 1 RELAND test file (`crates/wat-lru/wat-tests/lru/CacheService.wat`)
shipped Layer 0 + Layer 1 stepping stones; Layer 1
(`:wat-lru::test-lru-raw-send-no-recv`) is the failing canary
that surfaces the broken substrate consumer. **The test file is
correct; the substrate is the bug.** This brief fixes the
substrate; the test file becomes green by consequence.

## Substrate evidence (verified pre-brief)

The post-arc-146 substrate registers `Vector/length` at:

- `wat/core.wat:13` — Dispatch arm `((:wat::core::Vector<T>) :wat::core::Vector/length)`
- `src/runtime.rs:2841` — `":wat::core::Vector/length" => eval_vector_length(args, env, sym)`
- `src/runtime.rs:5414, 5453` — `dispatch_substrate_impl` op references
- `src/runtime.rs:6092` — direct dispatch arm
- `src/check.rs:11532` — TypeScheme registration

`:wat::core::Vector/len` exists nowhere in the substrate (verified
by `grep -rn "Vector/len\b" wat/ src/`). It is purely a stale
consumer reference.

## The 4 sites to fix

| File | Line | Use case |
|---|---|---|
| `crates/wat-lru/wat/lru/CacheService.wat` | 219 | Get branch's hit/miss count |
| `crates/wat-lru/wat/lru/CacheService.wat` | 246 | Put branch's entry count |
| `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` | 257 | Get branch's hit/miss count |
| `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat` | 285 | Put branch's entry count |

All 4 sites have identical shape:
```scheme
((n :wat::core::i64) (:wat::core::Vector/len <var>))
```

The fix at each site is identical:
```scheme
((n :wat::core::i64) (:wat::core::Vector/length <var>))
```

## What to do

### Pre-flight crawl (mandatory; verify before editing)

Run these greps to confirm the brief's evidence is current:

```bash
grep -rn "Vector/len\b" crates/ wat/ src/    # should show exactly the 4 sites named
grep -rn "Vector/length\b" wat/ src/         # should show the substrate registers Vector/length
grep -n "Vector/length" wat/core.wat         # should show the Dispatch arm at line 13
```

If `Vector/length` does NOT register in the substrate, **STOP** —
the brief's substrate evidence is wrong. Surface this as a clean
diagnostic and do not edit.

If the 4 sites are not as named, **STOP** — the brief's site list
is wrong. Surface the discrepancy.

### The edits

Make 4 mechanical replacements, one per site listed above.
`:wat::core::Vector/len` → `:wat::core::Vector/length`. No other
edits to these files. Preserve all surrounding code, comments,
indentation.

### Verification

Run the workspace test suite:

```bash
cargo test --release --workspace 2>&1 | tail -20
```

EXPECTED OUTCOMES:

1. **The previously-failing test passes:**
   `deftest_wat_lru_test_lru_raw_send_no_recv` should turn from
   FAILED → ok.

2. **All other tests still pass at their previous state:**
   the 6 `:should-panic("channel-pair-deadlock")` tests in
   `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat`
   should remain in the same state (still :should-panic; arc 130
   slice 2 will retire those panics later).

3. **No tests that were previously passing break.**

If any of these expectations fail, **STOP** and surface the
discrepancy. Do NOT modify additional files to make tests pass.

## Constraints

- **Substrate-only edits.** ONLY 2 files modified:
  - `crates/wat-lru/wat/lru/CacheService.wat`
  - `crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat`
- **No test-file edits.** The test file at
  `crates/wat-lru/wat-tests/lru/CacheService.wat` has a stale
  diagnostic comment about `:wat::core::reduce` (now-stale
  because arc 143 closed that gap; the current failure is
  `Vector/len` which this brief closes). DO NOT refresh that
  comment — it's a separate sweep and the test file's CURRENT
  state is the reland's historical record.
- **No substrate edits beyond the rename.** No new primitives.
  No removed primitives. No reshape. Only the 4 mechanical
  renames.
- **No commits, no pushes.** Working tree stays modified for the
  orchestrator to score.
- **STOP at first red.** If verification surfaces unexpected
  failures, stop and report.
- **No grinding.** If the rename doesn't make the failing test
  pass, the brief's evidence is wrong; surface that, don't
  iterate.

## Out of scope

The following are NOT in this brief's scope (they are separate
work captured in arc 130's queue):

- Test file's stale diagnostic comment refresh
  (`crates/wat-lru/wat-tests/lru/CacheService.wat:56-58`)
- Continuing Layers 2-7 of the original BRIEF-SLICE-1-RELAND
  (test-file extension after Layer 1 turns green)
- Slice 2's HolonLRU full reshape (Handle/DriverPair typealiases,
  Reply<V> enum mirror — different scope; the 2 HologramCacheService
  Vector/len references are touched by THIS brief because the
  rename is uniform across both crates' substrate files)
- Retiring the 6 `:should-panic` annotations on HologramCacheService
  tests (slice 3 closure work)
- Arc 130 slice 1 RELAND scoring + INSCRIPTION

## Reporting

Target ~150-200 words:

1. **Pre-flight crawl results:** confirm the 4 sites match;
   confirm substrate has `Vector/length`.

2. **Edits made:** list of 4 sites + the rename.

3. **Verification:**
   - The previously-failing test's new status
   - Workspace test count: passed/failed/ignored before vs after
   - Confirm: no previously-passing test broke
   - Confirm: the 6 :should-panic tests on HologramCacheService
     remain in their prior state

4. **Path:** Mode A clean (rename worked + test green) / Mode B
   (rename worked but unexpected side effect) / Mode C (rename
   didn't fix the failure — substrate evidence was wrong).

5. **Honest deltas:** any surprise — additional substrate cleanups
   surfaced, etc.

## Why this is the obvious next move

The user's framing 2026-05-06: "if the path forward is obvious -
write the context for a fresh sonnet to go resolve - the protocol
is that we prove we reached mutual agreement by having sonnet
satisfy the technical resolution."

The path forward IS obvious:
- The substrate registers `Vector/length`, not `Vector/len`
- Four substrate consumer sites use the wrong name
- The mechanical rename is the resolution
- The failing test's name (`raw-send-no-recv`) is the canary that
  proves the rename works
- Sonnet executing this brief and shipping the rename + the test
  turning green = the proof of mutual agreement

If sonnet ships clean against this brief, the orchestrator's
understanding of the consumer-sweep gap aligns with the user's
framing. If sonnet hits Mode B or Mode C, the brief reveals
whether the orchestrator's understanding was incomplete OR the
substrate's state has more drift than this brief named.

Either outcome calibrates the cooperation.
