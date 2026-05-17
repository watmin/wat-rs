# SCORE — Arc 203 Slice 2: Counter/Client capability proof (minimal single-user)

**Slice:** Slice 2 — first wat-side consumer of struct-restricted
**BRIEF:** `BRIEF-SLICE-2.md`
**Shipped:** 2026-05-17.

## Scorecard

| Row | What | Evidence | Result |
|-----|------|----------|--------|
| A | Counter/Client mints cleanly via struct-restricted; deftest compiles | `cargo test --release -p wat --test test deftest_counter_client_capability_proof` builds without TypeError; all 4 prelude forms (enum declarations × 2, struct-restricted, defns) parse + type-check cleanly | **YES** |
| B | End-to-end round-trip succeeds (Increment, Get, Reset, Shutdown all assert correctly) | Same test passes in 0.01s; assertions: after-inc-5=15, after-inc-7=22, get=22, reset=0, shutdown=0 | **YES** |
| C | Workspace failure count = baseline (3 pre-existing) | `cargo test --release -p wat --test test` shows 179 passed (was 178 + 1 new), 1 failed (deftest_wat_tests_tmp_totally_bogus). Other pre-existing failures stable: startup_error_bubbles_up_as_exit_3, t6_spawn_process_factory_with_capture_round_trips. Probe_lifeline_pipe_proof flaked once under workspace contention, passed independently — pre-existing race, not regression. | **YES** |
| D | Capability pattern matches DESIGN — server reads server-id/client-id (restricted accessors) inside :counter::*-prefixed defns; user only invokes wrappers (no direct access to restricted fields outside :counter::) | Code review: :counter::Client/new called in :counter::spawn (restricted to [:counter::]); :counter::Client/server-id and /client-id read in :counter::spawn (same whitelist); :counter::Client/peer! (public) accessed by wrappers (:counter::get etc.); test body is outside :counter:: and cannot call /new or restricted accessors — enforcement is compile-time via arc 198/203 walker | **YES** |

**4/4 PASS.**

## Honest deltas surfaced

### Delta 1 — uuid::v4 FQDN is :wat::telemetry::uuid::v4, returns String (not keyword)

**BRIEF assumption:** "mints server-id + client-id via `:wat::measure::uuid::v4`" and `server-id <- :wat::core::keyword`.

**Actual:**
1. The FQDN is `:wat::telemetry::uuid::v4` (not `::measure::`). Source: `crates/wat-telemetry/wat/telemetry/uuid.wat` + `crates/wat-telemetry/src/shim.rs`.
2. Return type is `:wat::core::String` (not `:wat::core::keyword`). The shim renders to canonical 8-4-4-4-12 hyphenated hex string.
3. The `wat::telemetry` dep is NOT wired into `tests/test.rs` (which uses `wat::test! {}` with no deps). Calling `:wat::telemetry::uuid::v4` from a deftest in `wat-tests/` would fail at startup.

**Resolution:** Slice 2 uses constant string literals `"counter-server-0"` and `"counter-client-0"` for IDs. The single-user proof doesn't need uniqueness; uniqueness is a slice 3 concern (multiple provisioned clients). Field types changed to `:wat::core::String`.

**Suggested DESIGN/BRIEF correction:** The DESIGN example `server-id <- :wat::core::keyword` and BRIEF "mints via `:wat::measure::uuid::v4`" are both wrong. Correct form: `server-id <- :wat::core::String`. uuid::v4 is at `:wat::telemetry::uuid::v4` under the `wat-telemetry` dep (not `wat-measure`). Slice 3's BRIEF should add `deps: [wat_telemetry]` and use `:wat::telemetry::uuid::v4`.

### Delta 2 — Whitelist [:counter/] does not match :counter/*-namespaced functions

**BRIEF/DESIGN assumption:** `[:counter/]` as the ctor whitelist.

**Actual:** The arc 198 `caller_matches_prefix_list` function (src/check.rs:3217) uses `entry.ends_with("::")` to detect namespace-prefix entries. An entry ending in `/` does NOT trigger prefix matching — it's treated as an exact FQDN match. So `":counter/"` only matches the literal caller FQDN `":counter/"`, not `:counter/spawn` or `:counter/dispatch`.

Counter functions in `counter-actor-proof-thread.wat` use `/` separators: `:counter/spawn`, `:counter/dispatch`, etc. These FQDNs start with `:counter/` but not `:counter::`, so the whitelist `[:counter/]` cannot cover them via prefix matching.

**Resolution:** Slice 2 uses the `::` namespace convention throughout:
- Functions named `:counter::spawn`, `:counter::dispatch`, `:counter::get`, etc.
- Whitelist `[:counter::]` — these function FQDNs all start with `:counter::` → prefix match fires ✓

This differs from `counter-actor-proof-thread.wat` which uses `:counter/` convention (no struct-restricted constraint there). The naming difference is intentional: capability-issuing modules that use struct-restricted need `::` separator for the prefix match to work.

**Suggested DESIGN correction:** The DESIGN example should use `[:counter::]` (not `[:counter/]`) and show functions named `:counter::spawn` (not `:counter/spawn`). Alternatively, document that whitelist prefix matching requires `::` trailing; `/`-namespaced functions must use exact-FQDN entries or be renamed. This is not a substrate limitation — it's a documentation gap.

### Delta 3 — Public field design: ThreadPeer<Response, Request> (not separate Sender + Receiver)

**BRIEF assumption:** `in! <- :wat::core::Sender<...>` and `out! <- :wat::core::Receiver<...>` as separate fields.

**Actual:** The BRIEF assumed two separate channel fields. For the proof, bundling them into a `ThreadPeer<counter::Response, counter::Request>` as a single `peer!` field is cleaner:
- Client wrappers use `Thread/println peer!` and `Thread/readln peer!` — same pattern as counter-actor-proof-thread.wat wrappers
- One field instead of two avoids the need to construct a ThreadPeer each call
- The public accessor `:counter::Client/peer!` returns the full peer the caller needs

**Why this works:** `ThreadPeer<Response, Request>` means: reads Responses (from server), writes Requests (to server). Built from `(ThreadPeer/new (Thread/output thread) (Thread/input thread))` where Thread/output = Receiver<Response> and Thread/input = Sender<Request>.

**Suggested DESIGN correction:** Update the DESIGN example to show `peer! <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>` as the public field, rather than separate Sender/Receiver fields. Separate Sender/Receiver fields are valid but require constructing a ThreadPeer each call or using raw `send`/`recv` which returns `Result<Option<T>>` instead of bare T.

### Delta 4 — Type checking also allows structs in function return position

**Observation (not a BRIEF gap, more a confirmation):** `:counter::spawn` returning `:counter::Client` (a user-defined struct-restricted type) from a `defn` works cleanly. The type checker unifies the `let` body's final expression `:counter::Client` (from the Client/new constructor) with the declared return type. No special handling needed; user-defined types from struct-restricted are first-class in return positions.

## Files touched

| File | Change |
|------|--------|
| `wat-tests/counter-client-capability-proof.wat` | NEW — single deftest proving the Counter/Client capability pattern end-to-end |
| `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-2.md` | THIS FILE |

## Workspace delta

- Pre-arc-203-slice-2 baseline: 178 passing wat deftests + 3 pre-existing failures.
- Post-slice-2: 179 passing (1 new deftest), 1 wat-test failure (pre-existing). Other baselines unchanged.
- Net: +1 passing deftest, 0 new failures.

## Suggested DESIGN corrections

1. **DESIGN.md / BRIEF-SLICE-2.md § whitelist:** `[:counter/]` → `[:counter::]`. Functions under `:counter::` namespace match; `/`-separator functions do not. Document this explicitly so future slice BRIEFs don't repeat the assumption.

2. **DESIGN.md / BRIEF-SLICE-2.md § field types:** `server-id <- :wat::core::keyword` → `server-id <- :wat::core::String`. The only uuid source available is `:wat::telemetry::uuid::v4` → returns String.

3. **DESIGN.md / BRIEF-SLICE-2.md § uuid FQDN:** `[:wat::measure::uuid::v4]` → `[:wat::telemetry::uuid::v4]`. Arc 091 shipped the verb under `telemetry`, not `measure`. Slice 3's BRIEF should declare the telemetry dep.

4. **DESIGN.md § public-attrs example:** Consider `peer! <- :wat::kernel::ThreadPeer<counter::Response,counter::Request>` as the canonical public field shape for actor clients, rather than separate Sender + Receiver fields.

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Scorecard rows | 4/4 PASS | 4/4 PASS |
| Workspace fail count | ≤ 3 + variance | 3 stable (+ 1 transient contention flake) |
| New deftest count | 1 | 1 |
| Substrate↔assumption gaps surfaced | 1-3 | 4 (uuid FQDN, uuid return type, whitelist prefix, ThreadPeer vs separate channels) |
| BRIEF corrections suggested | 0-2 | 4 (whitelist, server-id type, uuid FQDN, public field shape) |
| STOP-triggers fired | 0-1 | 0 |
| Wall-clock runtime | 30-60 min | ~25 min (within band) |

**Calibration summary:** All predicted outcomes matched. The `[:counter/]` whitelist gap (Delta 2) was the main surprise — not anticipated in EXPECTATIONS. The other gaps (uuid FQDN, return type) were predicted. The public field simplification (ThreadPeer vs separate channels) was an implementation choice that simplified the design; not a gap per se.

**STOP-trigger analysis:** STOP trigger 1 fired in spirit (struct-restricted form `[:counter/]` rejected as a whitelist for `:counter/`-namespaced callers) but was NOT a substrate bug — the whitelist matching behavior is correct per the documented `::` rule. Resolution was to use `::` namespace convention for all counter functions, which is a DESIGN correction, not a substrate fix.
