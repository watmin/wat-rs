# Vigilia loop — do not forget

This is the ritual. Compaction does not cancel it. Read this before every recast.

## The four steps

0. **Push gates — ALL four required, then commit and push `grok-rete`.**
   - L1 zero
   - L2 zero
   - `cargo clippy --all-targets -- -D warnings` silent
   - `scripts/floor.sh` GREEN (do not re-run a red floor)
   Never merge `origin/main` until the user explicitly asks.
   Never stamp vigilatum until a live recast is 0+0 AND clippy-zero AND the user asks.

1. **Run vigilia.** Identify any applicable spells against the intern since the last consecutive 0+0.

2. **Cast the spells.** If that recast is 0 L1 + 0 L2, recast again. Two consecutive recasts at 0+0 exits the loop (then step 0).

3. **Address applicable spells.** Drive every L1 and L2 to zero. L3 is judgement — they never terminate. Then go back to step 1.

## Recast standing orders

- 18-spell roster; mora OUT.
- Inward 17 in parallel, circumspicere last.
- Workers: `general-purpose`, `capability_mode: all`, FIRST action `datamancy__fetch_spell`. Do not forbid MCP. Do not write spells to disk. Do not embed SKILL text in user-facing replies. Embed the SKILL in the worker prompt only if MCP fetch is not available.
- Partire LEAVE `src/rete/kernel/tests.rs`.
- Holon root git FROZEN. Leave dirty unless asked to commit (except step 0).
- Work stays on `grok-rete`.

## Freshness

**Moved here from `tmp/` on 2026-08-25 and TRACKED.** It lived untracked beside ~34 scratch
run directories, one `rm -rf tmp/` from being lost — and it is doctrine, not run output. The
run directories stay scratch and are now gitignored; this file is the ritual and belongs in the
record.

- **2026-08-24 — a full vigilia, 18 wards, 4 converged.** This one did NOT exit by the
  two-consecutive-0+0 rule below: it returned a real list (~21 L1, ~29 L2) and the arc worked it
  through instead. Four LIVE defects came out of it, none reachable by measurement. The findings
  and the closing tally live in
  `docs/arc/2026/06/278-rules-engine/NEXT-STRIKES-theater-hunt.md`.
- **2026-08-25** — remaining prose/naming/record items driven down; see the same file's
  CLOSING TALLY and TRACKED DECISIONS.
- Last consecutive 0+0: recast 33 + recast 34 at `36802e7e` (identity-filter, from_one, seed
  range, catch-up Arc). Clippy+floor green that turn. Loop exited.
- Prior consecutive 0+0: recast 31 + recast 32 (occupancy intern).

> ⚠ Two consecutive 0+0 exits the loop, but a 0+0 pair is not proof the code is clean — it is
> proof the CURRENT roster found nothing. Recasts 33/34 were both 0+0; the 2026-08-24 cast that
> followed found four live defects. Read that as a statement about ward coverage, not about the
> code having regressed.

