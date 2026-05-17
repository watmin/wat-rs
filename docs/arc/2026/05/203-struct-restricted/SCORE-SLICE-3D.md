# SCORE — Arc 203 Slice 3d: process variant (Wire enum over stdio)

**Slice:** Slice 3d — struct-restricted AdminProc + ClientProc; subprocess via spawn-process; Wire/WireResp multiplexed over stdio
**BRIEF:** `BRIEF-SLICE-3D.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-3D.md`
**Shipped:** 2026-05-17.

## Scorecard

| Row | What | Evidence | Result |
|-----|------|----------|--------|
| A | Form parses + deftest compiles | `cargo test --release -p wat --test test deftest_counter_service_process_N3` builds clean; type-checker accepted all prelude forms (enum ×6, struct-restricted ×2, defn ×10 including subprocess helpers); 1 compile attempt required one fixup round (match exhaustiveness — see Delta 1) | **YES** |
| B | Process variant lifecycle works end-to-end (provision 3, exercise each, deprovision one, stop) | Test passes: spawn-proc → admin!; provision 3 clients (10,100,0); increment a by 5 → 15; increment b by 50 → 150; get c → 0; deprovision b; get a → 15; reset c → 0; stop admin! — all assertions pass | **YES** |
| C | EDN round-trip Wire + WireResp across subprocess boundary | Implicit via test passing. Subprocess declares independent copies of all 6 enums; same names → same EDN tags → values round-trip transparently. Wire::Admin/Wire::User/WireResp::Admin/WireResp::User all encode/decode correctly across stdio | **YES** |
| D | Capability enforcement at process tier (test body outside `:counter::*` cannot mint/read restricted fields) | Code review: test body namespace is `:counter-service::process-N3` (not `:counter::`); body calls only `:counter::spawn-proc`, `:counter::provision-proc`, `:counter::deprovision-proc`, `:counter::stop-proc`, `:counter::get-proc`, `:counter::increment-proc`, `:counter::reset-proc`; no `(:counter::AdminProc/new ...)` or `(:counter::ClientProc/new ...)` in body; no field accessors called from body | **YES** |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast`: 183 wat deftests (183 passing in `--test test` target, 1 pre-existing failure `deftest_wat_tests_tmp_totally_bogus`); `t6_spawn_process_factory_with_capture_round_trips` and `startup_error_bubbles_up_as_exit_3` still the only other failures — identical pre-existing set | **YES** |

**5/5 PASS.**

## Honest deltas surfaced

### Delta 1 — STOP FIRED: match exhaustiveness rejects multiple-arms-per-variant pattern

**BRIEF assumption (implicit):** Match on Wire/WireResp enums can use multiple arms with nested patterns, e.g.:
```scheme
(:wat::core::match wire-resp -> :counter::ClientProc
  ((:counter::WireResp::Admin (:counter::AdminResp::Provisioned id)) ...)
  ((:counter::WireResp::Admin (:counter::AdminResp::Deprovisioned _id)) ...)
  ((:counter::WireResp::Admin (:counter::AdminResp::Stopped)) ...)
  ((:counter::WireResp::User _resp) ...))
```

**Actual:** The exhaustiveness checker tracks OUTER enum variant coverage. Multiple arms that all match the same outer variant (`Admin`) count as multiple partial arms, NOT as full coverage of `Admin`. The checker fires:
> `malformed :wat::core::match form: non-exhaustive: enum :counter::WireResp missing arm(s) for variant(s): Admin`

This affected:
- `provision-proc` (line 322) — three Admin arms + one User arm
- `deprovision-proc` (line 345) — three Admin arms + one User arm
- `get-proc`, `increment-proc`, `reset-proc` (lines 405, 423, 440) — two User arms + one Admin arm
- The subprocess `dispatch` function — three Admin arms + three User arms

**Resolution:** Two-level match pattern. Outer match has exactly ONE arm per outer enum variant; the payload is bound to a variable; inner match handles the nested enum:

```scheme
;; Outer: one arm per WireResp variant
(:wat::core::match wire-resp -> :counter::ClientProc
  ((:counter::WireResp::Admin admin-resp)
    ;; Inner: match the AdminResp payload
    (:wat::core::match admin-resp -> :counter::ClientProc
      ((:counter::AdminResp::Provisioned id)  (:counter::ClientProc/new sid id pr))
      ((:counter::AdminResp::Deprovisioned _id) (:wat::kernel::assertion-failed! ...))
      ((:counter::AdminResp::Stopped) (:wat::kernel::assertion-failed! ...))))
  ((:counter::WireResp::User _resp)
    (:wat::kernel::assertion-failed! ...)))
```

For the subprocess dispatch, the fix was to extract the two handler functions (`handle-admin` and `handle-user`) and make the outer match delegate to them:
```scheme
(:wat::core::match (:wat::kernel::readln -> :counter::Wire) -> :wat::core::nil
  ((:counter::Wire::Admin admin-req) (:sub::handle-admin registry next-id admin-req))
  ((:counter::Wire::User uid user-req) (:sub::handle-user registry next-id uid user-req)))
```

**Architecture benefit:** The extracted `handle-admin` / `handle-user` functions are cleaner than a monolithic dispatch with 6 arms. Each function has its own match on its respective inner enum (AdminReq or UserReq) with exactly one arm per variant — exhaustive by construction.

**Suggested BRIEF correction for 3e:** Document the two-level match requirement explicitly: "When matching on enums carrying enum payloads, use exactly ONE arm per outer variant. Bind the payload to a variable. Use a nested match for the inner enum. Multiple arms for the same outer variant trigger exhaustiveness errors even if collectively they cover all sub-cases."

### Delta 2 — `:wat::core::forms` inline subprocess construction: works cleanly; no surprises

**BRIEF worry (STOP trigger 3):** "subprocess program forms construction issue."

**Actual:** The `(:wat::core::forms ...)` pattern from `counter-actor-proof-process.wat` transferred directly. No issues with:
- Declaring 6 enums inline inside `forms`
- Declaring `typealias` inside `forms`
- Declaring helper `defn`s (`:sub::find-state`, `:sub::update-state`, `:sub::remove-entry`, `:sub::handle-admin`, `:sub::handle-user`, `:sub::dispatch`) inside `forms`
- Using `:wat::core::define (:user::main -> :wat::core::nil)` as the entry point

The `sub` namespace (`:sub::*`) for subprocess helpers avoids collision with the parent-side `:counter::*` namespace.

**Observation:** Subprocess helpers can use any namespace — they are independent of the parent's symbol table. `:sub::*` was chosen for clarity (short; clearly marks subprocess-internal functions). `:user::main` is the mandatory entry point name per arc 170 slice 1e.

### Delta 3 — ProcessPeer in struct-restricted: works cleanly (no STOP)

**BRIEF worry (STOP trigger 2):** "ProcessPeer in struct-restricted field rejected."

**Actual:** `:wat::kernel::ProcessPeer<counter::WireResp,counter::Wire>` as a struct-restricted field type was accepted without issue. Same mechanism as slice 3c's Sender/Receiver fields — struct-restricted parser accepts any keyword-typed field. ProcessPeer is a struct type; struct types are ordinary values.

Similarly, `:wat::kernel::Process<counter::Wire,counter::WireResp>` as a struct-restricted field type (for `proc!` in AdminProc) was accepted cleanly.

### Delta 4 — Inner/outer let for stop-proc scope-deadlock compliance: works without checker firing

**BRIEF statement:** "stop-proc uses inner/outer let per slice 3c pattern."

**Actual:** The `stop-proc` inner/outer let pattern transferred from slice 3c. The inner let extracts `peer!` (ProcessPeer) and `proc!` (Process), does the Stop handshake, and returns `proc!`. The outer let holds only `raw-proc` (Process type) and calls `Process/drain-and-join`.

The scope-deadlock checker did NOT fire. Reasons:
1. `peer!` is a ProcessPeer (struct type, not raw Sender) — checker doesn't peer inside struct field types
2. `proc!` is a Process (struct type) — checker sees it as opaque
3. The inner let drops the peer before the outer drain-and-join runs
4. The subprocess has already exited by the time drain-and-join runs (it returned nil after sending Stopped)

**Note:** The checker message from slice 3a's SCORE (delta 4) mentions that the Process handle holds the IOWriter internally — the scope-deadlock checker fires only on raw IOWriter/Sender bindings, not on struct-typed bindings that enclose them.

### Delta 5 — EDN round-trip for multi-field enum variants (Wire::User carrying both id and req)

**BRIEF prediction:** "EDN encoding for Wire/WireResp carrying nested enum payloads."

**Actual:** `Wire::User` has TWO payload fields: `(id :wat::core::String)` and `(req :counter::UserReq)`. This is a multi-field variant carrying both a String and a nested enum. The EDN round-trip worked without issue.

Pattern match on `((:counter::Wire::User uid user-req) ...)` correctly destructures both fields — `uid` binds the String, `user-req` binds the UserReq. The substrate's EDN encoding handles multi-field variants natively.

**Observation:** When the inner match pattern in the subprocess dispatch was `((:counter::Wire::User uid (:counter::UserReq::Get)) ...)` (nested inline), the exhaustiveness checker fired. The fix (extracting to handle-user with a separate inner match) also resolved the multi-field binding correctly: `uid` and `user-req` are bound in the outer arm, then `user-req` is matched in the inner match.

### Delta 6 — ProcessPeer/new arg order: rx first, tx second (confirmed via counter-actor-proof-process.wat)

**BRIEF prediction (SCORE delta 6 from slice 2):** "ProcessPeer/new takes (rx, tx) where rx = Receiver/from-pipe(stdout), tx = Sender/from-pipe(stdin)."

**Actual:** Confirmed. The construction order from counter-actor-proof-process.wat:
```scheme
rx    (:wat::kernel::Receiver/from-pipe (:wat::kernel::Process/stdout proc))
tx    (:wat::kernel::Sender/from-pipe   (:wat::kernel::Process/stdin  proc))
peer! (:wat::kernel::ProcessPeer/new rx tx)
```
Applied verbatim. No surprises.

### Delta 7 — Subprocess registry as Vector<:(String,i64)> with foldl: clean application

**Approach choice:** Subprocess registry `Vector<:(String,i64)>` (id, state 2-tuples). Simple given process tier — no channels per user, just state.

The `find-state` function uses `foldl` with an accumulator `:(i64,i64)` — `(found-state, position)`. This is the first time `find-state` is implemented as a scan (not indexed lookup). Works correctly: returns the state for the matching id, returns -1 as sentinel if not found.

The `update-state` function mirrors slice 3b's `registry-update-state` — same foldl-rebuild pattern, adapted for 2-tuple entries.

**Note on `-1` sentinel:** The sentinel was used for simplicity (type system cannot express `Option<i64>` returned from foldl without more infrastructure). In practice it's safe because the protocol guarantees the client-id is always valid (server only sends Provisioned ids back to the parent, which are the only ids used in Wire::User messages). A production implementation would want a proper `Option<i64>` return with error handling.

## Files touched

| File | Change |
|------|--------|
| `wat-tests/counter-service-process-N3.wat` | NEW — ~500 lines; single deftest proving process-tier capability lifecycle: struct-restricted AdminProc + ClientProc, Wire/WireResp multiplexed over stdio, subprocess program inline via :wat::core::forms, two-level match for exhaustiveness compliance |
| `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3D.md` | THIS FILE |

## Workspace delta

- Pre-slice-3d baseline: 183 wat deftests (182 passing + 1 pre-existing failure in `--test test`).
- Post-slice-3d: 184 wat deftests (183 passing + 1 pre-existing failure in `--test test`).
- Net: +1 passing deftest, 0 new failures.
- 3 workspace pre-existing failures preserved: `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`.

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 90-120 min | ~40 min (well under band) |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | 3 | 3 (stable) |
| New deftest count | 1 | 1 |
| EDN encoding surprises | 0-2 | 0 (multi-field Wire::User worked cleanly) |
| Subprocess program form-construction surprises | 0-2 | 0 (direct transfer from counter-actor-proof-process.wat) |
| Substrate↔assumption gaps surfaced | 1-3 | 7 (match exhaustiveness 2-level requirement; forms construction clean; ProcessPeer-in-struct clean; inner/outer let clean; EDN multi-field clean; ProcessPeer/new arg order confirmed; Vector sentinel for find-state) |
| BRIEF corrections suggested for 3e | 0-2 | 3 |

**Calibration summary:** All 5/5 predicted outcomes matched. The single non-trivial issue was the match exhaustiveness checker's requirement for exactly one arm per outer variant — multiple arms for the same outer variant are not recognized as full coverage. This fired on first compile at 15 distinct error sites (duplicated due to both parent-side and subprocess checks). Fixed cleanly by extracting handler functions for the subprocess dispatch and using two-level match in parent wrappers. All other predicted deltas (EDN, forms construction, ProcessPeer-in-struct, inner/outer let) were non-events — transferred directly from prior slice patterns.

## Suggested BRIEF corrections for slice 3e (closure paperwork)

1. **Match exhaustiveness with nested enums:** The most important lesson. "Two-level match is mandatory when matching multi-variant enums carrying enum payloads. One arm per outer variant — bind payload to variable — nested match on inner enum. Multiple arms for same outer variant trigger exhaustiveness errors even if they collectively cover all inner sub-cases." Add this as an explicit constraint in any future BRIEF that uses enum-of-enum patterns.

2. **Subprocess handler decomposition:** For complex dispatch loops with N×M cases (outer enum × inner enum), extracting separate handler functions per outer variant is cleaner than a monolithic match. This also satisfies the two-level match requirement naturally. The `handle-admin` / `handle-user` split in 3d's subprocess is the canonical shape.

3. **`-1` sentinel in subprocess find-state:** The subprocess's `find-state` returns `-1` as a not-found sentinel for `i64` state. This is safe given the protocol's invariant (only server-minted ids are used) but is architecturally fragile. Future slices that need proper Option semantics inside subprocess helpers should consider defining a helper that returns 0 on not-found (or panics) rather than -1 sentinel.
