# EXPECTATIONS — Arc 227 Stone 227.1b — defclass → defrecord rename

Mode A target: 8/8 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `wat/holon/defclass.wat` renamed to `wat/holon/defrecord.wat` | `git mv` performed; all internal references updated; `grep -n "defclass" wat/holon/defrecord.wat` returns 0 |
| 2 | Macro verb renamed inside file | Line 56 defmacro head now reads `(:wat::core::defmacro (:wat::holon::defrecord ...))`; line 70 error message says "defrecord" not "defclass"; all doc-comment examples updated |
| 3 | `src/stdlib.rs` updated | Lines 74/82/83 reflect new path + verb name; comment notes the 227.1b rename |
| 4 | `tests/probe_arc227_stone1_defclass.rs` renamed to `tests/probe_arc227_stone1_defrecord.rs` | `git mv` performed; ALL 69 internal mentions of defclass replaced with defrecord (in WAT source strings + test fn names + comments); `grep -n "defclass" tests/probe_arc227_stone1_defrecord.rs` returns 0 |
| 5 | SCORE doc gets rename addendum (NOT a rewrite) | SCORE-STONE-227.1.md body unchanged; new "Addendum 2026-05-22 night — Stone 227.1b rename" section appended at END per `feedback_inscription_immutable` |
| 6 | Historical artifacts UNTOUCHED | BRIEF-STONE-227.1.md / EXPECTATIONS-STONE-227.1.md / STONE-227.2-NOTES.md / arc 232 DESIGN.md "Origin" section all retain their original defclass mentions as historical record |
| 7 | All test suites green + HARD CUT verified | `cargo build --release -p wat` 0 errors; `cargo test --release --lib -p wat [--skip 5]` PASS; new probe + arc 226/216/221/143/mvp PASS; wat-edn PASS; clippy clean; `grep -rn "defclass" --include="*.wat" --include="*.rs" .` returns ZERO live-code matches (historical doc/comment references in artifacts table OK) |
| 8 | New SCORE doc written | `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.1b.md` exists; mirrors SCORE-STONE-226.1.md shape; reports calibration |

## Independent prediction (calibration record)

**Target runtime:** 15-45 min Mode A
**Upper bound:** 90 min
**Confidence:** very high

**Rationale:**
- Pure mechanical rename; no new substrate; no new tests; no encoding changes
- ~80-90 total site edits (8 in macro + 3 in stdlib + 69 in probe + 1-3 in SCORE)
- Calibration trend favors faster-than-target — Stone 226.1 (~11 min) was a comparable-scope substrate-addition

**Risks:**
- `git mv` history preservation needs the right ordering (mv first, then edit)
- The 69 mentions in the probe test file include test fn names — renaming both the file AND the fn names within is mechanical but voluminous; sed/perl would help if available
- SCORE addendum must APPEND not rewrite — sonnet may reflexively rewrite; STOP-6 catches this

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Multi-field defrecord (Stone 227.2)
- defprotocol / extend-type (arc 232)
- Doctrine memory entry (orchestrator-side)
- CLIFFNOTES Currently refresh (orchestrator-side)
- holon-rs / wat-edn changes
- Aliases (HARD CUT)

## Honesty deltas accepted

- Sonnet may use sed/perl/python for the 69-site mass edit if it judges the manual edit ergonomically wrong; verify via `which sed perl python3` first if uncertain
- Test fn names may evolve from strict `probe_defrecord_*` if sonnet finds a more honest naming (document in SCORE)
- SCORE addendum prose may differ from sketch; load-bearing point is "appended, not rewritten, with rationale citing the rename"

## Honesty deltas NOT accepted

- ANY `defclass` alias for `defrecord` — STOP-5; HARD CUT
- Rewriting historical artifacts (BRIEF/EXPECTATIONS/SCORE/NOTES of Stone 227.1) — STOP-6; append-only per `feedback_inscription_immutable`
- Touching holon-rs — STOP-4
- Editing the arc 232 DESIGN.md Origin section's defclass mention — STOP-6 (it's historical user dialogue)
- "Pre-existing failure" framing for tests broken by this rename — broken-by-this-stone; honest framing per Stone 221.3 Delta 1a

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** unexpected substrate compile errors
- **STOP-2:** test failure beyond rename consequences
- **STOP-3:** 90 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** alias added (HARD CUT violation)
- **STOP-6:** historical artifact rewritten instead of left/appended
- **STOP-7:** bash discipline — cargo hang from pipes
