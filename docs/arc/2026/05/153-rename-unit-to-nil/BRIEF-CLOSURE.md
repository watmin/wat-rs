# Arc 153 — Slice 2 Closure BRIEF

**Drafted 2026-05-06.** Slice 2 closure for arc 153.

User direction: *"alright - let's get the paper work done"*

## Workspace state pre-spawn

- HEAD: `fd1b3fe` (arc 153 sweeps 1a + 1b atomic commit shipped + pushed)
- Working tree clean
- Workspace: 1988 passed / 0 failed
- 0 source spellings of `:wat::core::unit` remain (verified by grep across wat/, wat-tests/, crates/*, examples/)

## Goal

Close arc 153 cleanly. Two work streams:

1. **Substrate retirement** — retire transitional pieces that
   shipped to support the migration window:
   - Remove `:wat::core::unit` typealias from `src/types.rs` (the
     transitional alias that resolved old spellings during sweep 1b)
   - Retire `walk_type_for_legacy_unit_name` walker body in
     `src/check.rs` per substrate-as-teacher § "Retire the hint
     when its window closes"; leave a retirement comment in the
     section header
   - Retire `walk_type_for_bare`'s `:wat::core::unit` Path-arm
     detection (the body walker arm)
   - Retain `CheckError::BareLegacyUnitName` variant + Display as
     orphaned scaffolding (per arc 113's precedent — variant stays
     for testing/teaching; only the firing body retires)
   - Update arc 153 tests (`tests/wat_arc153_nil_rename.rs`) to
     reflect post-retirement behavior:
     - Type-position-retired test now expects "unknown FQDN type"
       behavior (sub-case: identify what the substrate produces
       when it sees an unrecognized `:wat::core::*` keyword in
       type position, then assert that)
     - Or remove obsolete migration-hint negative tests; tests #1
       (type-pos retired) and #6 (reverse mixed sig retired)
       become unable-to-trigger
   - Workspace stays 0-failed throughout

2. **Closure paperwork:**
   - **INSCRIPTION.md** at `docs/arc/2026/05/153-rename-unit-to-nil/`
     — full closure narrative; cross-reference DESIGN top section;
     pre-INSCRIPTION grep mandatory per FM 11 (no "future arc"
     / "deferred" / etc. language)
   - **058 row** in `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
     — one row capturing arc 153's substrate addition + sweep
     scope; insert in chronological order before
     PERSEVERARE signature
   - **USER-GUIDE entry** in `docs/USER-GUIDE.md` — section
     describing `:wat::core::nil` (singleton type, value-position
     usage, Rust-`()`-equivalent semantics, parallel to
     `:wat::core::Some(t)` / `:wat::core::None` discipline)
   - **WAT-CHEATSHEET update** at `docs/WAT-CHEATSHEET.md` — `nil`
     entry naming the singleton + cross-language analogy
   - **CONVENTIONS update** at `docs/CONVENTIONS.md` if existing
     content references `:wat::core::unit` (likely yes)
   - **Memory pointer** if relevant — update
     `~/.claude/projects/-home-watmin-work-holon/memory/MEMORY.md`
     with arc 153 lesson
   - **Task list updates:**
     - Mark #182 (rename `unit → Unit`) as SUPERSEDED — done by
       arc 153 in different direction (`unit → nil`)

## Constraints

- DO COMMIT + PUSH when all paperwork is done + workspace is
  0-failed. This is a self-contained closure slice; atomic commit
  scope is THIS slice only (sweep 1a + 1b already committed at
  fd1b3fe).
- Workspace must stay 0-failed throughout substrate retirement
  + test updates.
- No grinding. No backwards-compat shims (the arc 153 sweep is
  complete; no transitional infrastructure needed beyond what was
  shipped).
- **MANDATORY pre-INSCRIPTION grep per FM 11:**

```bash
grep -nE "deferred|deferral|future arc|future fix|future cleanup|future polish|future REPL|future-self|TODO|out of scope|when a caller|if pressure|if demand|when demand|when pressure|when needed|when surfaces|surfaces a need|small follow-up|small future|punted|scratch arc|next arc|pending arc|land later|will be|will land|can land later|left for|to be added|to-be-added|not yet implemented|not yet supported|not implemented" docs/arc/2026/05/153-rename-unit-to-nil/INSCRIPTION.md
```

For each match: rewrite to affirmative-out-of-scope language
("Out of arc 153 scope; tracked in arc M (DESIGN at ...)" OR
"Out of arc 153 scope; reason: <X>; not tracked elsewhere").
Run BEFORE committing.

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/153-rename-unit-to-nil/DESIGN.md` — full read
2. `docs/arc/2026/05/153-rename-unit-to-nil/BRIEF-SUBSTRATE.md`
3. `docs/arc/2026/05/153-rename-unit-to-nil/BRIEF-CONSUMERS.md`
4. `docs/SUBSTRATE-AS-TEACHER.md` § "Retire the hint when its
   window closes"
5. `docs/COMPACTION-AMNESIA-RECOVERY.md` § "MANDATORY
   pre-INSCRIPTION grep"
6. `feedback_inscription_immutable.md` (memory) — INSCRIPTION =
   DONE; no deferral language
7. Recent INSCRIPTION exemplars: arc 130, arc 119, arc 109 slice
   1d (closest precedent — same shape: bare-legacy-type retirement)
8. `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
   — see existing rows for shape

## Substrate retirement steps

1. Open `src/types.rs`, find the typealias entry binding
   `:wat::core::unit` to nil. Remove it. (Search for
   `wat::core::unit` to locate.)
2. Open `src/check.rs`, find:
   - `walk_type_for_legacy_unit_name` (or whatever the
     signature-position walker was named in slice 1a)
   - The `walk_type_for_bare` arm that detects `:wat::core::unit`
     Path
   Retire the bodies; leave a retirement comment naming arc 153.
3. Update `tests/wat_arc153_nil_rename.rs`:
   - Tests that asserted `BareLegacyUnitName` fires on
     `:wat::core::unit` may no longer be triggerable (typealias
     + walker retired). Replace with tests that show the new
     behavior (likely an "unknown type" error from the type
     resolver), OR remove the obsolete tests with a comment
     citing the retirement.
4. `cargo test --release --workspace` to verify 0-failed.
5. `cargo test --release --test wat_arc153_nil_rename` to verify
   the updated tests pass.

## Closure paperwork steps

1. Write `docs/arc/2026/05/153-rename-unit-to-nil/INSCRIPTION.md`
   — narrative closure. Include sections:
   - What shipped (substrate + sweep + retirement)
   - Why (cross-language familiarity + marker effect; supersedes
     task #182's `unit → Unit` rename plan)
   - The four questions ran (succinct recap from DESIGN)
   - Cross-references to arcs 109 slice 1d (precedent), 136 (do
     form arc whose slice 2 closure runs after this)
   - Affirmative-out-of-scope footers if any (BUT CHECK THE
     PRE-INSCRIPTION GREP FIRST)
2. Insert 058 changelog row (chronological order before
   PERSEVERARE)
3. Update USER-GUIDE.md (add section on `:wat::core::nil`)
4. Update WAT-CHEATSHEET.md (add `nil` entry)
5. Update CONVENTIONS.md (find any references to
   `:wat::core::unit` in canonical-name discussion; update or
   strikethrough with arc 153 reference)
6. Run pre-INSCRIPTION grep — fix any matches
7. Update task list: mark #182 SUPERSEDED with note "by arc 153
   (rename unit → nil instead)"
8. Commit + push

## Verification

- `cargo test --release --workspace`: 0 failed
- `cargo test --release --test wat_arc153_nil_rename`: all pass
- `grep -rn ':wat::core::unit' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/ src/check.rs src/runtime.rs`: 0 (only fixture-strings in arc 153 tests + retirement-comment references in src/* docs allowed)
- Pre-INSCRIPTION grep: clean (zero matches in INSCRIPTION.md)

## Reporting (~250 words)

1. **Pre-flight crawl confirmation:** all referenced files read.
2. **Substrate retirement summary:** what was retired, what stayed (variant + Display preserved per arc 113 precedent).
3. **Test updates:** what tests changed shape, what tests removed.
4. **Paperwork summary:** which docs updated, key content additions.
5. **Pre-INSCRIPTION grep result:** confirm zero matches; if any matched + were rewritten, name them.
6. **Verification:** workspace 0-failed; arc 153 tests pass.
7. **Path:** Mode A clean / Mode B substrate-retirement-bug /
   Mode C unexpected paperwork interaction.
8. **Honest deltas:** any subtleties in the substrate
   retirement; any docs that needed more update than predicted.

Commit + push when complete. No SCORE doc needed (this IS the closure).

## Time-box

90 minutes wall-clock. ScheduleWakeup at T+90 min.

## Why this matters

User direction 2026-05-06: "alright - let's get the paper work done." Slice 2 closes arc 153 cleanly. After this:
- Task #182 (rename to Unit) marked superseded
- Arc 109 v1 closure unblocks (was blocked on arc 145 + arc 130 +
  arc 144 + arc 153 — most now closed/closing)
- Arc 136 slice 2 closure runs next (do form story closes with
  return positions canonically `:wat::core::nil`)

Mode A clean = arc 153 closes; the foundation gains the `nil` keyword as the singleton-type-and-value canonical form across the substrate; the triplet `nil / Some / None` reads cleanly at every consumer site; the next arcs ride on top.
