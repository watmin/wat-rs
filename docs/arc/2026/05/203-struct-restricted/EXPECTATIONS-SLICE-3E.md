# EXPECTATIONS — Arc 203 Slice 3e: server-id validation wiring

**BRIEF:** `BRIEF-SLICE-3E.md`
**Drafted:** 2026-05-17 post-slice-3d commit `45a1727`.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

Reasoning:
- Modifies BOTH existing files (3c + 3d) in-place
- Wire enum grows: Admin from 1 arg → 2 args, User from 2 args → 3 args (server-id added)
- Response enums grow: AdminResp + UserResp each gain AccessDenied variant
- All wrappers updated to embed server-id when constructing Wire
- Server dispatch in both files validates server-id; AccessDenied on mismatch
- All match sites updated for new variant exhaustiveness
- Tests still pass (happy path)
- Optional forge demonstration if cleanly doable

**Time-box:** 120 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — both files parse + tests compile | YES | medium-high (Wire arg-count growth touches many sites; match exhaustiveness must be updated everywhere) |
| B — thread variant happy path | YES | high |
| C — process variant happy path | YES | medium-high (process tier has more wrappers + subprocess program forms; more sites to update) |
| D — server validates server-id (code review) | YES | high |
| E — workspace baseline preserved | YES | high |

**5/5 PASS predicted; ~75% confidence overall.** Lower than 3c (capability extension was cleanly additive) because this slice modifies many existing call sites; risk is missing one variant somewhere.

## Honest deltas predicted

### Likely

1. **Match exhaustiveness fires repeatedly** — per slice 3d lesson, every match on AdminResp + UserResp needs a new arm for AccessDenied. Sonnet will iterate; expect 5-15 fix cycles
2. **Wire constructor sites all update** — every send wrapper constructs Wire with new arg count; sonnet sweeps systematically
3. **Server dispatch shape** — need a one-line check at top of admin/user handlers before routing; sonnet picks clean idiom
4. **Forge demonstration** — sonnet may surface it as too contrived to ship cleanly (capability fields restricted; can't construct invalid Wire from outside `:counter::*`); document validated pattern as the documentation and skip forge test

### Less likely

5. **Server-id value source** — sonnet uses constant string per BRIEF; verify chosen value
6. **Process subprocess program forms re-construction** — subprocess declares its own Wire enum; same growth applies inside the forms block

## Workspace baseline (post-slice-3d commit `45a1727`)

3 pre-existing failures.

Post-3e target: same 184 passing wat deftests (unchanged count, same tests in-place); 3 pre-existing failures preserved.

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 60-90 min | TBD | TBD |
| Scorecard rows | 5/5 PASS | TBD | TBD |
| Workspace fail count | 3 | TBD | TBD |
| Forge demonstration shipped | possible; depends on cleanliness | TBD | TBD |
| Substrate↔assumption gaps surfaced | 1-3 | TBD | TBD |
| BRIEF corrections suggested for slice 3f | 0-2 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
