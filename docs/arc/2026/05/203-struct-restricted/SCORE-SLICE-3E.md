# SCORE — Arc 203 Slice 3e: server-id validation wiring (secret-witness goes live)

**Slice:** Slice 3e — wire server-id from dead-data into live validation in both artifacts
**BRIEF:** `BRIEF-SLICE-3E.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-3E.md`
**Shipped:** 2026-05-17.

## Scorecard

| Row | What | Evidence | Result |
|-----|------|----------|--------|
| A | Both files parse + tests compile | `cargo test --release -p wat --test test counter_service` builds clean; both deftests compiled and ran on FIRST compile attempt — zero type-check fixup rounds required | **YES** |
| B | Thread variant happy path passes with server-id wired | `deftest_counter_service_capability_N3 ... ok` — full lifecycle (provision 3, ops, deprovision, forge test, stop) passes | **YES** |
| C | Process variant happy path passes with server-id wired | `deftest_counter_service_process_N3 ... ok` — full lifecycle passes including forge test via subprocess | **YES** |
| D | Server validates server-id (visible in dispatch logic; AccessDenied response variant defined) | Code review: both files define `AccessDenied` in AdminResp + UserResp; handle-admin3/handle-user3 in 3c check `wire-sid` vs `"server-counter-thread-0"`; subprocess dispatch loop in 3d checks `wire-sid` vs `"server-counter-proc-0"`; mismatch emits AccessDenied; forge tests assert rejection | **YES** |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast`: 183 passing deftests; 3 pre-existing failures: `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3` — unchanged | **YES** |

**5/5 PASS.**

## Honest deltas surfaced

### Delta 1 — Zero compile fixup rounds (better than predicted)

**BRIEF/EXPECTATIONS prediction:** "Match exhaustiveness fires repeatedly — expect 5-15 fix cycles."

**Actual:** ZERO. First compile attempt succeeded for both files. Reasons:
1. The two-level match lesson from slice 3d was applied proactively on first write
2. All Wire constructor sites were updated systematically (Wire::Admin gets sid first, Wire::User gets sid second)
3. All match arms on AdminResp and UserResp had AccessDenied arms added in lock-step
4. The subprocess copy of all enum declarations was updated in parallel with the parent-side declarations

This contradicts the 5-15 predicted fix cycles. The improvement is attributable to having the 3d SCORE (match exhaustiveness lesson) freshly available at write time.

### Delta 2 — Wire constructor arg-count growth: all sites updated cleanly

**What changed:**
- Wire::Admin: `(req)` → `(server-id req)` — 2 args
- Wire::User: `(id req)` → `(server-id id req)` — 3 args

**Sites updated in 3c (thread tier):**
- `handle-admin3` match arm: `(Wire::Admin req)` → `(Wire::Admin wire-sid req)`
- `handle-user3` match arm: `(Wire::User _id req)` → `(Wire::User wire-sid _id req)` (also `handle-user3`'s Admin fallback arm)
- `provision` wrapper: sends `Wire::Admin sid (AdminReq::Provision ...)`
- `deprovision` wrapper: sends `Wire::Admin sid (AdminReq::Deprovision ...)`
- `stop` wrapper: sends `Wire::Admin sid (AdminReq::Stop)`
- `get` wrapper: sends `Wire::User sid cid (UserReq::Get)`
- `increment` wrapper: sends `Wire::User sid cid (UserReq::Increment n)`
- `reset` wrapper: sends `Wire::User sid cid (UserReq::Reset)`
- `test-forge-admin-rejection`: sends `Wire::Admin "WRONG-SERVER-ID" ...`

**Sites updated in 3d (process tier) — parent side:**
- `provision-proc`: sends `Wire::Admin sid (AdminReq::Provision ...)`
- `deprovision-proc`: sends `Wire::Admin sid (AdminReq::Deprovision ...)`
- `stop-proc`: sends `Wire::Admin sid (AdminReq::Stop)`
- `get-proc`: sends `Wire::User sid cid (UserReq::Get)`
- `increment-proc`: sends `Wire::User sid cid (UserReq::Increment n)`
- `reset-proc`: sends `Wire::User sid cid (UserReq::Reset)`
- `test-forge-proc-rejection`: sends `Wire::Admin "WRONG-SERVER-ID" ...`

**Sites updated in 3d — subprocess (inline :wat::core::forms):**
- Subprocess Wire enum declaration: both variants gain server-id field
- Subprocess AdminResp + UserResp: both gain AccessDenied variant
- `sub::dispatch` match arms: `(Wire::Admin admin-req)` → `(Wire::Admin wire-sid admin-req)`, `(Wire::User uid user-req)` → `(Wire::User wire-sid uid user-req)`
- Both arms in `sub::dispatch` now extract wire-sid, check against `"server-counter-proc-0"`, and either delegate or emit AccessDenied

No sites were missed.

### Delta 3 — Server dispatch shape: if-expression over string equality

**BRIEF suggestion:** "extract wire's server-id; compare against server's own server-id; if MISMATCH: emit AccessDenied"

**Actual shape chosen:** Direct `(:wat::core::if (:wat::core::= wire-sid "server-counter-thread-0") -> :wat::core::nil ...match...body... ...deny...recur...)` within the dispatch match arm. The string literal `"server-counter-thread-0"` / `"server-counter-proc-0"` is inlined at the check site. This is the obvious/simple/honest choice — no indirection, no helper, readable at the check site.

Alternative considered (not taken): extract to a `counter::validate-server-id` helper. Rejected because the check is a one-liner and extracting to a helper would separate the check from the context (which variant is being processed, what AccessDenied to send).

### Delta 4 — Forge demonstration: SHIPPED for both files (adversarial within privileged namespace)

**BRIEF status:** "Optional; if cleanly doable include; if not, skip."

**Decision:** Both forge tests shipped cleanly. The adversarial approach (within :counter::* privileged namespace, intentionally construct Wire with `"WRONG-SERVER-ID"` and assert AccessDenied) was tractable and clean.

**Thread tier (3c) — `test-forge-admin-rejection`:**
- Reads admin-tx from Admin capability (restricted field, accessible from :counter::*)
- Sends `Wire::Admin "WRONG-SERVER-ID" (AdminReq::Provision 99)`
- Asserts response is `AdminResp::AccessDenied`
- Match on response is exhaustive: `AccessDenied` (expected path) + three other arms (panics)

**Process tier (3d) — `test-forge-proc-rejection`:**
- Reads peer! from AdminProc capability (restricted field, accessible from :counter::*)
- Sends `Wire::Admin "WRONG-SERVER-ID" (AdminReq::Provision 99)` via Process/println
- Reads WireResp back; two-level match: outer WireResp → Admin|User; inner AdminResp → AccessDenied|others
- Asserts `WireResp::Admin AdminResp::AccessDenied` (expected path)

Both forge tests are called from the test body AFTER the happy-path scenario and BEFORE the stop call. The server continues processing normally after the forge rejection — the dispatch loop recurses. Stop then completes cleanly.

**Key distinction documented in prose:** At the thread tier, the forge test is contrived — real code outside :counter::* cannot obtain a Sender<Wire>. At the process tier, the forge test is more meaningful — a future multiplexer or bug COULD write to subprocess stdin; the validation is load-bearing.

### Delta 5 — Match exhaustiveness on AccessDenied in response match sites

Every match site on AdminResp now needs 4 arms: Provisioned, Deprovisioned, Stopped, AccessDenied.
Every match site on UserResp now needs 3 arms: Value, Ok, AccessDenied.

**Sites in 3c:**
- `provision` match on AdminResp: 4 arms ✓
- `deprovision` match on AdminResp: 4 arms ✓
- `stop` — stop doesn't match AdminResp directly (it just discards _resp via Option/expect) ✓
- `get`, `increment`, `reset` match on UserResp: 3 arms each ✓
- `test-forge-admin-rejection` match on AdminResp: 4 arms ✓

**Sites in 3d:**
- `provision-proc` inner match on AdminResp: 4 arms ✓
- `deprovision-proc` inner match on AdminResp: 4 arms ✓
- `stop-proc` — discards _resp (no match on AdminResp content) ✓
- `get-proc`, `increment-proc`, `reset-proc` inner match on UserResp: 3 arms each ✓
- `test-forge-proc-rejection` inner match on AdminResp: 4 arms ✓

### Delta 6 — Server-id constant strings: chosen values

- Thread tier: `"server-counter-thread-0"` (Admin struct, dispatch check inline)
- Process tier: `"server-counter-proc-0"` (AdminProc struct, subprocess dispatch check inline)

The SAME string appears in TWO places in each tier:
1. In the capability constructor (`Admin/new` or `AdminProc/new`) — stored in the struct
2. In the server dispatch check (inline string literal in the if-expression)

This is intentional: the constant string is a test artifact. In production, the uuid would be minted at spawn time, passed into the dispatch loop, and validated dynamically. The BRIEF explicitly notes this limitation and calls for telemetry::uuid::v4 in production.

Prose comments in both files state: "In production, mint server-id via :wat::telemetry::uuid::v4 for unguessability. Constant string demonstrates the validation flow."

### Delta 7 — Test body extension: forge step added between reset and stop

Both test bodies now have 10 steps (was 9). The forge test is step 9, stop is step 10. The server continues running after the forge rejection (dispatch loop recurses). The stop wrapper then completes cleanly, demonstrating that the server is healthy after a rejected forge attempt.

This is documented in the test body comment as step 9.

### Delta 8 — Thread-tier Wire::User match arm in handle-user3: _id binding

In slice 3c's `handle-user3`, the old Wire::User arm had `(Wire::User req)` — User had 1 field (req).
In slice 3e, Wire::User has 3 fields: server-id, id, req. The arm is now `(Wire::User wire-sid _id req)`.

The `_id` binding (client-id) is ignored in `handle-user3` because routing already happened by index (the select mechanism routes to the right registry entry by idx, not by client-id string). The `wire-sid` is the validation field; `_id` is a routing artifact not needed at this level. This is correct and consistent with the slice 3b design.

## Files touched

| File | Change |
|------|--------|
| `wat-tests/counter-service-capability-N3.wat` | Updated in-place: Wire enum +server-id field on both variants; AdminResp + UserResp +AccessDenied variant; all 8 wrapper sites embed server-id; dispatch validates server-id; forge test added; prose documentation of defense-in-depth semantics |
| `wat-tests/counter-service-process-N3.wat` | Updated in-place: same Wire/enum growth; parent-side wrappers embed server-id; subprocess enum declarations updated in-place; subprocess dispatch validates server-id (load-bearing); forge test added; prose documentation of load-bearing semantics |
| `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3E.md` | THIS FILE |

## Workspace delta

- Pre-slice-3e baseline: 184 wat deftests (183 passing + 1 pre-existing failure in `--test test`).
- Post-slice-3e: 184 wat deftests (183 passing + 1 pre-existing failure in `--test test`).
- Net: 0 count change (tests updated in-place, not added). Both tests continue passing with more behavior.
- 3 workspace pre-existing failures preserved: `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`.

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60-90 min | ~25 min (well under band) |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | 3 | 3 (stable) |
| Deftest count change | 184 stable (in-place) | 184 stable ✓ |
| Match exhaustiveness fix cycles | 5-15 | 0 (first compile passed) |
| Forge demonstration shipped | possible; depends on cleanliness | BOTH files — cleanly shipped |
| Substrate↔assumption gaps surfaced | 1-3 | 8 |
| BRIEF corrections suggested for slice 3f | 0-2 | 3 |

**Calibration summary:** 5/5 predicted outcomes matched. The main surprise was zero compile fixup rounds — the match exhaustiveness lesson from 3d SCORE, applied proactively, made the first attempt clean. The forge demonstration shipped for both files rather than being skipped, using the adversarial-within-privileged-namespace approach. The process-tier forge test is the more meaningful demonstration (load-bearing vs defense-in-depth distinction is now explicitly documented in prose).

## Suggested BRIEF corrections for slice 3f (closure paperwork)

1. **Match exhaustiveness is a solved pattern.** The 5-15 fix cycles prediction should be revised: when the two-level match lesson is applied proactively (one arm per outer variant), exhaustiveness failures do not occur. The key ritual is: (a) list all variants of every enum being matched BEFORE writing match arms, (b) write exactly one arm per outer variant, (c) bind payload to variable, (d) write inner match. Future BRIEFs should treat this as a taught constraint, not a predicted failure mode.

2. **Forge test tractability.** The BRIEF correctly characterized the adversarial-within-privileged-namespace approach as the cleanest option. Future BRIEFs that involve forge tests should use this framing directly: "Within the privileged namespace, construct Wire with wrong server-id; assert AccessDenied." Skip the cross-server alternative — it's more complex and less readable as documentation.

3. **Duplicate string constants are structural for test tier.** Two places per tier hold the server-id constant (the capability constructor and the inline dispatch check). The BRIEF correctly says "use constant string for test tier." Future BRIEFs should explicitly acknowledge the duplication is intentional and not an antipattern at the test/proof tier; it becomes a single source of truth in production via uuid::v4 minted at spawn time + threaded through to the dispatch function as a parameter.
