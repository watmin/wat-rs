# Arc 138 F-NAMES-1e — SCORE

**Written:** 2026-05-03 AFTER sonnet's report + orchestrator spot-check.
**Agent ID:** `aab66afa6a449fe34`
**Runtime:** ~5 min (285 s).

## Verification

| Claim | Disk-verified |
|---|---|
| Files modified | 2 (src/spawn.rs, src/runtime.rs) ✓ |
| diff stat 67+/47- | ✓ |
| 3 spawn sites updated with Builder::new().name(...) | ✓ |
| All 7 arc138 canaries | 7/7 PASS ✓ |
| Workspace tests | empty FAILED ✓ |
| **`<unnamed>` panics across workspace** | **ZERO** ✓ |

## Hard scorecard: 5/5 PASS.

## Spot-check (the moment of truth)

`RUST_BACKTRACE=1 cargo test --release --workspace 2>&1 | grep "thread '"` post-F-NAMES-1e:

```
thread 'runtime::tests::call_stack_populates_on_assertion' panicked at wat-rs/src/runtime.rs:15220:55:
thread 'wat-thread:::wat::kernel::spawn-program-ast' panicked at src/runtime.rs:15130:10:19:
thread 'wat-thread:::wat::kernel::spawn-program-ast' panicked at /home/watmin/work/holon/wat-rs/wat-tests/core/option-expect.wat:77:19:
thread 'wat-thread:::wat::kernel::spawn-program-ast' panicked at /home/watmin/work/holon/wat-rs/wat-tests/test.wat:81:13:
...
```

**Zero `<unnamed>` entries.** Every panic header carries a real thread identity. Every coordinate is navigable.

## Substrate observations

1. **3 derivations chosen by sonnet:**
   - src/spawn.rs:183 — uses `op` (the spawn primitive name like `:wat::kernel::spawn-program-ast`)
   - src/runtime.rs:12421 — uses body lambda's keyword name OR `body_fn.name` field, with fallback
   - src/runtime.rs:18780 — Rust unit test, hardcoded function-name string

2. **Cosmetic stutter — `wat-thread:::wat::kernel::...`** (double colon) because the primitive path itself starts with `:`. Honest and informative; could be tweaked later for visual cleanliness if desired.

3. **Sonnet caught a brief mislabel:** runtime.rs:18780 was tagged "service spawn worker" but is actually a Rust unit test thread spawn. Sonnet corrected the framing in its report.

## Calibration

Predicted 10-15 min; actual 5 min. F-NAMES-1c established the Builder pattern; F-NAMES-1e was straightforward replication across 3 sites.

## Ship decision

**SHIP.** F-NAMES campaign complete for the wat::test! / wat-spawn paths. Every panic in the substrate now navigates to a real Rust thread name + a real file:line:col coordinate.

## Arc 138 status post-F-NAMES-1e

**All known cracks closed.** The user's original UX observation:
```
thread '<unnamed>' panicked at <test>:10:19:
```
is now:
```
thread 'wat-thread::<primitive>' panicked at /path/to/real-file.wat:10:19:
```

Both halves navigable. Coordinates open in editors. Thread names disambiguate which sub-thread panicked.

## Next per NAMES-AUDIT

- F-NAMES-2: `<lambda>` audit (define-bound lambdas losing names)
- F-NAMES-3: `<runtime>` invariant check (assert no user-visible `<runtime>:` rendering)
- F-NAMES-4: `<entry>` investigation (freeze.rs:421)
- Slice 6: doctrine + INSCRIPTION + USER-GUIDE + 058 row → arc 138 closure
