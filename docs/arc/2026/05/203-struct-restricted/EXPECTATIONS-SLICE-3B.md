# EXPECTATIONS — Arc 203 Slice 3b: dynamic Provision/Deprovision

**BRIEF:** `BRIEF-SLICE-3B.md`
**Drafted:** 2026-05-17 post-slice-3a commit `d4d76b4`.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

Reasoning:
- Extends 3a (~370 lines) with registry + dynamic select-set rebuild + multi-user lifecycle
- New artifacts: ~150-250 lines added (registry as HashMap or Vector of records; expanded AdminReq/AdminResp; updated dispatch; multi-user test body)
- Main novelty: HashMap-carrying-Receiver-values may surface substrate edges; dynamic select-set rebuild each iteration; per-user state tracking

**Time-box:** 120 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — form parses + compiles | YES | medium-high (HashMap-with-channel-values may need workaround) |
| B — Provision returns ends + distinct IDs | YES | high |
| C — per-user state independent | YES | high |
| D — Deprovision drops; others continue | YES | medium-high (dynamic select-set rebuild may surface edge cases) |
| E — workspace baseline preserved | YES | high |

**5/5 PASS predicted; ~70% confidence overall.** Lower than 3a because registry-as-state with embedded channel values is new ground; substrate may force a Vector-of-records fallback.

## Honest deltas predicted

### Likely

1. **Registry shape pivot** — if HashMap<String, ...> with embedded Receiver<Wire> values has issues (parse, type-check, runtime), sonnet pivots to Vector<RegistryEntry> where RegistryEntry is a struct holding (id, rx, tx, state). Both are valid; Vector is slightly heavier per-lookup but works.

2. **Scope-deadlock checker firing on registry Senders** — server-side dispatch holds N user-side-tx Senders. Checker may flag. Sonnet uses inner-let or factored-fn pattern per slice 3a's experience.

3. **AdminResp variant carrying Sender + Receiver values** — should work (channel values are first-class wat values per arc 110/111), but verify.

### Less likely

4. **Dynamic select-set construction** — `:wat::kernel::select` takes `Vec<Receiver<Wire>>`. Building this Vec from `[admin-rx, *(map .rx (.values registry))]` each iteration should work via Vector primitives + cons; verify if there's an idiomatic pattern.

5. **Auto-cleanup on Disconnect** — `select` returns `(idx, Result<Option<Wire>, ThreadDiedError>)`. When idx>0 and result is `Err(Disconnected)`, server drops that registry entry on next iteration. Verify Disconnected is surfaceable.

## Workspace baseline (post-slice-3a commit `d4d76b4`)

3 pre-existing failures preserved across slice 3a.

Post-3b target: +1 passing deftest (deftest_counter_service_thread_N3); 3 failures preserved.

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 60-90 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | 3 | TBD | TBD |
| New deftest count | 1 | TBD | TBD |
| Registry shape (HashMap vs Vector-of-records) | HashMap likely; Vector fallback if needed | TBD | TBD |
| Substrate↔assumption gaps surfaced | 1-3 | TBD | TBD |
| BRIEF corrections suggested for stones 3c-3d | 1-2 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
