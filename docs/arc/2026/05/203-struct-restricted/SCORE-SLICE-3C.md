# SCORE — Arc 203 Slice 3c: capability structs (struct-restricted Admin + Client)

**Slice:** Slice 3c — struct-restricted Admin + Client capability wrappers
**BRIEF:** `BRIEF-SLICE-3C.md`
**EXPECTATIONS:** `EXPECTATIONS-SLICE-3C.md`
**Shipped:** 2026-05-17.

## Scorecard

| Row | What | Evidence | Result |
|-----|------|----------|--------|
| A | Form parses + deftest compiles | `cargo test --release -p wat --test test deftest_counter_service_capability_N3` builds clean; type-checker accepted all prelude forms (enum ×5, typealias ×3, struct-restricted ×2, defn ×12); zero type fixups required; first compile attempt passed | **YES** |
| B | Capability wrappers work end-to-end (provision returns Client; each client independent ops; deprovision + stop work) | Test passes: spawn-cap → admin!; provision 3 clients (10,100,0); increment a by 5 → 15; increment b by 50 → 150; get c → 0; deprovision b; get a → 15; reset c → 0; stop admin! — all assertions pass | **YES** |
| C | Test body (outside `:counter::*`) CANNOT mint Admin or Client | Code review: test body prefix is `:counter-service::capability-N3` (not `:counter::`); body calls only `:counter::spawn-cap`, `:counter::provision`, `:counter::deprovision`, `:counter::stop`, `:counter::get`, `:counter::increment`, `:counter::reset`; no `(:counter::Admin/new ...)` or `(:counter::Client/new ...)` in body | **YES** |
| D | Test body CANNOT read restricted fields | Code review: body never calls `:counter::Admin/server-id`, `:counter::Admin/admin-tx`, `:counter::Admin/admin-rx`, `:counter::Admin/thread`, `:counter::Client/server-id`, `:counter::Client/client-id`, `:counter::Client/user-tx`, `:counter::Client/user-rx`; all field reads occur inside `:counter::*` defns | **YES** |
| E | Workspace baseline preserved | `cargo test --release --workspace --no-fail-fast`: 182 wat deftests (182 passing in `--test test` target, 1 pre-existing failure `deftest_wat_tests_tmp_totally_bogus`); `t6_spawn_process_factory_with_capture_round_trips` and `startup_error_bubbles_up_as_exit_3` still the only other failures — identical pre-existing set | **YES** |

**5/5 PASS.**

## Honest deltas surfaced

### Delta 1 — struct-restricted with separate Sender + Receiver fields: NO STOP (direct, no ThreadPeer)

**BRIEF assumption (STOP trigger 1):** "If substrate rejects channel-values as struct-restricted fields, surface; we may need to wrap channels in ThreadPeer first then put ThreadPeer in the capability."

**Actual:** The substrate accepted separate `Sender<Wire>` and `Receiver<UserResp>` as struct-restricted field types directly. No ThreadPeer wrapping needed. The struct-restricted parser accepts any keyword-typed field (line 1794-1808 of src/types.rs: field type parsed via `parse_type_expr_with_span`); channel types are ordinary type keywords. No rejection.

**Admin struct shape that worked:**
```scheme
(:wat::core::struct-restricted :counter::Admin
  [:counter::]
  ([:counter::] server-id <- :wat::core::String
   [:counter::] admin-tx  <- :wat::kernel::Sender<counter::Wire>
   [:counter::] admin-rx  <- :wat::kernel::Receiver<counter::AdminResp>
   [:counter::] thread    <- :wat::kernel::Thread<counter::Wire,counter::AdminResp>)
  ())
```

**Suggested BRIEF correction for 3d:** Remove the "ThreadPeer fallback" STOP trigger for separate channel fields — it's now proven unnecessary. Separate Sender + Receiver fields in struct-restricted work cleanly.

### Delta 2 — Thread stored IN Admin (not returned separately); stop absorbs drain-and-join

**BRIEF assumption:** "Whether Admin struct should also hold the Thread handle (for drain-and-join) or it's returned separately; sonnet picks cleanest."

**Actual:** Thread stored as the 4th restricted field on Admin. This was the cleanest option because:
1. Test body's BRIEF sketch shows only `admin!` from spawn (no separate Thread binding)
2. `stop` wrapper absorbs the SERVICE-PROGRAMS lockstep: inner-let extracts adm-tx clone + does send/recv + returns thread; outer-let holds only Thread → drain-and-join
3. Test body has NO inner/outer let structure — all SERVICE-PROGRAMS complexity absorbed into `:counter::stop`

**Architecture insight:** This is the key capability pattern advantage. Slice 3b's test body required inner/outer let for scope-deadlock compliance. Slice 3c's test body is a flat let — all lockstep discipline lives inside the wrappers.

**Suggested BRIEF correction for 3d:** "Admin holds the Thread handle. `:counter::stop` does the inner/outer let pattern internally. Test body is a flat let — no inner/outer structure needed."

### Delta 3 — Accessor semantics: clone (not move)

**BRIEF question (unstated):** Do struct field accessors MOVE the value out of the struct, or clone it?

**Actual:** Accessors use `struct-field` primitive (src/runtime.rs:1944-1948), which reads the field by index from the runtime struct Value. For channel ends (Sender/Receiver — Arc-wrapped crossbeam channels), the accessor returns a clone of the channel end. The Admin struct retains its internal field copy. The wrapper's local let-binding holds a second clone.

**Scope-deadlock implication:** In `:counter::stop`, the inner-let extracts `adm-tx` (a Sender<Wire> clone). The Admin struct's internal adm-tx clone remains alive in the `admin!` parameter until `stop` returns. However:
1. The inner-let drops the local `adm-tx` clone at exit
2. After the inner-let, the outer-let holds only `thread` (Thread type)
3. drain-and-join runs in the outer-let — the scope-deadlock checker sees only `thread` at this scope level (not the local `adm-tx` which already dropped)
4. The Admin struct's internal Sender is in `admin!` (a struct type, not a raw Sender) — the checker doesn't peer inside struct field types
5. Server exits cleanly after sending Stopped (regardless of Admin's internal Sender still being alive)
6. drain-and-join returns immediately (thread already exited)

**Result:** No scope-deadlock warning fired. Test passes.

**Suggested BRIEF correction for 3d:** "Struct field accessors clone channel ends (Arc-clones). The wrapper's local Sender clone drops at inner-let exit; the struct's internal copy remains alive but is enclosed in a struct type (invisible to scope-deadlock checker). This is safe provided the server has already exited before drain-and-join runs — which is guaranteed by the Stop → Stopped handshake."

### Delta 4 — spawn wrapper renamed to `:counter::spawn-cap` (not `:counter::spawn`)

**BRIEF assumption:** wrapper named `:counter::spawn`.

**Actual:** The slice 2 reference (`counter-client-capability-proof.wat`) already defines `:counter::spawn` (for a different constructor — the single-user ThreadPeer-based actor). Using `:counter::spawn` in slice 3c would collide if both files are loaded together. Renamed to `:counter::spawn-cap` to avoid collision.

Additionally, the slice 2 proof defines `:counter::get`, `:counter::increment`, `:counter::reset`, `:counter::shutdown`. The slice 3c wrappers with the same names for a different protocol shape would collide. However: each deftest has its own prelude (spliced under a unique deftest namespace at freeze time), so collision is only a concern if test files are loaded concurrently. In practice, each deftest runs in isolation with its own prelude, so collision doesn't manifest at runtime.

**Naming rationale:** `:counter::spawn-cap` is unambiguous (explicitly names "capability" version); avoids any potential collision with slice 2's `:counter::spawn`.

**Suggested BRIEF correction for 3d:** Mention that wrapper names should be unique across the test suite if tests might share a namespace. For process-variant (3d), name functions clearly (e.g. `:counter::spawn-process` or `:counter::spawn-cap-proc`).

### Delta 5 — No BRIEF errors encountered in form syntax; all slice 3b lessons applied successfully

**BRIEF assumption (various):** Lessons from 3a/3b corrections re: inner colon in tuples, foldl not reduce, inline :() annotations, first/second/third.

**Actual:** All lessons applied from memory on first pass:
- Inner type aliases in `:(...)` are bare: `counter::TxStatePair` not `:counter::TxStatePair` ✓
- Fold primitive is `foldl` ✓
- Inline `:(...)` on one line (no whitespace) ✓
- `first`/`second`/`third` tuple accessors (no `Tuple/N`) ✓
- Thread/drain-and-join (not Thread/join-result) ✓
- Inner/outer let for scope-deadlock compliance (inside `stop` wrapper) ✓
- AdminResp::Provisioned.rx is `Receiver<UserResp>` not `Receiver<Wire>` ✓

**Result:** Zero type-check fixups required. Test compiled and passed on first attempt.

## Files touched

| File | Change |
|------|--------|
| `wat-tests/counter-service-capability-N3.wat` | NEW — ~380 lines; single deftest proving capability-wrapped multi-user lifecycle: struct-restricted Admin + Client, privileged wrappers, flat test body |
| `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-3C.md` | THIS FILE |

## Workspace delta

- Pre-slice-3c baseline: 182 wat deftests (181 passing + 1 pre-existing failure in `--test test`).
- Post-slice-3c: 183 wat deftests (182 passing + 1 pre-existing failure in `--test test`).
- Net: +1 passing deftest, 0 new failures.
- 3 workspace pre-existing failures preserved: `deftest_wat_tests_tmp_totally_bogus`, `t6_spawn_process_factory_with_capture_round_trips`, `startup_error_bubbles_up_as_exit_3`.

## Calibration

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60-90 min | ~25 min (well under band) |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | 3 | 3 (stable) |
| New deftest count | 1 | 1 |
| struct-restricted-with-channel-fields shape | direct preferred; ThreadPeer fallback if rejected | direct (no fallback needed; substrate accepted immediately) |
| Substrate↔assumption gaps surfaced | 1-2 | 5 (direct channel fields ok, Thread-in-Admin, accessor-clone semantics, spawn-cap renaming, zero type fixups) |
| BRIEF corrections suggested for slice 3d | 1-2 | 5 |

**Calibration summary:** All 5/5 predicted outcomes matched. First compile attempt passed — all slice 3a/3b lessons applied correctly on first pass. The main novelty (struct-restricted with separate Sender + Receiver fields) worked directly without the ThreadPeer fallback. The Thread-in-Admin design choice was made by the implementor and absorbed all SERVICE-PROGRAMS complexity inside the wrappers — test body is a clean flat let.

## Suggested BRIEF corrections for slice 3d

1. **Wrapper naming disambiguation:** 3d (process variant) should use unambiguous names for spawn/wrappers that don't collide with slice 3c or slice 2 names. Suggest `:counter::spawn-proc` or `:counter::spawn-cap-proc`. Alternatively, place 3d wrappers under a separate sub-namespace.

2. **Thread-in-Admin is established pattern:** Admin holds Thread as restricted field; stop wrapper does inner/outer let + drain-and-join. For process variant, Process handle replaces Thread handle. Same pattern applies: Admin holds Process, stop extracts Process in inner-let, drain-and-join in outer-let. Document this explicitly in 3d BRIEF.

3. **Accessor semantics documented:** Struct field accessors clone Arc-wrapped values. No move semantics. Scope-deadlock checker does not peer inside struct field types — only top-level Sender bindings in let scope are checked. Document in 3d BRIEF to eliminate uncertainty.

4. **struct-restricted with channel fields proven:** No fallback to ThreadPeer needed. 3d can use separate Sender + Receiver fields in capability structs directly. Remove the ThreadPeer-wrap fallback STOP trigger from 3d BRIEF.

5. **Wire enum encoding for process variant:** 3d ships process variant. Wire enum variants must encode as EDN on stdio. Admin stop is a protocol message on stdin; response on stdout. Same unified Wire enum works; 3d adds EDN encoding/decoding layer. BRIEF should specify EDN shape for each Wire variant (e.g. `#counter.Wire/Admin #counter.AdminReq/Provision [10]` or simpler atom forms).
