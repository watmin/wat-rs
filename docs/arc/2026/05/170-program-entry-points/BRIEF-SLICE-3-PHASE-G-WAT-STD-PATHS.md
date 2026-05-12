# Arc 170 slice 3 Phase G-wat-std-paths BRIEF — kill the phantom paths + fictional directories

**Sonnet.** Fourth and final slice of Phase 1 retirement-theater purge (after G-console `b4ea6a4` + G-stream `2b8c253` + G-lambda-docstrings `b174bfc`). Drains the file-path time-warp: `wat/std/` directory does not exist; `wat-tests/std/` does not exist; `fork-with-forms` is a phantom verb (never existed in the codebase); README's ASCII directory tree at lines 658-680 is complete fiction.

User direction 2026-05-12: drain retirement-theater purge in priority order; you're slice 4 of 4.

This is the last Phase 1 slice. After this, all 48 audit findings are drained (modulo Slice 4 destructive-reap items deliberately deferred per the two-phase plan in INVENTORY).

## Backstory — the three lies on disk

### Lie 1 — `wat/std/` directory (38 hits across 19 files)

The `wat/std/` directory does NOT exist. Files that used to live there moved:
- `wat/std/stream.wat` → `wat/stream.wat` (arc 109 slice 9d)
- `wat/std/hermetic.wat` → `wat/kernel/hermetic.wat`
- `wat/std/sandbox.wat` → `wat/kernel/sandbox.wat`
- `wat/std/test.wat` → `wat/test.wat`
- `wat/std/service/Console.wat` → DELETED (arc 170 slice 1f-η; ambient :wat::kernel::println/eprintln/readln replaced)

`wat-tests/std/` does NOT exist either. Files at `wat-tests/` root + thematic subdirs (`wat-tests/kernel/services/`, `wat-tests/holon/`, `wat-tests/core/`, `wat-tests/edn/`).

### Lie 2 — `fork-with-forms` phantom verb (3 hits in README.md)

`fork-with-forms` does not exist anywhere in the codebase. The canonical verb is `:wat::kernel::fork-program-ast` per arc 104a (verb rename was completed). Plus `wait-child` references — hermetic.wat no longer uses wait-child per arc 105c.

### Lie 3 — README ASCII directory tree (lines 658-680)

Shows wat/std/ subdir with stream.wat, hermetic.wat, test.wat, service/Console.wat — ALL WRONG (moved/deleted). Shows wat-tests/std/ with Subtract/Circular/Reject/Sequential/Trigram/test/stream.wat — directory doesn't exist; files at wat-tests/ root. Shows generic `tests/wat_run_sandboxed{,_ast}.rs` — those test files may not exist.

## Real directory truth (verified disk state 2026-05-12)

```
wat/
├── core.wat edn.wat holon.wat list.wat runtime.wat stream.wat test.wat
├── holon/
│   └── Amplify.wat Bigram.wat Circular.wat Log.wat Ngram.wat Project.wat
│       ReciprocalLog.wat Reject.wat Sequential.wat Subtract.wat Trigram.wat
└── kernel/
    ├── channel.wat hermetic.wat sandbox.wat
    └── services/
        └── stderr.wat stdin.wat stdout.wat

wat-tests/
├── service-template.wat stream.wat test.wat time.wat tmp-*.wat
├── core/
│   └── option-expect.wat result-expect.wat struct-to-form.wat
├── edn/
│   └── render.wat roundtrip.wat
├── holon/
│   └── (many test wats — get full list during sweep)
└── kernel/services/
    └── ambient-stdio.wat
```

## What KEEPS (Bucket C/D)

- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 8 "Real incident, 2026-05-02: Sonnet created wat/std/ast.wat..." — historical incident reference. KEEP literal phrasing OR carefully reword to preserve the incident context while noting wat/std/ is gone.
- Historical context comments in substrate (src/check.rs, src/types.rs, etc.) that record "X moved from wat/std/ to wat/kernel/" — KEEP if structured as historical record.
- Anything under `docs/arc/` — never touched (FM 11).

## What GETS PURGED

### Bucket A — active claims about live paths

**README.md (3 hits + 3 fork-with-forms = 6 hits):**
- `:501` — "Every file under `wat/std/` is baked into the binary..." — directory doesn't exist; rewrite to "Every file under `wat/`" or list the actual subdirs.
- `:658-680` — ASCII directory tree FICTION; rewrite to match real disk truth (template above).
- `:98` — "fork-with-forms` + `wait-child`" phantom verb. Replace with `fork-program-ast`. Drop wait-child references (hermetic.wat no longer uses it).
- `:236-238` — "`wat/std/hermetic.wat` on top of `:wat::kernel::fork-with-forms`" — TWO lies in one line. Fix path + verb.
- `:238` — "fork-with-forms`, `wait-child`" — phantom verb list.

**docs/USER-GUIDE.md (3 hits):**
- `:3471-3472` — "wat stdlib define in `wat/std/sandbox.wat`" / "wat stdlib define in `wat/std/hermetic.wat`" — fix paths to `wat/kernel/sandbox.wat` / `wat/kernel/hermetic.wat`.
- 3rd hit — find via grep.

**wat-tests/README.md (2 hits):**
- `:80, 86` — "See `wat/std/hermetic.wat`" → `wat/kernel/hermetic.wat`.

**docs/ZERO-MUTEX.md (1 hit per audit):**
- `:313` — "Reference: `wat-rs/wat/std/service/Console.wat`" — file DELETED. Replace with note about ambient kernel stdio replacement (point to `wat/kernel/services/{stdin,stdout,stderr}.wat` or to examples/console-demo).

**docs/README.md (1 hit):**
- Verify content; transform if Bucket A.

### Bucket B — comments / docstrings about CURRENT layout (substrate + docs files with multiple hits)

**src/check.rs (9 hits) — most volume:**
Triage each: historical context comment (Bucket C, keep) vs. current-state claim about live paths (Bucket B, update).

**src/types.rs (3 hits), src/stdlib.rs (3 hits), src/runtime.rs (2 hits), src/special_forms.rs (1 hit), src/freeze.rs (1 hit), src/sandbox.rs (1 hit), src/spawn.rs (1 hit):**
Same triage. Likely mix of Bucket C (historical "X moved from wat/std/...") and Bucket B (stale "wat/std/..." paths in active text).

**wat/test.wat (2 hits), wat/kernel/hermetic.wat (2 hits), wat/kernel/sandbox.wat (1 hit):**
Self-referential comments. Each file mentions its own move. Likely Bucket C historical context (each file knowing where it came from). Verify per hit.

**tests/wat_arc113_cross_fork_cascade.rs (1 hit), tests/wat_core_cond.rs (1 hit):**
Likely test text comments. Triage per hit.

**crates/wat-telemetry-sqlite/src/auto.rs (1 hit):**
Single hit. Triage.

### Bucket C — historical context with careful triage

**docs/COMPACTION-AMNESIA-RECOVERY.md (4 hits) — discipline doc, careful:**
- `:522` — "creating a new file under `wat/std/` or adding new symbols..." — FM 8 signature. The literal `wat/std/` phrasing is now misleading (directory doesn't exist), but the FM 8 lesson is still real (`:wat::std::*` namespace is dying). Recommended: reword "creating a new file under `wat/std/`" → "creating a new file under any `wat/std/*` location (note: this directory is now gone; if a file claims to be there, it's stale)" OR drop the `wat/std/` phrasing entirely and say "adding new symbols to `:wat::std::*` namespace".
- `:526` — "NEVER add to `wat/std/`" — same shape; reword.
- `:530-531` — "Sonnet created `wat/std/ast.wat`... User: 'remove wat/std/ast.wat'" — HISTORICAL INCIDENT QUOTE. The user's actual words are preserved as direct quotes; the incident happened. KEEP literal quotes (FM 11 corollary: historical record). But may add a parenthetical note about the directory's current absence.

Surface the COMPACTION-AMNESIA-RECOVERY.md edits for orchestrator review BEFORE commit — this is our discipline doc; precision matters.

### Bucket D — none (no scaffolding-style wat/std/ in src/check.rs's BareLegacy*)

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-3-PHASE-G-LAMBDA-DOCSTRINGS.md`** + `SCORE-SLICE-3-PHASE-G-LAMBDA-DOCSTRINGS.md` (commit `b174bfc`) — immediate precedent
2. **`docs/arc/2026/05/170-program-entry-points/RETIREMENT-THEATER-INVENTORY.md`** — full audit context
3. **README.md:655-685** — the ASCII directory tree FICTION to rewrite
4. **Real disk truth** (already captured in this BRIEF):
   - `wat/` structure: core.wat edn.wat holon.wat list.wat runtime.wat stream.wat test.wat + wat/holon/ + wat/kernel/ + wat/kernel/services/
   - `wat-tests/` structure: flat root files + wat-tests/core/ + wat-tests/edn/ + wat-tests/holon/ + wat-tests/kernel/services/
5. **`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 8 (line 522 area)** — the discipline doc context for the careful triage
6. **`docs/SUBSTRATE-AS-TEACHER.md`** — Pattern 3 doctrine

## Implementation path

### Phase 1 — High-value rewrites (30-45 min)

1. **README.md:658-680 ASCII tree rewrite** — full reconstruction matching real disk truth. Use the template above. Surface final tree in SCORE for orchestrator review.
2. **README.md:501 "Every file under wat/std/..."** — rewrite to reflect current layout.
3. **README.md:98, 236-238 fork-with-forms phantom + wait-child** — replace phantom verb with `fork-program-ast`; remove or correct wait-child references.
4. **docs/COMPACTION-AMNESIA-RECOVERY.md § FM 8 (4 hits)** — careful triage. Preserve historical quotes (FM 11 corollary); reword teaching prose to current reality. Surface final wording.

### Phase 2 — Path corrections (20-30 min)

Per file, per hit, transform:
- `wat/std/sandbox.wat` → `wat/kernel/sandbox.wat`
- `wat/std/hermetic.wat` → `wat/kernel/hermetic.wat`
- `wat/std/stream.wat` → `wat/stream.wat`
- `wat/std/test.wat` → `wat/test.wat`
- `wat/std/service/Console.wat` → DELETED; either drop the reference or note the deletion + point to replacement
- Wat-tests/std/ — point to actual wat-tests/ structure

Files: docs/USER-GUIDE.md, wat-tests/README.md, docs/ZERO-MUTEX.md, docs/README.md, crates/wat-telemetry-sqlite/src/auto.rs, tests/wat_arc113_cross_fork_cascade.rs, tests/wat_core_cond.rs

### Phase 3 — Substrate/wat-source comment triage (15-20 min)

Per file, per hit, Bucket classification:
- `src/check.rs` (9 hits) — heavy triage
- `src/types.rs` (3), `src/stdlib.rs` (3), `src/runtime.rs` (2), `src/special_forms.rs` (1), `src/freeze.rs` (1), `src/sandbox.rs` (1), `src/spawn.rs` (1)
- `wat/test.wat` (2), `wat/kernel/hermetic.wat` (2), `wat/kernel/sandbox.wat` (1) — self-referential moves

For each: is the comment recording WHERE THE FILE USED TO BE (Bucket C — keep as historical record OR add ", now at <new path>" if helpful) OR claiming the file IS CURRENTLY at wat/std/X (Bucket B — update)?

### Phase 4 — Verify

```bash
# 1. Workspace stays green
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed (unchanged)

# 2. Final grep — wat/std/ hits remain ONLY in Bucket C
grep -rln "wat/std/" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: only files with Bucket C historical context (list in SCORE)

# 3. fork-with-forms grep
grep -rn "fork-with-forms" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: empty (phantom verb fully gone)

# 4. wat-tests/std/ grep
grep -rn "wat-tests/std/" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: empty
```

## Scope (what's IN)

- All 38 `wat/std/` hits triaged + transformed (Bucket A/B) or preserved (Bucket C with rationale)
- All 3 `fork-with-forms` phantom hits transformed to `fork-program-ast`
- README.md ASCII tree rewritten to match disk truth
- COMPACTION-AMNESIA-RECOVERY.md § FM 8 carefully updated (preserving historical quotes)
- Workspace stays at 2205 / 0 failed

## Scope (what's OUT)

- Anything under `docs/arc/` (FM 11)
- `~/.claude/` memory system
- `eval_kernel_wait_child` dead Rust fn — deferred to Slice 4 (substrate retirement; folds in there)
- Phase G-fork-program-walker-notes — deferred to AFTER Slice 4 per INVENTORY (notes are fully accurate only post-stdlib-arm retirement)
- New substrate features or walker mints
- Touching arc 170 DESIGN docs or TIERS.md (locked architecture)

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | README.md ASCII tree (658-680) rewritten to match real disk truth | manual review (sonnet surfaces new tree in SCORE) |
| B | All `fork-with-forms` hits in README replaced with `fork-program-ast` | grep returns zero for `fork-with-forms` |
| C | `wat/std/` path lies transformed across docs + substrate; Bucket C historical preserved with rationale | per-file inventory in SCORE |
| D | `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 8 carefully updated; historical quotes preserved | surface wording for orchestrator review |
| E | `cargo check --release` green; workspace 2205 / 0 failed | full test run |
| F | Final grep returns ONLY Bucket C files (historical context, with each entry justified in SCORE) | grep |

**6 rows.** All must PASS.

## Predicted runtime

**60-90 min sonnet.** Bigger than G-stream (more files; ASCII tree rewrite; discipline-doc triage); smaller than G-console (no walker mint).

**Hard cap:** 180 min (2×).

## Constraints (hard)

- DO NOT touch anything under `docs/arc/` (FM 11 immutable)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT erase historical user-quote context in COMPACTION-AMNESIA-RECOVERY.md (the "Sonnet created wat/std/ast.wat" + user response is direct quote — preserve verbatim or in clearly-marked quote block)
- DO NOT use --no-verify or skip hooks
- DO NOT add new walker / substrate features
- DO NOT touch eval_kernel_wait_child (deferred to Slice 4)
- DO NOT add walker-fires notes to fork-program docs (deferred to G-fork-program-walker-notes post-Slice-4)
- Workspace must stay at 2205 / 0 failed

## Honest delta categories (anticipated)

1. **README.md ASCII tree wording** — surface final tree for orchestrator review (highest-stakes single edit; will be read by every new developer)
2. **COMPACTION-AMNESIA-RECOVERY.md § FM 8 rewording** — surface final phrasing for review; preserves the FM 8 lesson while correcting current reality
3. **Substrate comment Bucket triage** — list each hit's classification (Bucket B/C) with rationale; surface judgment calls
4. **Bonus catches** — additional files beyond audit's 13 (expected per G-console / G-stream / G-lambda-docstrings pattern; 38 hits already discovered)
5. **Self-referential wat-file comments** (`wat/test.wat`, `wat/kernel/hermetic.wat`, `wat/kernel/sandbox.wat`) — each file's own historical move; surface treatment per file
6. **Anything unexpected** — particularly any pre-existing source-level `:wat::std::*` use the workspace would surface

## Cross-references

- `b174bfc` — Phase G-lambda-docstrings (most recent precedent)
- `2b8c253` — Phase G-stream (pure doc sweep pattern)
- `b4ea6a4` — Phase G-console (walker-mint pattern; not applicable here)
- `daa973d` — let* purge (original purge pattern)
- `RETIREMENT-THEATER-INVENTORY.md` — the audit
- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 3 doctrine
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 11 + § FM 14 — discipline doctrine
- Arc 104a INSCRIPTION — fork-with-forms → fork-program-ast verb rename
- Arc 105c INSCRIPTION — hermetic.wat retired from wait-child
- Arc 109 slice 9d INSCRIPTION — wat/std/stream.wat → wat/stream.wat move
- Arc 170 slice 1f-η INSCRIPTION — Console.wat deletion
