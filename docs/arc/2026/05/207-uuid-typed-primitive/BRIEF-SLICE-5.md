# BRIEF — Arc 207 Slice 5: closure paperwork

**Predecessors:** Slices 1-4 SHIPPED. Tip at `3865569`. Typed `:wat::core::Uuid` ships end-to-end; arc 206 namespace verbs retired; consumer ripple complete; all tests green; workspace baseline preserved (3 pre-existing failures unchanged).

**Scope: pure paperwork — no source files touched.** Three artifacts:

1. **`docs/arc/2026/05/207-uuid-typed-primitive/INSCRIPTION.md`** (NEW) — arc 207's closure record
2. **`docs/arc/2026/05/207-uuid-typed-primitive/DESIGN.md`** (UPDATE) — status header changes OPEN → CLOSED; slice table marks all 5 slices SHIPPED with commit refs
3. **`/home/watmin/work/holon/holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`** (APPEND) — wat-rs arc 207 row inscribing the typed-Uuid promotion + the discipline lesson forward

After this slice ships: arc 207 closed; arc 170 + arc 203 + lab reconstruction all unblock.

## INSCRIPTION.md required content

Structure (mirror prior arc INSCRIPTIONs for consistency — see `docs/arc/2026/05/206-uuid-substrate-promotion/INSCRIPTION.md` for the immediate-prior format):

### Status header
- `**Status:** SHIPPED 2026-05-17.` + brief one-line summary of what shipped

### What arc 207 gave the substrate
- Typed `:wat::core::Uuid` primitive + 5 verbs + nil-uuid (6 entries) using Type/verb naming per `feedback_wat_namespace_principle`
- `Value::wat__core__Uuid(uuid::Uuid)` variant (Pattern B; mirrors Instant/Duration/keyword)
- EDN `#uuid "..."` reader-literal end-to-end roundtrip (read + write arms in edn_shim)
- `hashmap_key` arm for typed Uuid (added slice 3 in-scope when telemetry test surfaced demand)
- Retired arc 206's `:wat::core::uuid::*` namespace verbs entirely (no parallel keep-both per `feedback_refuse_easy_solutions`)
- Telemetry alias `:wat::telemetry::uuid::v4` retargets to typed `:wat::core::Uuid/v4` (return type :String → :Uuid is honest breaking change for the secret-witness pattern)
- Arc 203 demos (capability + process tier + single-user proof) rippled to typed surface; secret-witness security model now type-honest in test setup

### Slices (table)
- Slice 1 — substrate audit + shape decision (`SCORE-SLICE-1.md`; option (c) Value variant per Pattern B)
- Slice 2 — mint type + 6 verbs + edn_shim arms (`SCORE-SLICE-2.md`; 10 tests pass)
- Slice 3 — retire namespace verbs + retarget telemetry alias + hashmap_key arm in-scope (`SCORE-SLICE-3.md`)
- Slice 4 — consumer ripple + USER-GUIDE rewrite + Mode D latent gap fix in-scope (`SCORE-SLICE-4.md`)
- Slice 5 — this closure (`SCORE-SLICE-5.md`)

### Substrate touchpoints (final inventory)
- All files actually touched across slices 2-4 with commit refs
- Pull from each SCORE doc's file:line list; tabulate

### Out of arc 207's scope (affirmatively, NOT deferral)
- **Other UUID versions (v1/v3/v7/v8)** — arc 206's DESIGN documented the 3-step mechanical pattern; one-line edit per version when consumer surfaces
- **`#uuid "..."` as wat-syntax-level reader literal** — EDN read path already covers reader-literal semantics; wat-syntax change is parser-layer concern; no consumer pressure today
- **`Uuid/version` extraction** — UUID is identifier; construction technique invisible to consumers; no consumer pressure today
- **`uuid?` predicate verb** — substrate dispatch handles type-predicates polymorphically
- **`values_compare` arm (Uuid ordering)** — UUIDs are identifiers not ordinals; same shape as keyword/Enum/Struct
- **Lenient `Uuid/from-string` parsing** — strict canonical-only matches EDN-layer strictness; lenient if real consumer surfaces

### Discipline lessons inscribed (the load-bearing part)

THIS IS THE FORWARD-CORRECTION OF ARC 206. Arc 206 INSCRIPTION-SLICE-2 named typed `:wat::core::Uuid` as "out of scope; no current consumer demands it." That framing was deferral dressed in affirmative language. The consumer pressure was on disk:

- `src/edn_shim.rs:404` rejected `Edn::Uuid(_)` with `EdnReadError::Other(...)` — a substrate arm literally waiting to be filled
- `:wat::core::uuid::v5` runtime-panicked on invalid `:String` namespace argument — a documented foot-gun in USER-GUIDE

User direction 2026-05-17: *"deferral is a dishonest term."* Arc 207 forward-corrects the discipline failure. Carry-forward:

> **Before naming anything "out of scope; no consumer demands it," grep the substrate for arms / errors / panics that name the missing type. If they exist, that IS the consumer pressure; the type belongs in scope.**

Also inscribed: the substrate-as-teacher cascade ran cleanly across slice 3 (telemetry test surfaced hashmap_key gap → sonnet fixed in-scope) and slice 4 (subprocess `readln` surfaced edn_to_typed_value_inner gap → sonnet fixed in-scope). The pattern `feedback_no_known_defect_left_unfixed` worked at both slices: when a consumer's actual code path surfaces a substrate gap, the right move is in-scope fix at the slice that surfaced it, not punt to a future arc.

### Cross-references
- Arc 092 (initial uuid mint in wat-edn)
- Arc 206 (substrate promotion + telemetry de-dup; INSCRIPTION-SLICE-2 + SLICE-3; immutable)
- Arc 203 (capability pattern; consumer ripple target)
- Arc 170 (unblocks closure)
- INTERSTITIAL § seven-greats convergences (wat-MCP entry; lineage entry)
- `feedback_inscription_immutable`, `feedback_refuse_easy_solutions`, `feedback_no_known_defect_left_unfixed`, `feedback_wat_namespace_principle`

## DESIGN.md update

Single-section change at the top of file:
- Status header: `OPEN 2026-05-17` → `CLOSED 2026-05-17 — INSCRIPTION at INSCRIPTION.md`
- Slice table: mark all 5 slices SHIPPED with their commit refs (sonnet pulls from git log)

NOTHING else in DESIGN changes. The forward-corrections from slice 1 audit are already inscribed in DESIGN per FM 13 (DESIGN is living).

## 058 changelog row (lab repo)

Append at the bottom of `/home/watmin/work/holon/holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md` (before the closing `*these are very good thoughts.*` signoff).

Format: mirror the existing arc 200/201/202/206 rows. The row should:
- Date: 2026-05-17
- Title: `**wat-rs arc 207 — :wat::core::Uuid typed primitive promotion**`
- Brief summary: what shipped + the discipline-lesson forward-correction
- Cite slice commit refs (1-4 + closure)
- Cite arc 206 as forward-corrected (arc 206 INSCRIPTIONs stay immutable; arc 207 corrects the deferral pattern)
- Close with `Full INSCRIPTION at wat-rs/docs/arc/2026/05/207-uuid-typed-primitive/INSCRIPTION.md. | wat-rs arc 207 |`

## FM 11 pre-INSCRIPTION grep (MANDATORY)

Before committing the INSCRIPTION:

```
grep -nE "deferred|deferral|future arc|future fix|future cleanup|future polish|future REPL|future-self|TODO|out of scope|when a caller|if pressure|if demand|when demand|when pressure|when needed|when surfaces|surfaces a need|small follow-up|small future|punted|scratch arc|next arc|pending arc|land later|will be|will land|can land later|left for|to be added|to-be-added|not yet implemented|not yet supported|not implemented" docs/arc/2026/05/207-uuid-typed-primitive/INSCRIPTION.md
```

Expected: ZERO matches for any deferral pattern. The "out of scope (affirmatively)" section uses affirmative language ("Arc 207 intentionally does NOT cover X because <architectural reason>") not deferral language ("future arc when X surfaces"). If any match surfaces, rewrite to affirmative form per FM 11 doctrine.

The grep is the discipline checkpoint. Trust the grep, not the felt sense. Run it BEFORE commit.

## HARD constraints

- DO NOT touch source files (`*.rs`, `*.wat`, `*.toml`). Pure paperwork slice.
- DO NOT touch arc 206 INSCRIPTIONs (immutable per `feedback_inscription_immutable`).
- DO NOT amend slice 1-4 SCORE docs (immutable historical record).
- DO NOT amend prior 058 changelog rows in lab repo (append-only).
- DO NOT commit. Orchestrator commits atomically per repo (wat-rs commit + lab commit; two atomic commits).
- DO NOT use `--no-verify` / `--no-gpg-sign`.
- cwd `/home/watmin/work/holon/wat-rs/` for wat-rs work; use `git -C /home/watmin/work/holon/holon-lab-trading` for lab git op (do NOT cd).
- Never `.claude/worktrees/`.

## STOP triggers

1. **FM 11 grep returns ANY match** — rewrite to affirmative form before commit
2. **Any prior arc 206 INSCRIPTION file appears modified** in `git status` — surface; you must NOT touch them
3. **058 changelog row format ambiguity** — read prior rows (arc 200/201/202/206) to confirm format; if pattern is unclear, surface

## SCORE methodology

`docs/arc/2026/05/207-uuid-typed-primitive/SCORE-SLICE-5.md` rows (atomic YES/NO):

| Row | Evidence |
|---|---|
| A — INSCRIPTION.md written with required sections (status, what shipped, slices, touchpoints, out-of-scope-affirmative, discipline lesson, cross-refs) | Section headers cited |
| B — FM 11 pre-INSCRIPTION grep returns ZERO matches | Grep command + empty output inscribed |
| C — DESIGN.md status updated OPEN → CLOSED; slice table marks all 5 slices SHIPPED with commit refs | Diff inscribed |
| D — 058 changelog row appended in lab repo with arc 207 content + slice refs + forward-correction note | Diff inscribed |
| E — Arc 206 INSCRIPTIONs NOT touched | `git status` confirms arc 206 dir clean |
| F — Slice 1-4 SCORE docs NOT touched | Same |
| G — No source files (`*.rs`, `*.wat`, `*.toml`) touched | `git status` only shows the 3 paperwork files |

## Time-box

Predicted 30-45 min sonnet. Hard stop 60 min. Pure paperwork — INSCRIPTION drafting is most substantive part; DESIGN update + 058 row are mechanical.

## On completion

Return summary: rows passed/failed, FM 11 grep result, any STOP-triggers fired. Orchestrator commits both repos atomically + pushes after independent verification.

T-minus 0. Begin.
