# FINDING (arc 240, 2026-05-27) — two pre-existing wat-telemetry-sqlite reader bugs, surfaced by the 240.3b drift sweep

Stone 240.3b cleared all consumer-`.wat` drift in wat-telemetry (36/0) and the
sqlite drift-checks. Clearing reader.wat's check block (recipe-4 HashMap arity)
**revealed two pre-existing bugs** that had been masked — reader.wat failed at
*check* time before, so it never reached runtime. **Neither is consumer-`.wat`
drift; both are out of arc 240's scope.** 6 of 8 `wat-telemetry-sqlite` tests
(all the `reader` tests) block on them.

## Error 1 (the blocker) — sqlite cursor cannot roundtrip `::`-namespaced keywords

`crates/wat-telemetry-sqlite/src/cursor.rs` — `reify_log_row` → `decode_notag_holon(&namespace, "namespace")` panics:

```
LogCursor: row reify failed: NoTag decode of namespace:
EDN parse error: invalid keyword: keyword begins with ::
```

A log row's `namespace` is written to SQLite as a NoTag-wrapped holon, then read
back + EDN-decoded. The decode rejects a keyword "beginning with `::`". **Likely
an arc-230 ripple:** arc 230 made keywords pure `Bind` compositions, changing
their EDN serialization; the cursor's `decode_notag_holon` decode-form was never
updated (reader.wat was dead behind the check-error, so the workspace `--lib`
metric never exercised this path). This is a **Rust bug in the sqlite cursor**
(crate arcs 091/093 closed) — needs a focused investigation stone: determine
whether the `::` is a write-side form bug or a decode-logic staleness, then fix
`decode_notag_holon` (and possibly the write side) so the keyword roundtrips.

## Error 2 (downstream) — reader.wat calls a non-existent test verb

`crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat:235,270` call
`:wat::test::assertion-failed` — which does not exist. Only
`:wat::kernel::assertion-failed!` is registered (`src/check.rs:15749`). A
mechanical drift (verb namespace/rename), BUT downstream of Error 1 (the None
branch is only reached because the cursor bug leaves the event vec empty), so
fixing the name alone will not green the test.

## Disposition

- **NOT arc 240 scope** (consumer-`.wat` drift, which is complete). Surfaced here;
  to be fixed in a focused stone — the substrate-as-teacher cascade continues
  (arc 239 → 240 → this).
- The 6 `reader` tests in `wat-telemetry-sqlite` are **known-red, documented**
  (this file). reader.wat's correct drift-fixes ARE committed (240.3b); the two
  bugs above are the remaining blockers.
- A fix stone (when taken) addresses Error 1 first (cursor), then Error 2 (the
  verb rename), and re-runs `cargo test -p wat-telemetry-sqlite` → 0 failed.

Cross-ref: arc 240 DESIGN (root cause A); 240.3b SCORE; arc 230 (keyword→Bind).
