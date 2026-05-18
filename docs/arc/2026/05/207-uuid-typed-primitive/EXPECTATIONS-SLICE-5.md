# EXPECTATIONS — Arc 207 Slice 5

## Mode prediction

**Mode A — clean closure paperwork ships (~85%).** Sonnet drafts INSCRIPTION, updates DESIGN status, appends 058 row, FM 11 grep clean, all rows YES. ~35-45 min wall-clock.

**Mode B — FM 11 grep catches deferral language (~10%).** First INSCRIPTION draft contains some "future arc when X surfaces" prose; sonnet rewrites to affirmative form; second grep clean. Adds ~5-10 min.

**Mode C — DESIGN status update has surprise (~3%).** DESIGN file structure changed since slice 1 forward-corrections in ways that make the simple OPEN→CLOSED replacement ambiguous. Sonnet reads carefully + adapts. Adds ~5 min.

**Mode D — 058 row format unclear (~2%).** Sonnet reads arc 200/201/202/206 rows for format precedent, picks the consistent format. Adds ~5 min.

Hard stop 60 min. Pure paperwork should NOT take longer than that.

## INSCRIPTION quality bar

The INSCRIPTION is the load-bearing artifact for "arc 207 closes the deferral pattern arc 206 inscribed." It must:
- Honestly name arc 206's framing as wrong (without disrespecting arc 206 — INSCRIPTIONs are historical; the wrongness IS the lesson)
- Inscribe the discipline carry-forward as quotable doctrine ("before naming anything out-of-scope no-consumer-demands-it, grep the substrate for arms/errors/panics that name the missing type")
- Demonstrate the substrate-as-teacher cascade running cleanly across slices 3 + 4 (in-scope fixes when consumer code surfaced gaps)
- Cross-reference but NOT amend arc 206 INSCRIPTIONs

Affirmative-out-of-scope language pattern (per FM 11):
- ✓ "Arc 207 intentionally does NOT cover X because <architectural reason>; if/when a consumer surfaces with concrete shape pressure, a new arc opens"
- ✗ "X deferred to a future arc" / "X to be added later" / "X not yet implemented"

The forward-correction part of the INSCRIPTION can quote prior INSCRIPTIONs as evidence of the discipline failure, but should NOT modify those prior files.

## Workspace baseline expected

Sonnet doesn't run cargo test (no source changes). Workspace baseline unchanged from slice 4 (3 pre-existing failures).

## Out-of-scope findings (surface, don't act)

- USER-GUIDE § 11 was rewritten in slice 4; if INSCRIPTION inscription discovers any inconsistency with slice 4's USER-GUIDE content, surface (don't edit USER-GUIDE — already shipped)
- Any inscription idea that would require touching arc 206 INSCRIPTIONs (immutable; do NOT touch)
- Any discipline lesson worth carrying into a memory/INTERSTITIAL entry separate from this INSCRIPTION — surface for orchestrator decision (NOT in scope to write here)

## Failure-mode catches

- FM 11 (the MAIN check) — the grep is the discipline; trust the grep
- FM 14 (surface retirement leftover) — slices 3+4 already swept namespace verbs; INSCRIPTION just narrates
- FM 16 (no tool preamble) — BRIEF doesn't preamble tool availability
- `feedback_inscription_immutable` — arc 206 INSCRIPTIONs stay unchanged; THIS INSCRIPTION carries the forward-correction

## Atomic commit shape

Sonnet writes 3 artifacts; orchestrator commits in TWO atomic commits:
1. **wat-rs commit:** INSCRIPTION.md + DESIGN.md update + SCORE-SLICE-5.md (one commit)
2. **lab commit:** 058 changelog row append (separate repo)

Both pushed.

## Calibration record

- Slice 1 (audit): 36 min
- Slice 2 (substantive): 93 min
- Slice 3 (mechanical): ~30 min
- Slice 4 (consumer ripple + Mode D fix): ~30 min (sonnet's report didn't time-stamp but bounded; 5 files + Mode D fix in <31 min real-time)
- Slice 5 (paperwork): predicted 35-45 min; smallest slice; bounded

If sonnet runs over 45 min on pure paperwork, something's structurally wrong — surface for orchestrator review.

## What "done" means for slice 5 (and arc 207)

- 7/7 SCORE rows YES
- INSCRIPTION.md exists; FM 11 grep clean; reads as honest closure record + forward-correction
- DESIGN status CLOSED; slice table reflects all 5 slices SHIPPED
- 058 row appended in lab repo
- Both commits pushed
- Arc 170 unblocks; arc 203 unblocks; lab reconstruction unblocks

Sonnet: trust the disk; trust the discipline; ship the closure.
