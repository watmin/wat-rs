# EXPECTATIONS — Arc 203 Slice 3c: capability structs

**BRIEF:** `BRIEF-SLICE-3C.md`
**Drafted:** 2026-05-17 post-slice-3b commit `15cf7a8`.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

Reasoning:
- Wraps 3b's bare-channel multi-user flow in struct-restricted capability values
- New: 2 struct-restricted declarations (Admin + Client) + 6 wrapper defns
- Main novelty: struct-restricted with multiple channel-value fields (slice 2 used single ThreadPeer field; 3c uses separate Sender + Receiver fields)
- 3b's dispatch loop reused unchanged (server-side dispatch doesn't care about capability wrappers — only client API does)

**Time-box:** 120 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — form parses + compiles | YES | medium-high (struct-restricted with channel-value fields is new shape) |
| B — capability wrappers work end-to-end | YES | high (3b's dispatch unchanged; wrappers are thin layer) |
| C — test body cannot mint capabilities (outside `:counter::*`) | YES | high (struct-restricted enforces per slice 1+2 proven mechanism) |
| D — test body cannot read restricted fields | YES | high (same enforcement) |
| E — workspace baseline preserved | YES | high (purely additive) |

**5/5 PASS predicted; ~80% confidence overall.** Higher than 3b (substrate work was newer); lower than 3a (capability shape is new vs 3a's straightforward extension).

## Honest deltas predicted

### Likely

1. **struct-restricted with Sender + Receiver fields** — slice 2's Counter/Client used a single ThreadPeer field bundling channels. Slice 3c declares separate Sender + Receiver fields. Should work (channel values are first-class) but if substrate rejects, fall back to ThreadPeer-wrapping
2. **Server-id minting in `:counter::spawn`** — use a constant `"server-counter-0"` string (no telemetry dep)
3. **Wrappers accessing restricted fields** — `:counter::provision` reads `admin.admin-tx` (restricted accessor); should succeed since the defn is under `:counter::` prefix

### Less likely

4. **Test body's enum variant construction** — body uses wrappers, but may incidentally construct Wire variants (e.g., implicitly via provision wrapper). Verify no leakage of Wire construction outside `:counter::*`
5. **Drain-and-join with Admin capability** — admin holds the thread? Or `:counter::spawn` returns Admin AND a Thread? Sonnet picks based on what works cleanly; might add `thread!` as another restricted field on Admin

## Workspace baseline (post-slice-3b commit `15cf7a8`)

3 pre-existing failures.

Post-3c target: +1 passing deftest (`deftest_counter_service_capability_N3`); 3 failures preserved.

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 60-90 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | 3 | TBD | TBD |
| New deftest count | 1 | TBD | TBD |
| struct-restricted-with-channel-fields shape (direct vs ThreadPeer-wrapped) | direct preferred; fallback TBD | TBD | TBD |
| Substrate↔assumption gaps surfaced | 1-2 | TBD | TBD |
| BRIEF corrections suggested for slice 3d | 1-2 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
