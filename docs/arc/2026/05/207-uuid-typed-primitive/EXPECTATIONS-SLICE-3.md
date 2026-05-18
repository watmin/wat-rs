# EXPECTATIONS — Arc 207 Slice 3

## Mode prediction

**Mode A — clean retirement ships (~75%).** Sonnet greps consumers, finds the expected set (arc 206 tests + telemetry alias only), retires registrations + handlers + dispatch arms + telemetry alias body, deletes arc 206 test files, workspace baseline preserved. ~30-40 min.

**Mode B — telemetry alias has external consumer (~15%).** `grep ":wat::telemetry::uuid::v4" --include="*.wat"` surfaces a caller outside arc 203 demos. Could be:
- Lab repo (out of scope; surface as honest delta)
- An example wat program in `examples/`
- A wat-test in wat-telemetry's own test suite

Sonnet surfaces; orchestrator decides whether to extend slice 3 scope or defer to slice 4. Adds ~10 min.

**Mode C — arc 206 test files have entanglement (~7%).** Deleting `tests/wat_arc206_uuid_substrate.rs` or `_v5.rs` triggers compile errors in another test file referencing helpers from those files. Unlikely — they're standalone — but if so, sonnet refactors helpers into arc 207 test file or scope them out. Adds ~10 min.

**Mode D-time-violation — anything past 60 min.** Surface; orchestrator decides.

## Expected file changes

| File | Change |
|---|---|
| `src/check.rs` | Retire 2 type scheme registrations (~10-20 lines removed) |
| `src/string_ops.rs` | Retire `eval_uuid_v4` + `eval_uuid_v5` handlers (~30-60 lines removed); possibly retire `is_canonical_uuid` if unused |
| `src/runtime.rs` | Retire 2 dispatch arms (~4-8 lines removed) |
| `crates/wat-telemetry/wat/telemetry/uuid.wat` | One-line body change + comment update |
| `tests/wat_arc206_uuid_substrate.rs` | DELETED |
| `tests/wat_arc206_uuid_v5.rs` | DELETED |
| `docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-3.md` | NEW |

Expected diff: ~-150 lines (retirement net), +1 file (SCORE).

## Workspace baseline expected

Same 3-4 pre-existing failures (lifeline flaky may toggle). NO new failures.

The telemetry alias return-type change (`:String` → `:Uuid`) breaks any wat code that did `(let [(s :String) (:wat::telemetry::uuid::v4)] ...)` — the type checker rejects assigning `:Uuid` to a `:String` binding. The pre-existing telemetry test suite (36 tests) uses the alias internally but probably stores it generically — slice 2 already ran workspace tests with no regression, so telemetry tests handle the typed return. Expected: still 36/36 green.

## Out-of-scope findings (surface, don't act)

- Lab-side consumers of `:wat::core::uuid::v4` or `:wat::telemetry::uuid::v4` (out of arc 207 scope; lab reconstruction is the dependent unblock)
- `holon-rs` crate consumers (separate workspace; not arc 207 scope)
- Any USER-GUIDE prose still mentioning `:wat::core::uuid::v4` — slice 5 closure paperwork covers

## Failure-mode catches

- FM 1 (grep before claiming): sonnet's verification gate IS the grep audit
- FM 9 (load-bearing tests verified): cargo test on wat-telemetry + arc 207 IS the verification
- FM 11 (deferral language): N/A this slice
- FM 14 (surface retirement leaving internal identifiers): sonnet greps to confirm no leftover internal `uuid_v4` / `uuid_v5` identifier strings outside the typed family

## Atomic commit shape

NO commit by sonnet. Orchestrator commits all touched files + the 2 deletions + new SCORE atomically when sonnet returns.

## Calibration record

Slice 1 (audit) ran 36 min; slice 2 (substantive) ran ~93 min. Slice 3 (mechanical retirement) predicted 30-40 min; smaller surface, no new tests, well-bounded. If sonnet runs over 45 min, something surprising surfaced.

Sonnet: trust the substrate's grep; ship clean retirement; surface honestly; return.
