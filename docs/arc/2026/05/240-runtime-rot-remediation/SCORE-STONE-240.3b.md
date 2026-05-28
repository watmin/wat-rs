# SCORE — Stone 240.3b — consumer `.wat` drift sweep (telemetry + telemetry-sqlite)

## Test result lines (verbatim)

### wat-telemetry
```
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

### wat-telemetry-sqlite
```
test result: FAILED. 5 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.22s
```

### workspace build
```
cargo build --release --tests --workspace → 0 errors
```

---

## Per-file site counts

### `crates/wat-telemetry/wat/telemetry/WorkUnit.wat` (prod)
- 0 verb-call sites changed (no Atom/atom-value/HashMap calls)
- 1 stale comment updated (line 25–29: "Atom :requests" → "to-holon :requests"; arc 225 attribution)

### `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` (test)
- 1 stale comment updated (header, lines 15–18: "Atom :requests" → "to-holon :requests")
- Recipe 4 × 2 sites:
  - line 35: `(:wat::core::HashMap :wat::telemetry::Tag)` → `(:wat::core::HashMap :wat::holon::HolonAST :wat::holon::HolonAST)` (empty-tags define)
  - line 495: `(:wat::core::HashMap :wat::telemetry::Tag ...)` → `(:wat::core::HashMap :wat::holon::HolonAST :wat::holon::HolonAST ...)` (test-tags-roundtrip)
- Recipe 1 × 18 sites (keyword arg → to-holon):
  - line 39: `(:wat::holon::Atom :wat-telemetry::test::ns)` (default-ns define)
  - line 344: `:hits` (test-wu-recv-event-is-some)
  - line 384: `:never-incremented` (test-counter-default)
  - line 394: `:requests` (test-incr-then-counter)
  - line 405: `:requests` (test-incr-many)
  - line 418: `:sql-page` (test-append-dt-then-read)
  - line 442: `:sql-fetch` (test-timed-bumps-counter-records-duration)
  - line 460: `:work` (test-timed-twice-accumulates)
  - lines 490–493: `:asset`, `:BTC`, `:stage`, `:market-eval` (test-tags-roundtrip × 4)
  - line 520: `:hits` (test-scope-passes-result)
  - line 540: `:requests` (test-build-counter-metric)
  - line 566: `:sql-page` (test-build-duration-metric)
  - lines 626–627: `:sql-page` × 2 (test-collect-metrics-two-duration-samples)
  - line 663: `:my::function` (test-namespace-roundtrip)
  - line 715: `:hits` (test-make-scope-ships-counter)

### `crates/wat-telemetry-sqlite/wat-tests/telemetry/hashmap-field.wat`
- Recipe 4 × 1 site: `(:wat::core::HashMap :wat::telemetry::Tag)` → `(:wat::core::HashMap :wat::holon::HolonAST :wat::holon::HolonAST)`
- Recipe 1 × 4 sites: `:asset`, `:BTC`, `:stage`, `:market` (all in the assoc chain)

### `crates/wat-telemetry-sqlite/wat-tests/telemetry/edn-newtypes.wat`
- Recipe 1 × 1 site: `(:wat::holon::Atom "hello")` → `(:wat::holon::to-holon "hello")`

### `crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat`
- Recipe 4 × 1 site: `(:wat::core::HashMap :(wat::holon::HolonAST,wat::holon::HolonAST))` → `(:wat::core::HashMap :wat::holon::HolonAST :wat::holon::HolonAST)` (tuple-alias form expanded to 2 args)
- Recipe 3 × 1 site: `(:wat::core::atom-value h)` → `(:wat::holon::from-holon h)` (test-data-ast-extracts-holon)

---

## STOP — reader.wat failures are NOT recipe drifts

After applying recipes 3 + 4 to `reader.wat`, the check errors clear but 6 reader tests still fail at **runtime**. The errors are NOT recipe drifts; they originate in the Rust cursor and in a missing function registration:

### Error 1 — substrate runtime panic (cursor.rs:318)
```
wat-telemetry-sqlite::LogCursor: row reify failed: NoTag decode of namespace:
EDN parse error: EDN parse error at byte 28: invalid keyword: keyword begins with ::
```
All 6 reader tests hit this. The cursor panics when re-reading rows whose `namespace` column was written with a `::` namespaced keyword (e.g. `:test::reader`). This is NOT a verb-rename or arity drift — it is a read-path decode bug in `crates/wat-telemetry-sqlite/src/cursor.rs`. Editing `.wat` files cannot fix it; it requires changes to `src/cursor.rs` (out of scope for this stone).

**Pre-existing status:** Confirmed pre-existing. Before this stone's changes, all 6 reader tests failed at check time (type-check errors masked the runtime path). This stone's recipe-4 HashMap fix cleared the check-time block, making the underlying cursor decode bug newly visible at runtime.

### Error 2 — unknown function: `:wat::test::assertion-failed`
```
failure: /home/watmin/work/holon/wat-rs/crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat:235:11:
unknown function: :wat::test::assertion-failed
```
Used in `None` match arms of `test-data-ast-extracts-holon` (line 235) and `test-data-value-lifts-string` (line 270). The function `:wat::test::assertion-failed` is not registered in the substrate (only `:wat::kernel::assertion-failed!` exists). This is NOT a verb-rename drift — it is a missing function that needs either substrate registration or a call-site change to the correct `(:wat::kernel::assertion-failed! ...)` form. Downstream from Error 1 (the None branch is reached because the cursor returns an empty vec); but the function itself is genuinely absent regardless.

Both errors require orchestrator judgment before proceeding. Reader tests: 0/6 cleared, blocked on these two non-drift substrate issues.

---

## Summary

| Crate | Before | After | Delta |
|-------|--------|-------|-------|
| wat-telemetry | 22 failed | **0 failed** | ✓ DONE |
| wat-telemetry-sqlite | 8 failed | 6 failed | 2 cleared (hashmap-field, edn-newtypes); 6 blocked on STOP |
| workspace build errors | — | **0** | ✓ |

Files touched (`.wat` only, as required):
- `crates/wat-telemetry/wat/telemetry/WorkUnit.wat` — comment only
- `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` — 20 sites (recipe 1 × 18, recipe 4 × 2, comment)
- `crates/wat-telemetry-sqlite/wat-tests/telemetry/hashmap-field.wat` — 5 sites (recipe 1 × 4, recipe 4 × 1)
- `crates/wat-telemetry-sqlite/wat-tests/telemetry/edn-newtypes.wat` — 1 site (recipe 1)
- `crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat` — 2 sites (recipe 3 × 1, recipe 4 × 1)

No `src/*.rs`, no holon-rs, no lru/holon-lru touched.
