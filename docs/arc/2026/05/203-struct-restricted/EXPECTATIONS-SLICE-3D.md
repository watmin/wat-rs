# EXPECTATIONS — Arc 203 Slice 3d: process variant

**BRIEF:** `BRIEF-SLICE-3D.md`
**Drafted:** 2026-05-17 post-slice-3c commit `e7aa671`.

## Independent prediction

**Runtime band:** 90-120 min sonnet.

Reasoning:
- Most complex stone of the chain so far
- Subprocess program construction (`:wat::core::forms` inline declaration of enums + :user::main)
- Bidirectional Wire/WireResp enums (parent and subprocess both declare)
- Multi-variant enums with EDN round-trip across process boundary
- ProcessPeer in struct-restricted fields (slice 3c proved channel values work; ProcessPeer is itself a struct)
- ~400-500 lines artifact estimated

**Time-box:** 150 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — form parses + compiles | YES | medium-high (subprocess program forms novel; EDN encoding may surface edges) |
| B — process variant lifecycle works end-to-end | YES | medium (most novel; combines slice 3c capability pattern + Counter actor process proof's spawn-process construction) |
| C — EDN round-trip Wire + WireResp | YES | medium-high (slice 2 EDN coercion fix + Counter actor proofs both prove enum round-trip; this combines them) |
| D — capability enforcement at process tier | YES | high (same struct-restricted mechanism as slice 3c) |
| E — workspace baseline preserved | YES | high (purely additive) |

**5/5 PASS predicted; ~70% confidence overall.** Lower than 3c (capability pattern was straightforward extension) because process tier introduces multiple novel composition surfaces (subprocess program forms + bidirectional Wire + EDN multi-variant + ProcessPeer-in-struct).

## Honest deltas predicted

### Likely

1. **Subprocess `:user::main` shape** — must end in `-> :wat::core::nil` per arc 170 slice 1e; subprocess dispatch loop tail-calls itself; verify pattern matches Counter actor process proof
2. **`:wat::core::forms` syntax for inline program construction** — sonnet copies from counter-actor-proof-process.wat; verify forms-vector + spawn-process composition works
3. **EDN encoding for `(Wire/User id req)` carrying nested enum** — `req` is itself a `:counter::UserReq` variant; verify the substrate's wat-edn handles nested-enum-as-payload (slice 2 EDN coercion fix made zero-field tagged variants round-trip; multi-field carrying enum should work)
4. **ProcessPeer/new arg order** — slice 2 SCORE delta 6 documented `(rx, tx)` order; sonnet verifies during construction

### Less likely

5. **Sequential response matching** — if subprocess interleaves responses (it shouldn't; single-threaded server), parent reads wrong response. Slice 3d's server is request-response sequential; verify no out-of-band emission
6. **AdminProc holding both peer + proc** — verify struct-restricted accepts both ProcessPeer field AND Process field; slice 3c proved Thread inside struct works (Thread is a value); same should hold for Process
7. **Drain-and-join in `:counter::stop-proc`** — inner/outer let pattern absorbs the lockstep per slice 3c precedent; verify Process/drain-and-join signature matches

## Workspace baseline (post-slice-3c commit `e7aa671`)

3 pre-existing failures.

Post-3d target: +1 passing deftest (`deftest_counter_service_process_N3`); 3 failures preserved.

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 90-120 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | 3 | TBD | TBD |
| New deftest count | 1 | TBD | TBD |
| EDN encoding surprises | 0-2 | TBD | TBD |
| Subprocess program form-construction surprises | 0-2 | TBD | TBD |
| Substrate↔assumption gaps surfaced | 1-3 | TBD | TBD |
| BRIEF corrections suggested for slice 3e | 0-2 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
