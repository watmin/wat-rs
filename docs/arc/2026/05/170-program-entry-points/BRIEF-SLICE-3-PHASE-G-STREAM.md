# Arc 170 slice 3 Phase G-stream BRIEF — sweep `:wat::std::stream::*` doc rot

**Sonnet.** Second slice of the retirement-theater purge. Closes the `:wat::std::stream::*` namespace doc rot. Walker already fires correctly (verified probe: BareLegacyStreamPath emits "legacy stream path ...is retired (arc 109 slice 9d); canonical form is ':wat::stream::*'"). Users get the diagnostic. But user-facing docs still teach the OLD namespace.

User direction 2026-05-12 (yesterday): drain retirement-theater purge in priority order; console first (`b4ea6a4` shipped), stream next.

See `RETIREMENT-THEATER-INVENTORY.md` for full audit context. This BRIEF scopes to ONLY `:wat::std::stream::*` namespace + the related Stream<T> typealias inner-type lie + file-path inside the stream typealias table.

## Backstory — what arc 109 slice 9d shipped + the gap

**Arc 109 slice 9d** (in completed tasks) graduated the stream stdlib:
- Namespace: `:wat::std::stream::*` → `:wat::stream::*`
- File: `wat/std/stream.wat` → `wat/stream.wat`
- `BareLegacyStreamPath` walker enforces (variant + Display + Diagnostic + walker firing — verified at src/check.rs:344-357, 682, 2605-2640, prefix-match `:wat::std::stream::`)

**Substrate is clean.** Walker fires fatal on any source-level `:wat::std::stream::*` with helpful "rename to `:wat::stream::*`" diagnostic.

**The gap:** ~25 doc hits across 4 user-facing files still teach the old namespace. Plus CONVENTIONS.md:642-644 typealias table has THREE wrongs in three rows (namespace + inner type + file path).

## What KEEPS (Bucket C historical, Bucket D scaffolding)

- `src/check.rs:344-357, 682, 1007, 1704-1708, 2658-2674` — variant + Display + Diagnostic + walker firing infrastructure. The legacy namespace string is intentional (teaches the migration). Bucket D scaffolding. KEEP.
- `docs/SUBSTRATE-AS-TEACHER.md:225` — historical example listing the migration: `":wat::std::stream::*" → ":wat::stream::*" (9d)`. Bucket C. KEEP.
- Historical INSCRIPTIONs under `docs/arc/` — never touched.

## What GETS PURGED (Bucket B textual sweep)

### Doc files (25 hits across 4 files)

| File | Hits | Notes |
|---|---|---|
| `docs/USER-GUIDE.md` | 20 | Tier-4 stdlib description (~586); streaming section §11+ (~2052, 2092-2099, 2112, 2142); reference table (~3487-3495). Mechanical 1:1 `:wat::std::stream::*` → `:wat::stream::*` except tier list (judgment). |
| `docs/CONVENTIONS.md` | 3 | Typealias table at 640-650. Three rows with **triple wrong** content: namespace + inner type + file path. |
| `wat-scripts/README.md` | 1 | Line 12 — code example uses `:wat::std::stream::*` combinators. |
| `README.md` | 1 | Lines 520-525 — stream-stdlib feature description listing `:wat::std::stream::Stream<T>` typealias + verbs. |

### The triple-wrong CONVENTIONS.md typealias table (640-650)

Current (lying):
```
| `:wat::std::stream::Stream<T>`        | `:(Receiver<T>,ProgramHandle<()>)`              | `wat/std/stream.wat` |
| `:wat::std::stream::ChunkStep<T>`     | `:(Vec<T>,Vec<Vec<T>>)`                         | `wat/std/stream.wat` |
| `:wat::std::stream::KeyedChunkStep<K,T>` | `:((Option<K>,Vec<T>),Vec<Vec<T>>)`          | `wat/std/stream.wat` |
```

Truth from `wat/stream.wat:49`:
```
:wat::stream::Stream<T>
:(wat::kernel::Receiver<T>, wat::kernel::Thread<wat::core::nil, wat::core::nil>)
```

Required corrections (per row):
- **Namespace**: `:wat::std::stream::*` → `:wat::stream::*` (FQDN canonical paths)
- **Stream<T> inner type**: `ProgramHandle<()>` → `Thread<wat::core::nil, wat::core::nil>` (post arc 114 — `ProgramHandle` is dead; Thread<I,O> is the canonical handle type)
- **File path**: `wat/std/stream.wat` → `wat/stream.wat` (arc 109 slice 9d file move)
- **ChunkStep + KeyedChunkStep**: namespace + file path only (inner types `Vec<T>,Vec<Vec<T>>` etc. are correct shape; only the FQDN needs `:wat::core::Vec<...>` if that's the canonical form — verify against actual typealias bodies in `wat/stream.wat`)

### The USER-GUIDE.md tier-list judgment (~586)

Current:
> 4. **Stdlib plumbing** (`:wat::std::*`) — non-algebra conveniences written in wat: stream combinators (`:wat::std::stream::*`), the hermetic-test wrapper. Each expressible in wat on top of core + kernel. (The former Console stdio service retired in arc 109 § kill-std / arc 170 slice 1f-η; see § 11 for the ambient kernel trio that replaces it.)

Issue: Teaches `:wat::std::*` as the namespace for "stream combinators (`:wat::std::stream::*`)" — but stream graduated OUT of `:wat::std::*` to its own top-level tier `:wat::stream::*` per arc 109 slice 9d. The "every substrate concern earns its own top-level tier; `:wat::std::*` empties out" framing (in the walker's diagnostic) should reflect here.

**Judgment for sonnet:**
- Update the tier description to reflect the new structure — stream is no longer "stdlib plumbing"; it's its own tier `:wat::stream::*`
- Note what's actually left in `:wat::std::*` (if anything user-facing remains). The directory `wat/std/` no longer exists; the entire namespace appears to be on its way out per arc 109's mission "FQDN every substrate-provided symbol; flatten std".
- This may require a small restructure of the tier list. Surface the proposed rewording before finalizing.

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-3-PHASE-G-CONSOLE.md`** + **`SCORE-SLICE-3-PHASE-G-CONSOLE.md`** (commit `b4ea6a4`) — the precedent slice; same purge pattern
2. **`docs/arc/2026/05/170-program-entry-points/RETIREMENT-THEATER-INVENTORY.md`** — full audit context
3. **`docs/SUBSTRATE-AS-TEACHER.md`** — Pattern 3 (symbol migration); arc 109 slice 9d is the canonical example
4. **`src/check.rs:344-357, 682, 1007, 1704-1708, 2658-2674`** — the BareLegacyStreamPath scaffolding (READ ONLY; don't modify)
5. **`wat/stream.wat:40-90`** — the canonical typealias bodies (use these to fix CONVENTIONS.md)
6. **`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 14** — the discipline

## Implementation path

### Phase 1 — Tight namespace sweep (15-20 min, mechanical)

For each file in the doc-hits table above, replace every `:wat::std::stream::` token with `:wat::stream::`. Exception: keep historical references that name "what was retired and replaced by what" (Bucket C — same shape as docs/SUBSTRATE-AS-TEACHER.md:225). For each hit, judgment: is the text DESCRIBING current usage (Bucket B — transform) or RECORDING the migration (Bucket C — keep)?

### Phase 2 — CONVENTIONS.md typealias table fix (10-15 min, careful)

Three rows; verify each correction against `wat/stream.wat` actual typealias bodies. Surface the final table rows in SCORE for orchestrator review before commit.

### Phase 3 — USER-GUIDE.md tier list update (5-10 min, judgment)

Restructure the tier-4 description to reflect that stream graduated out. Surface proposed wording in SCORE.

### Phase 4 — Verify

```bash
# 1. Workspace stays green
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: 2205 passed / 0 failed (unchanged)

# 2. Probe: walker still fires
echo '(:wat::std::stream::map x y)' > /tmp/probe-stream.wat
./target/release/wat /tmp/probe-stream.wat 2>&1 | head -10
# Expected: BareLegacyStreamPath fires with friendly diagnostic naming :wat::stream::* canonical

# 3. Final grep — :wat::std::stream:: hits remain ONLY in Bucket C/D
grep -rln "wat::std::stream" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: src/check.rs (Bucket D scaffolding) + docs/SUBSTRATE-AS-TEACHER.md (Bucket C historical)
```

## Scope (what's IN)

- 4 user-facing files swept (USER-GUIDE, CONVENTIONS, wat-scripts/README, README)
- CONVENTIONS.md typealias table triple-fix
- USER-GUIDE.md tier-list restructure
- Workspace stays at 2205 / 0 failed

## Scope (what's OUT)

- Other retirement-theater items (lambda docstrings, wat/std/ phantom paths broadly, fork-program walker notes) — separate Phase G-* slices
- The broader `wat/std/` directory phantom-path sweep — separate Phase G-wat-std-paths (would be too much overlap with Phase G-stream's careful CONVENTIONS.md edits to bundle)
- ANY substrate modification — walker is already correct; this is pure doc work
- Anything labeled INSCRIPTION-class — this is a slice
- Touching docs/SUBSTRATE-AS-TEACHER.md:225 (Bucket C historical context)
- Touching docs/arc/ (FM 11 immutable)
- Touching ~/.claude/ memory system

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `docs/USER-GUIDE.md` sweep complete (20 hits including tier-4 list + streaming section + reference table) | grep + read |
| B | `docs/CONVENTIONS.md` typealias table corrected (namespace + Stream<T> inner type + file path; 3 rows) | manual review of new wording |
| C | `wat-scripts/README.md` + `README.md` sweep complete (1 hit each) | grep |
| D | `cargo check --release` green; workspace 2205 / 0 failed | full test |
| E | Probe: `(:wat::std::stream::map x y)` still fires BareLegacyStreamPath with `:wat::stream::*` canonical teaching | manual probe |
| F | Final grep returns ONLY src/check.rs (Bucket D scaffolding) + docs/SUBSTRATE-AS-TEACHER.md (Bucket C) | grep |

**6 rows.** All must PASS.

## Predicted runtime

**30-50 min sonnet.** Smaller than G-console because no walker mint needed; bigger than pure mechanical because of the triple-wrong CONVENTIONS.md table + USER-GUIDE.md tier-list judgment.

**Hard cap:** 100 min (2×).

## Constraints (hard)

- DO NOT touch any file under `docs/arc/` (FM 11 immutable)
- DO NOT touch `docs/SUBSTRATE-AS-TEACHER.md:225` (Bucket C)
- DO NOT touch `src/check.rs` BareLegacyStreamPath scaffolding (Bucket D)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT broaden scope to other `wat/std/` paths outside the stream typealias table (separate Phase G-wat-std-paths)
- DO NOT use --no-verify or skip hooks
- Workspace must stay at 2205 / 0 failed

## Honest delta categories (anticipated)

1. **CONVENTIONS.md typealias rewording** — surface the final 3 rows for orchestrator review (especially the `Thread<wat::core::nil, wat::core::nil>` shape — verify it's canonical, not `Thread<:nil,:nil>` or similar)
2. **USER-GUIDE.md tier-list restructure** — surface the proposed new tier-4 description for review; this is the most judgment-heavy edit
3. **Bucket C identifications** — any text where transforming would erase legitimate historical context
4. **Workspace impact** — should be zero; surface anything unexpected
5. **Anything pre-existing source-level `:wat::std::stream::*`** — would surface in workspace tests if any leaked (substrate walker would fire); STOP and report if so (means slice 9d's sweep missed something)

## Cross-references

- `RETIREMENT-THEATER-INVENTORY.md` — the audit
- `b4ea6a4` — Phase G-console (the precedent purge SCORE shape)
- `daa973d` — let* purge (the original purge pattern at scale)
- `SCORE-SLICE-1F-ETA.md` — Console retirement that the audit framework is patterned on
- `docs/SUBSTRATE-AS-TEACHER.md` — Pattern 3 doctrine
- `wat/stream.wat:40-90` — canonical typealias bodies (source of truth for CONVENTIONS.md fix)
