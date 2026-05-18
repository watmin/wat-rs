# EXPECTATIONS — Arc 208 Slice 3

## Mode prediction

**Mode A — clean closure paperwork ships (~85%).** Sonnet drafts INSCRIPTION, updates DESIGN status, appends 058 row, FM 11 grep clean, all rows YES. ~30-40 min wall-clock.

**Mode B — FM 11 grep catches deferral language (~10%).** First INSCRIPTION draft contains some "future arc" or "when X surfaces" prose; sonnet rewrites to affirmative form; second grep clean. Adds ~5-10 min.

**Mode C — minor adjustment (~5%).** 058 row format quirk or DESIGN slice table edit needs minor reshaping; sonnet adapts.

Hard stop 60 min. Pure paperwork.

## INSCRIPTION quality bar

The INSCRIPTION is the load-bearing artifact for arc 208's closure record. It must:
- Honestly name the substantive work (substrate Result-flip + walker + consumer ripple)
- Inscribe the mirror-precedent discipline as load-bearing for future substrate evolution
- Cross-reference but NOT amend arc 110/111/112/203 historical INSCRIPTIONs
- Explicitly disclaim the orphan-leak fix (arc 208 doesn't address it; INTERSTITIAL leak notes are the diagnostic for that separate concern)

Affirmative-out-of-scope language pattern (per FM 11):
- ✓ "Arc 208 intentionally does NOT cover X because <reason>"
- ✗ "X deferred to future arc when Y surfaces"

## Workspace baseline expected

Sonnet doesn't run cargo test (no source changes). Workspace baseline unchanged from slice 2 (4 flaky failures: lifeline / tmp_totally_bogus / t6 / startup_error_exit_3).

## Out-of-scope findings (surface, don't act)

- USER-GUIDE updates documenting the new Result-returning Process I/O — slice 3 doesn't update USER-GUIDE explicitly; if INSCRIPTION inscription surfaces a USER-GUIDE gap, surface for orchestrator (NOT in scope here)
- Any discipline lesson worth carrying to memory beyond what INSCRIPTION captures

## Failure-mode catches

- FM 11 (the MAIN check) — the grep is the discipline; trust the grep
- FM 14 (surface retirement leaving leftovers) — slice 1 retired no surface; slice 2 retired Result/expect option-a patches; INSCRIPTION just narrates
- FM 16 (no tool preamble) — BRIEF doesn't preamble tool availability
- `feedback_inscription_immutable` — arc 110/111/112/203 INSCRIPTIONs stay unchanged

## Atomic commit shape

Sonnet writes 3 artifacts; orchestrator commits in TWO atomic commits:
1. **wat-rs commit:** INSCRIPTION.md + DESIGN.md update + SCORE-SLICE-3.md (one commit)
2. **lab commit:** 058 changelog row append (separate repo)

Both pushed.

## Calibration record

- Slice 1 (substrate flip + walker): ~93 min sonnet
- Slice 2 (consumer ripple + ServerDied): ~91 min sonnet (per task duration)
- Slice 3 (paperwork): predicted 30-40 min; smaller; bounded

If sonnet runs over 45 min on pure paperwork, something's structurally wrong — surface.

## What "done" means for slice 3 (and arc 208)

- 7/7 SCORE rows YES
- INSCRIPTION.md exists; FM 11 grep clean; honest closure record + mirror-precedent discipline carry-forward
- DESIGN status CLOSED; slice table reflects all 3 slices SHIPPED
- 058 row appended in lab repo
- Both commits pushed
- Arc 203 demand 2 satisfied; arc 203 closure waits on demand 1 (protocols arc)

Sonnet: trust the disk; trust the discipline; ship the closure.
