# Arc 170 slice 3 Phase G-lambda-docstrings BRIEF — kill the docstring lies + doc rot

**Sonnet.** Third slice of the retirement-theater purge (after G-console `b4ea6a4` + G-stream `2b8c253`). Walker `BareLegacyLambda` fires correctly (verified probe: "`:wat::core::lambda` is retired (arc 155); canonical FQDN is `:wat::core::fn`..."). User-facing docs and TWO substrate docstrings still lie about lambda routing/rendering.

**Acuteness inside this slice:** the substrate docstring lies in `eval_fn` (src/runtime.rs:4231-4235) and `infer_fn` (src/check.rs) actively mislead FUTURE IMPLEMENTERS about what code paths exist. They claim "`:wat::core::lambda` (retired fall-through) routes here" — but lambda dispatch was REMOVED in arc 155 slice 2. The walker fires fatal at check-time; nothing routes to eval_fn / infer_fn via the lambda path.

## Backstory — what arc 155 + arc 162 + arc 163 shipped

- **Arc 155**: user-facing keyword `:wat::core::lambda` retired in favor of `:wat::core::fn`. Walker `BareLegacyLambda` minted. Dispatch arms removed from runtime + check.
- **Arc 162**: Rust-side identifier sweep — `eval_lambda` → `eval_fn`, `infer_lambda` → `infer_fn`, `parse_lambda_signature*`, `WatLambdaSigmaFn`, `wat__core__lambda`, `<lambda@>` debug strings, etc.
- **Arc 163**: walker re-arm audit — re-armed `BareLegacyLambda` (the body had been prematurely retired); confirmed walker fires fatal at check time.

**The result:** zero runtime dispatch for `:wat::core::lambda`. Walker rejects at check.

**The lies still on disk:**

1. **Substrate docstrings (Bucket B, high acuteness):**
   - `src/runtime.rs:4231-4235` (eval_fn docstring): *"Dispatch arms for both `:wat::core::fn` (canonical) and `:wat::core::lambda` (retired fall-through) route here."* FALSE — lambda has no dispatch arm.
   - `src/check.rs` (infer_fn docstring, audit said ~9897 but line numbers shifted post-G-console; grep needed): same false claim. Verify on disk.

2. **Documentation prose (Bucket B):**
   - `docs/USER-GUIDE.md:584` — tier 2 list: "lambda, let, match..." → should be "fn, let, match..."
   - `docs/USER-GUIDE.md:1918-1919` — "Each Thread is an OS thread running the body **lambda**; the body owns its state (moved in via the **lambda's** closure...)" → concept rename to "fn"
   - `docs/USER-GUIDE.md:2112` — "no **lambda** wrapper needed" → concept rename
   - `docs/USER-GUIDE.md:2716` — "Anonymous lambdas render as `<lambda@<file>:<line>:<col>>`" — FALSE. Actual format per src/runtime.rs:14532 is `<fn@{}>`. Rewrite to true rendering.
   - `docs/USER-GUIDE.md:2836` — list mentioning "lambda, define, defmacro..." → context-dependent rewrite
   - `docs/USER-GUIDE.md:3236` — reference table row `:wat::core::lambda` — should NOT exist as a live row; remove or move to "retired" section
   - `docs/USER-GUIDE.md:3298` — spawn-thread table: "body is a **lambda**..." → "body is a **fn**..."

3. **Documentation Bucket A (active code examples that would fire walker):**
   - `docs/USER-GUIDE.md:1888` — `(:wat::core::lambda ...)` code example → transform to `(:wat::core::fn ...)`
   - `crates/wat-edn/docs/IPC-BRIDGE.md:150, 341` — two `(:wat::core::lambda ...)` examples → transform

4. **README.md (Bucket B stale test-file ref):**
   - `README.md:158` — lists `wat_spawn_lambda` test file. Renamed to `tests/wat_spawn_fn.rs` per arc 162. Update.

5. **.claude/skills/complectens/SKILL.md (Bucket A + B):**
   - `:293-294` — `(:wat::lru::HologramCacheService::MetricsCadence/new gate (:wat::core::lambda ...))` + `(:wat::core::lambda ((tx :Sender) ...) ...)` — Bucket A code refs → transform
   - `:303, 305, 328` — "embedded lambda", "lambda/closure literals" prose → Bucket B concept rename to "fn"

6. **Bucket C — KEEP (historical context):**
   - `docs/USER-GUIDE.md:803` — "Arc 155 collapsed the previous lambda / fn vocabulary into a..." — historical statement of WHAT WAS retired. KEEP.
   - `docs/USER-GUIDE.md:809` — "`:wat::core::lambda` is dead (arc 155 slice 2 retired the dispatch..." — historical, KEEP.
   - `docs/USER-GUIDE.md:137` — judgment call ("with a wat lambda" — context determines)
   - `src/check.rs` BareLegacyLambda variant + Display + walker firing — Bucket D scaffolding. KEEP.
   - `src/special_forms.rs` retirement scaffolding — KEEP if mirror of let* / lambda pattern.
   - `tests/wat_arc144_special_forms.rs` — Bucket D test fixtures (verify it's similar to wat_arc154_kill_let_star.rs).
   - Any docs/arc/ — never touched (FM 11).

7. **Other files surfaced by grep** (verify Bucket per file):
   - `docs/CONVENTIONS.md` — check hits
   - `docs/SERVICE-PROGRAMS.md` — check hits
   - `docs/COMPACTION-AMNESIA-RECOVERY.md` — likely historical context (Bucket C) but verify each hit; it's our discipline doc, careful
   - `docs/README.md` — check hits

## Required reading IN ORDER

1. **`docs/arc/2026/05/170-program-entry-points/BRIEF-SLICE-3-PHASE-G-STREAM.md`** + `SCORE-SLICE-3-PHASE-G-STREAM.md` (commit `2b8c253`) — the immediate precedent
2. **`docs/arc/2026/05/170-program-entry-points/RETIREMENT-THEATER-INVENTORY.md`** — full audit context
3. **`src/runtime.rs:4231-4235`** — the eval_fn docstring lie (verify line numbers; may have shifted post-G-console)
4. **`src/check.rs` infer_fn docstring** — grep for the parallel lie; verify on disk
5. **`src/runtime.rs:14532`** — the actual fn debug format (`<fn@{}>` — the truth USER-GUIDE.md:2716 contradicts)
6. **`docs/SUBSTRATE-AS-TEACHER.md`** — Pattern 3 (symbol migration)
7. **`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 14** — the discipline

## Implementation path

### Phase 1 — Substrate docstring fixes (10-15 min)

Two surgical edits:
1. **`src/runtime.rs:4231-4235` (eval_fn)**: rewrite the docstring. Remove the "lambda (retired fall-through) routes here" claim. Replace with: "Arc 155 retired `:wat::core::lambda`; canonical is `:wat::core::fn`. Walker fires fatal at check time on user-source `:wat::core::lambda`; no runtime dispatch path exists. This function is reached only via the `:wat::core::fn` dispatch arm." Surface final wording in SCORE.
2. **`src/check.rs` infer_fn docstring**: parallel fix. Grep for current location; rewrite with same shape.

### Phase 2 — Documentation prose sweep (20-30 min)

File-by-file, hit-by-hit, judgment-driven:
- `docs/USER-GUIDE.md`: ~10 hits across tier list / Thread body description / fn rendering claim / spawn-thread table / reference table / code examples. Mix of Bucket A (code → transform) + Bucket B (concept rename) + Bucket C (historical KEEP).
- `crates/wat-edn/docs/IPC-BRIDGE.md`: 2 Bucket A code examples → transform.
- `README.md:158`: test file name correction.
- `.claude/skills/complectens/SKILL.md`: 5 hits, mix Bucket A + B.
- Other files (`CONVENTIONS.md`, `SERVICE-PROGRAMS.md`, `COMPACTION-AMNESIA-RECOVERY.md`, `docs/README.md`): triage per hit.

### Phase 3 — Verify

```bash
# 1. Workspace stays green
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'

# 2. Walker probe
echo '(:wat::core::lambda [x] x)' > /tmp/probe-lambda.wat
./target/release/wat /tmp/probe-lambda.wat 2>&1 | head -5
# Expected: BareLegacyLambda with :wat::core::fn canonical

# 3. Final grep — should return only Bucket C/D
grep -rln ":wat::core::lambda\|lambda@\|wat__core__lambda\|eval_lambda\|infer_lambda" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: src/check.rs + src/special_forms.rs + tests/wat_arc144_special_forms.rs (Bucket D) + historical Bucket C entries (USER-GUIDE:803,809 etc — list in SCORE)

# 4. fn rendering truth check
grep -n "fn@" src/runtime.rs | head -3
# Confirms `<fn@{}>` is the actual format
```

## Scope (what's IN)

- 2 substrate docstring fixes (eval_fn + infer_fn)
- ~20-30 doc hits across 6-9 files swept
- USER-GUIDE.md reference table cleanup (remove the live `:wat::core::lambda` row OR move to retired section)
- Bucket C inventory documented in SCORE
- Workspace stays at 2205 / 0 failed

## Scope (what's OUT)

- Walker / variant retirement — explicitly NOT (per user direction "we remove its raised exceptions when arc 109 is completed"; that's Phase 2 Slice 4)
- `BareLegacyLambda` scaffolding — KEEP (Bucket D)
- `wat/std/` phantom paths — separate Phase G-wat-std-paths
- `:wat::std::stream::*` — already shipped (G-stream)
- `:wat::console::*` — already shipped (G-console)
- Anything under `docs/arc/` (FM 11)
- `~/.claude/` memory system

## Ship criteria (6 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `src/runtime.rs` eval_fn docstring lie fixed | grep + read; new wording surfaced for review |
| B | `src/check.rs` infer_fn docstring lie fixed | grep + read; new wording surfaced |
| C | Documentation prose sweep complete across ~6-9 files | grep |
| D | `docs/USER-GUIDE.md:2716` fn rendering claim corrected to actual `<fn@{}>` format | manual review |
| E | `cargo check --release` green; workspace 2205 / 0 failed | full test run |
| F | Final grep returns ONLY Bucket C/D scaffolding + historical context | grep |

**6 rows.** All must PASS.

## Predicted runtime

**30-50 min sonnet.** Larger doc sweep than G-stream (more files, more judgment) but no walker mint.

**Hard cap:** 100 min (2×).

## Constraints (hard)

- DO NOT delete `BareLegacyLambda` variant / Display / Diagnostic / walker firing (Bucket D — stays until arc 109 closes)
- DO NOT delete `tests/wat_arc144_special_forms.rs` (test fixtures verify retirement)
- DO NOT delete USER-GUIDE.md:803 or :809 historical context (Bucket C)
- DO NOT touch anything under `docs/arc/` (FM 11)
- DO NOT commit (orchestrator atomic-commits)
- DO NOT use deferral language in SCORE
- DO NOT operate outside `/home/watmin/work/holon/wat-rs/`
- DO NOT touch `~/.claude/` memory system
- DO NOT use --no-verify or skip hooks
- The substrate docstring rewrite must NOT introduce a NEW lie; surface final wording for orchestrator review
- Workspace must stay at 2205 / 0 failed

## Honest delta categories (anticipated)

1. **Substrate docstring final wording** — surface eval_fn + infer_fn new docstrings for orchestrator review before commit
2. **USER-GUIDE.md:3236 reference table** — judgment call: remove the `:wat::core::lambda` row entirely, OR move to a "retired" section, OR replace with `:wat::core::fn`. Surface choice.
3. **USER-GUIDE.md:2716 fn rendering claim** — final wording for the corrected sentence (verify `<fn@{}>` matches actual debug format including any colon/space conventions)
4. **Bucket C identifications** — historical entries to keep; list each file:line in SCORE
5. **`docs/COMPACTION-AMNESIA-RECOVERY.md` hits** — careful triage; this is the discipline doc. Most likely Bucket C historical, but verify each hit
6. **Bonus catches** — if sweeping surfaces hits beyond the audit's 9 (expected per G-console + G-stream pattern), surface them
7. **Anything pre-existing source-level `:wat::core::lambda`** in workspace — would mean arc 155 / 162 sweep missed something; STOP and report

## Cross-references

- `b4ea6a4` — Phase G-console (precedent walker mint + sweep)
- `2b8c253` — Phase G-stream (precedent pure doc sweep)
- `RETIREMENT-THEATER-INVENTORY.md` — the audit
- `SUBSTRATE-AS-TEACHER.md` — Pattern 3 doctrine
- `src/runtime.rs:14532` — actual fn debug format (source of truth for USER-GUIDE:2716 fix)
- Arc 155 INSCRIPTION (lambda user-facing retirement)
- Arc 162 INSCRIPTION (lambda Rust-side rename)
- Arc 163 retirement leftover audit (walker re-arm)
