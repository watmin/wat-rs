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

## Disposition — DEFER → arc 170 (user direction 2026-05-27)

- **NOT arc 240 scope** (consumer-`.wat` drift, which is complete). The
  wat-telemetry-sqlite log sink is a **daemon** (the auto-spawned `Service`,
  arc 089/095), and **arc 170 is actively reworking the whole spawn / Service /
  process-management layer it lives in.** Per the in-flight-dependency rule
  ("broken in code an open arc is actively building → defer + mark"), this is
  arc 170's to correct as part of that daemon rework. User: *"handle it in 170
  since it's reworking all the async process management stuff — this is a daemon
  to go correct."* Marked on `docs/arc/2026/05/170-program-entry-points/KNOWN-BROKEN.md`.
- **Aim the fix true:** the specific defect is decode-side —
  `decode_notag_holon` (cursor.rs) EDN-rejects `::`-namespaced keywords on
  read-back. When 170 corrects the daemon, fix that decode path. **If 170's
  rework does NOT touch the row-decode logic, re-home to arc 219b** (wat-edn EDN
  spec conformance, #445) — the `::`-keyword parse rejection may be a wat-edn
  parser conformance gap rather than a daemon concern.
- The 6 `reader` tests in `wat-telemetry-sqlite` are **known-red, documented**
  (here + arc 170 KNOWN-BROKEN). reader.wat's correct drift-fixes ARE committed
  (240.3b); Error 1 (cursor) + Error 2 (`:wat::test::assertion-failed` →
  `:wat::kernel::assertion-failed!`) are the remaining blockers.

Cross-ref: arc 240 DESIGN (root cause A); 240.3b SCORE; arc 230 (keyword→Bind);
arc 170 KNOWN-BROKEN; arc 219b (#445, EDN conformance — fallback owner).
