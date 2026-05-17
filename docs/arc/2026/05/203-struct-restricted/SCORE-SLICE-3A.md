# SCORE — Arc 203 Slice 3a: server dispatch loop (thread, N=1 user)

**Slice:** Slice 3a — server dispatch loop foundation, thread tier, hardcoded N=1 user
**BRIEF:** `BRIEF-SLICE-3A.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-3A.md`
**Shipped:** 2026-05-17.

## Scorecard

| Row | What | Evidence | Result |
|-----|------|----------|--------|
| A | Form parses + deftest compiles | `cargo test --release -p wat --test test deftest_counter_service_thread_N1` builds clean in 4.88s; no TypeErrors; all prelude forms (enum ×5, defn ×5) parse + type-check cleanly | **YES** |
| B | User round-trip succeeds (Increment 5→15, Increment 7→22, Get→22, Reset→0 all assert correctly) | Same test passes; assertions verified sequentially via user-increment, user-get, user-reset wrappers | **YES** |
| C | Admin Stop succeeds with Final state (Stopped(0) received) | Test passes; `admin-stop` sends Wire::Admin(AdminReq::Stop), receives AdminResp::Stopped(final=0); assert-eq 0 holds | **YES** |
| D | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast` shows exactly 3 failures: `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3` — identical to baseline; 180 passed in wat test suite (+1 new passing test) | **YES** |

**4/4 PASS.**

## Honest deltas surfaced

### Delta 1 — STOP 1 FIRED: heterogeneous select rejected; Wire enum pivot adopted (predicted)

**BRIEF assumption:** "Admin and user channels carry STRUCTURALLY-DIFFERENT enum types" — implied that `select` could operate on `[Receiver<AdminReq>, Receiver<UserReq>]`.

**Actual:** `:wat::kernel::select` is `∀T. Vec<Receiver<T>> → :(i64, Result<Option<T>, ThreadDiedError>)`. Uniform T is enforced at the type-check level (src/check.rs:13810 TypeScheme with single `T` type param). Heterogeneous receivers in the same select vec are type-checker-rejected.

**Resolution:** `:counter::Wire` unified enum adopted:
```scheme
(:wat::core::enum :counter::Wire
  (Admin (req :counter::AdminReq))
  (User  (req :counter::UserReq)))
```
Both admin and user clients send `Sender<Wire>`. Server selects on `Vec<Receiver<Wire>>` — uniform T. Server pattern-matches `Wire::Admin(req)` vs `Wire::User(req)` to route to admin or user handler. Protocol separation is behavioral (server routes by variant), not type-system-enforced (admin client CAN construct Wire::User variants; protocol discipline prevents it).

**Architecture consequence:** The spawn-thread auto-channels naturally become the admin channels:
- spawn-thread `[admin-wire-rx <- Receiver<Wire>, admin-resp-tx <- Sender<AdminResp>]`
- Thread/input(thread) = admin-tx (Sender<Wire>); Thread/output(thread) = admin-resp-rx (Receiver<AdminResp>)
- User channels created explicitly via make-bounded-channel; closed over in spawn-thread lambda
- select set: `[admin-wire-rx, user-wire-rx]` — both Receiver<Wire> ✓

**Note for stones 3b-3d:** The Wire enum is already the correct shape for dynamic provisioning (3b) — each provisioned client gets a `Sender<Wire>` and a `Receiver<UserResp>`. The server selects on a Vec of Wire receivers. No architecture change needed at 3b; only the registry logic changes. Stone 3d (process variant) will need a separate Wire encoding on the stdio stream.

**Suggested BRIEF correction:** BRIEF § "Admin and user channels carry STRUCTURALLY-DIFFERENT enum types (admin can't send UserReq; user can't send AdminReq — type system enforces)" — this is WRONG. The type system enforces nothing; protocol enforces. Correct framing: "Admin and user clients are logically separated by Wire variant; the server routes by match on Wire. Protocol enforces per-client discipline; type system does not." (Consistent with arc 198/203's "behavior enforces, not type system" lesson.)

### Delta 2 — `:(...)` inline tuple annotation must be on ONE line (no whitespace inside)

**BRIEF assumption:** Multi-line return type annotation `-> :(A, B, C)` implied possible.

**Actual:** The lexer rejects whitespace inside `:(...)` keywords (WAT-CHEATSHEET.md § 2: "NO whitespace inside `<...>`, `:(...)`, `:fn(...)`, or `:[...]`"). The initial spawn defn had a multi-line `-> :(T1, T2, T3)` annotation that triggered `lex error: whitespace inside unclosed bracket in keyword at byte 10488`. Fixed by putting the entire inline tuple on one line.

**Suggested BRIEF correction:** BRIEF spawn signature example should show the return type on one line. This gap is documented in WAT-CHEATSHEET.md § 2 but not applied consistently in BRIEF examples.

### Delta 3 — `Thread/join-result` is def-restricted to `[:wat::]`; use `Thread/drain-and-join`

**BRIEF assumption:** "Join thread" — `Thread/join-result` implied.

**Actual:** `Thread/join-result` is a def-restricted binding (arc 170 Stone B) accessible only from `[:wat::]` namespace. Test body is `:counter-service::thread-N1`, which doesn't match. Use `Thread/drain-and-join` (a TypeScheme registration, not def-restricted). This is consistent with counter-actor-proof-thread.wat which already uses drain-and-join.

**Suggested BRIEF correction:** BRIEF should say "join via Thread/drain-and-join" explicitly. BRIEF says "Thread exits cleanly via Thread/drain-and-join" in the counter-actor-proof comment, but the body sketch uses join-result implicitly. Use drain-and-join in all BRIEF examples for user-namespace test bodies.

### Delta 4 — Scope-deadlock checker: Senders must be in inner let scope before drain-and-join

**BRIEF assumption:** Single flat let scope for all bindings including drain-and-join.

**Actual:** The scope-deadlock checker (arc 131) sees Senders (`admin-tx` from Thread/input, `user-tx` from spawn-result) as siblings to the drain-and-join call and fires deadlock warnings. Fix: inner let holds all Senders + does all communication; returns only the thread to the outer scope. Outer scope calls drain-and-join after all senders dropped. SERVICE-PROGRAMS lockstep per SERVICE-PROGRAMS.md.

This is standard discipline (seen in service-template.wat) but not explicitly stated in BRIEF.

**Suggested BRIEF correction:** BRIEF body sketch should show the inner/outer let structure with a comment "inner let: senders + communication; outer: drain only."

### Delta 5 — 3-tuple return from spawn (first/second/third); NOT 4-tuple as BRIEF described

**BRIEF assumption:** "Returns (Tuple admin-tx admin-rx-for-caller user-tx-for-caller user-rx)" — a 4-tuple; BRIEF suggested `Tuple/N` indexing.

**Actual:**
1. `Tuple/N` accessors DON'T EXIST in the substrate. Only `first`, `second`, `third` are registered (src/check.rs:5234).
2. After Wire enum pivot, admin channels come naturally from the Thread handle (Thread/input + Thread/output). The spawn return carries only user-side extras.
3. 3-tuple used: `(thread, user-tx, user-resp-rx)` — accessible via first/second/third.

**Suggested BRIEF correction:** Remove `Tuple/N` from BRIEF. The correct form is `first`, `second`, `third`. For more than 3 elements, nest 2-tuples. For the Wire-enum architecture, spawn returns a 3-tuple: (Thread<Wire,AdminResp>, Sender<Wire>, Receiver<UserResp>).

## Files touched

| File | Change |
|------|--------|
| `wat-tests/counter-service-thread-N1.wat` | NEW — single deftest proving server dispatch foundation: Wire enum fan-in, admin Stop, user Get/Increment/Reset |
| `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3A.md` | THIS FILE |

## Workspace delta

- Pre-slice-3a baseline: 180 wat deftests (179 passing + 1 pre-existing failure).
- Post-slice-3a: 181 wat deftests (180 passing + 1 pre-existing failure).
- Net: +1 passing deftest, 0 new failures.

## Suggested BRIEF corrections for stones 3b-3d

1. **3b (dynamic registry / Provision/Deprovision):** Wire enum is already in place. Registry maps `client-id → Sender<Wire>`. Provision creates a new user channel pair (user-tx: Sender<Wire> sent back to admin), Deprovision removes it. Server maintains `Vec<(client-id, Receiver<Wire>, Sender<UserResp>)>` and rebuilds the select set each iteration. No new wire type needed.

2. **3c (capability structs):** `Counter::AdminClient` holds `Sender<Wire>` (admin variants) + `Receiver<AdminResp>`. `Counter::UserClient` holds `Sender<Wire>` (user variants) + `Receiver<UserResp>`. Constructor whitelist: `[:counter::]`. The type system still doesn't prevent an AdminClient from sending Wire::User variants — behavior enforces. If 3c wants TRUE type separation, it needs two separate channel types (AdminWire, UserWire enums), but then select is back to heterogeneous problem. Honest answer: either keep unified Wire (behavior-enforces) or re-spec 3c as two separate server loops (admin-only + user-only) with cross-signaling.

3. **3d (process variant):** The stdio stream carries line-delimited EDN. Wire enum variants encode as EDN atoms/maps. Process variant replaces the thread; Wire encoding/decoding replaces send/recv. No new select needed (process only handles one stream). Admin stop is a protocol message on stdin; response on stdout. Same Wire enum works.

4. **All slices:** Use `Thread/drain-and-join` in test bodies (not `Thread/join-result`). Use inner/outer let structure for scope-deadlock compliance. Use `first/second/third` for tuples (no `Tuple/N`). Inline `:(...)` annotations on one line.

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 45-75 min | ~35 min (within band) |
| Scorecard rows | 4/4 PASS | 4/4 PASS |
| Workspace fail count | 3 | 3 (stable) |
| New deftest count | 1 | 1 |
| STOP-1-fires (Wire enum pivot) | YES (likely) | YES (fired; adopted) |
| Other substrate↔assumption gaps surfaced | 1-2 | 4 (whitespace in `:(...)`, `Thread/join-result` restricted, scope-deadlock inner-let, `Tuple/N` nonexistent) |
| BRIEF corrections suggested for stones 3b-3d | 1-2 | 5 |

**Calibration summary:** All predicted outcomes matched. STOP 1 fired as anticipated; Wire enum adopted cleanly. The four additional gaps (whitespace, join-result restriction, scope-deadlock pattern, Tuple/N) are all documented in WAT-CHEATSHEET.md or SERVICE-PROGRAMS.md but weren't applied in the BRIEF examples. The 3b-3d corrections from Delta 1 are the most architecturally significant: the Wire enum already supports dynamic provisioning without further wire-type changes.
