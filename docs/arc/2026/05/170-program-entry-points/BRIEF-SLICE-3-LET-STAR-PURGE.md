# Arc 170 slice 3 — let* purge BRIEF (substrate housekeeping + user-side sweep)

**Sonnet.** Purge every `let*` text reference from the repo except the substrate retirement-diagnostic scaffolding (which stays per user direction). User direction 2026-05-11:

> *"let* is illegal - we remove its raised exceptions when arc 109 is completed - every single invocation must fail - clean it all"*
> *"do whatever is necessary to get a subagent to purge let* from my code. this poison infuriates me."*

The substrate walker at `src/check.rs:2376` already rejects every `:wat::core::let*` source-level use fatally (verified via probe). This slice cleans the textual residue: stale comments lying about "fall-through" that no longer exists post-arc-168+163, the registry-entry asymmetry with arc 155's lambda retirement, and ~170 textual references in user-facing docs + wat sources + skills + README.

## Backstory — what arc 168 + arc 163 collectively eliminated

- **Arc 168** renamed `step_let_star → step_let` (runtime.rs:18075). The runtime dispatch arms for `:wat::core::let*` no longer exist — there's no `eval_let_star` or `infer_let_star` to "fall through to let". The fall-through described in arc 154's INSCRIPTION + the comments at check.rs:1636-1647 is **gone**.
- **Arc 163** re-armed the check-time walker (check.rs:2376). Every `:wat::core::let*` source-level token now fires `BareLegacyLetStar` fatally before any runtime dispatch could see it.

So the comments saying "arms keep functional fall-through to `:wat::core::let`" are STALE LIES. The substrate is correctly killing let*. The lies are textual.

## What KEEPS (per user direction + arc 113 precedent)

- `CheckError::BareLegacyLetStar` variant (check.rs:261-263)
- Display impl (check.rs:655-660)
- Diagnostic field emission (check.rs:949-953)
- **Active walker firing** (check.rs:2376-2378) — this is what makes "every single invocation fail"
- `tests/wat_arc154_kill_let_star.rs` — verifies the walker fires correctly
- Historical retirement comments in arc 154/155 INSCRIPTIONs and other immutable inscriptions (FM 11 corollary — "what is inscribed is inscribed")

User direction: *"we remove its raised exceptions when arc 109 is completed"* — that final scaffolding removal is a future arc; not this slice.

## What GETS PURGED

### Bucket A — substrate housekeeping (small, surgical)

1. **`src/special_forms.rs:147`** — registry entry `insert(&mut m, ":wat::core::let*", &["<retired-use-let>"])`. Lambda's registry entry was removed in arc 155 slice 2; let*'s wasn't. Symmetry fix. After: `(help :wat::core::let*)` returns "no such form" (matches lambda's behavior).
2. **`src/check.rs:1636-1665`** — comment block claiming "arms for `:wat::core::let*` keep functional fall-through to `:wat::core::let`" — STALE. Update to reflect post-arc-168+163 reality: no runtime arms exist; walker fires fatal at check time.
3. **`src/runtime.rs`** doc-strings (5 hits at lines 2725, 2887-2888, 3276, 3279, 4439, 18075-18076) — same stale framing in doc-strings. Update each to reflect current reality.

### Bucket B — user-code sweep (the bulk)

For each hit: **transform `let*` → `let` 1:1**. The semantic is identical post-arc-154 (let IS sequential — Clojure-faithful). Code examples become valid wat. Prose discussing "the nested-let* shape" becomes "the nested-let shape." Same meaning, no longer poison.

**Documentation files (~89 hits across 8 files):**
- `docs/USER-GUIDE.md` (39 hits)
- `docs/SERVICE-PROGRAMS.md` (35 hits)
- `docs/WAT-CHEATSHEET.md` (8 hits)
- `docs/CIRCUIT.md` (3 hits)
- `docs/CONVENTIONS.md` (3 hits)
- `docs/CLOJURE-ROSETTA.md` (1 hit)
- `docs/INTENTIONS.md` (1 hit)
- `README.md` (1 hit)

**Wat source comments (6 hits across 3 files):**
- `wat/kernel/services/stdout.wat` (2 hits)
- `wat/kernel/services/stderr.wat` (2 hits)
- `wat/kernel/services/stdin.wat` (2 hits)

All references are in comments like `;; One let* per function per feedback_simple_forms_per_func.` → `;; One let per function per feedback_simple_forms_per_func.` (memory rule name unchanged; comment text updated).

**Spell SKILL.md files (~19 hits across 2 files):**
- `.claude/skills/complectens/SKILL.md` (17 hits)
- `.claude/skills/vocare/SKILL.md` (2 hits)

Spell prose teaching about let* shape; update to let.

**Test files NOT named `wat_arc154_kill_let_star.rs` (~4 hits across 2 files):**
- `tests/wat_arc136_do_form.rs` (1 hit) — review; transform if it's a comment, KEEP if it's a fixture testing legacy rejection
- `tests/wat_arc155_fn_rename.rs` (3 hits) — review; transform comments, KEEP if fixtures test legacy rejection

### Bucket C — KEEP (historical record, FM 11 corollary)

- Everything under `docs/arc/2026/**/INSCRIPTION.md` — immutable historical record
- `tests/wat_arc154_kill_let_star.rs` (24 hits) — test fixtures DEMONSTRATING let* rejection; the let* tokens are load-bearing for the tests
- Comments in substrate that record "Arc 154 retired let*" as historical context — KEEP. Comments that lie about CURRENT BEHAVIOR ("arms keep functional fall-through") — UPDATE.

The judgment call per hit: is the let* text describing CURRENT behavior (lie — update) or HISTORICAL context (truth — keep)?

## Implementation path

### Phase 1 — Substrate housekeeping (sonnet picks order)

1. Remove `let*` registry entry from `src/special_forms.rs:147`
2. Update stale "fall-through" comments in `src/check.rs:1636-1665`
3. Update stale doc-strings in `src/runtime.rs` (5 sites)

### Phase 2 — User-code sweep

Mechanical 1:1 `let*` → `let` text replacement, file by file. For each file in the list above:
- Replace every `let*` occurrence (whether `:wat::core::let*`, bare `let*` in prose, or `let*` in code blocks)
- Verify the prose still reads correctly (no orphaned `let*` references)

### Phase 3 — Verify

```bash
# 1. Workspace stays green
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed (unchanged)

# 2. let* references remain ONLY in Bucket C locations
grep -rln "let\*" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/" | grep -v "tests/wat_arc154_kill_let_star.rs"
# Expected output: empty (or only file paths classified as Bucket C per honest review)

# 3. Probe: let* still fatal at check
echo '(:wat::core::let* [x 1] x)' > /tmp/probe-let-star.wat
target/release/wat /tmp/probe-let-star.wat 2>&1 | head -5
# Expected: BareLegacyLetStar fires with friendly diagnostic
```

## Scope (what's IN)

- All Bucket A substrate fixes
- All Bucket B user-code sweep (~170 hits, all transformations mechanical 1:1)
- SCORE doc with the verification commands above + classification of any judgment-call hits

## Scope (what's OUT)

- Delete BareLegacyLetStar variant / Display / walker firing — explicitly NOT yet per user direction ("we remove its raised exceptions when arc 109 is completed")
- Delete `tests/wat_arc154_kill_let_star.rs` — test exercises walker; stays
- Edit any file under `docs/arc/` — immutable historical record per FM 11 corollary
- Touch `~/.claude/projects/-home-watmin-work-holon/memory/` — memory system is separate; rule-name updates happen separately
- Rename `feedback_simple_forms_per_func.md` memory entry — memory rename is separate
- Anything labeled Path B or Path C from prior discussion — sonnet stops at Path A (lambda precedent symmetry)

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | Substrate housekeeping complete (registry entry removed; stale comments updated in check.rs + runtime.rs) | grep + read |
| B | Documentation sweep complete (8 files; ~89 hits) | grep |
| C | Wat source sweep complete (3 files; 6 hits) | grep |
| D | Spell SKILL.md sweep complete (2 files; ~19 hits) | grep |
| E | Test file judgment-call sweep complete (2 files; ~4 hits) — preserves fixtures where load-bearing | manual review |
| F | Verification passes: workspace 0 failed, grep returns only Bucket C, probe still fatal | full check |

**6 rows.** All must PASS.

## Required reading IN ORDER

1. **`docs/SUBSTRATE-AS-TEACHER.md`** — the discipline doc. This is a textual sweep so substrate diagnostic stream doesn't directly apply, but the four-step recipe + the "diagnostic IS the brief" principle frames the approach.
2. **`docs/arc/2026/05/154-kill-let-star/FOLLOWUP-SUBSTRATE-RETIREMENT.md`** — backstory + retirement-theater pattern context. Note: that doc's framing has been corrected in this BRIEF.
3. **`docs/arc/2026/05/154-kill-let-star/INSCRIPTION.md`** — historical record of arc 154's retirement decisions; immutable (FM 11 corollary)
4. **`src/check.rs:1636-1665`** — the stale comment block to update
5. **`src/check.rs:2376-2378`** — the active walker firing (KEEP; this is what makes "every single invocation fail")
6. **`src/special_forms.rs:147`** — the registry entry to remove (line is the let* entry inside the inserts block)
7. **`tests/wat_arc154_kill_let_star.rs`** — keep all 24 fixtures; they test the walker

## Predicted runtime

**50-80 min sonnet.** Mostly mechanical text replacement; some prose-reading judgment in the docs. The ~170 hits are concentrated in ~10 files; per-file the transformation is uniform.

**Hard cap:** 160 min.

## Constraints (hard)

- **DO NOT** delete `BareLegacyLetStar` variant / Display / Diagnostic field
- **DO NOT** retire the walker firing at check.rs:2376
- **DO NOT** delete `tests/wat_arc154_kill_let_star.rs`
- **DO NOT** touch anything under `docs/arc/` (immutable historical record per FM 11)
- **DO NOT** touch memory system (`~/.claude/`)
- **DO NOT** commit (orchestrator atomic-commits after scoring)
- **DO NOT** use deferral language in SCORE
- **DO NOT** run `git add` / `git commit` from any path other than `/home/watmin/work/holon/wat-rs/` (FM 7)
- Workspace must stay at 0 failed

## Honest delta categories (anticipated)

1. **Judgment-call hits** — any text where transforming `let*` → `let` could change meaning (e.g., prose discussing the historical distinction between let and let*). Most should be mechanical; surface any that require thought.
2. **Bucket C identifications** — any text that records "Arc 154 retired let*" or similar historical context where keeping let* is correct. Surface the call.
3. **Substrate comment rewriting** — the stale "fall-through" comments need rewriting, not deletion. Surface the new wording for review.
4. **Test fixture review** — `wat_arc155_fn_rename.rs` + `wat_arc136_do_form.rs` may need fixtures preserved if they test legacy rejection.
5. **Workspace impact** — any test that flakes or fails due to comment / doc changes (shouldn't, but verify).

## Cross-references

- Arc 154 INSCRIPTION (the retirement that didn't fully kill): `docs/arc/2026/05/154-kill-let-star/INSCRIPTION.md`
- Arc 154 FOLLOWUP doc (this slice's backstory): `docs/arc/2026/05/154-kill-let-star/FOLLOWUP-SUBSTRATE-RETIREMENT.md`
- Arc 155 lambda precedent (the symmetry target): `docs/arc/2026/05/155-fn-rename/INSCRIPTION.md`
- Arc 163 walker re-arm (eliminated the "fall-through"): `docs/arc/2026/05/163-retirement-leftover-audit/`
- Arc 168 step_let rename (eliminated runtime arms): `docs/arc/2026/05/168-let-flat-shape/`
- Substrate-as-teacher discipline: `docs/SUBSTRATE-AS-TEACHER.md`
- FM 11 inscription-immutable: `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 11
- FM 14 surface retirement: `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 14
