# BRIEF — Stone 241.14.fix — `src/restriction_entry.rs` doc-comment rewrite

You are sonnet. **Stone 241.14.fix — small doc-only stone.** Fixes a missed item from Stone 241.14's cascade. User flagged 2026-05-29 very late: doc comments in `src/restriction_entry.rs` describe the RETIRED `:wat::core::def-restricted` form + DELETED `defined_value_restrictions` field + DELETED/RENAMED walker as if they're live.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## CRITICAL doctrine (pre-authorized)

1. **DOC-ONLY scope.** Edit ONLY `src/restriction_entry.rs` doc comments. The struct + `inventory::collect!` + `#[restricted_to(...)]` proc-macro surface stays. No code changes.

2. **Stone 241.17 scope OFF-LIMITS** — INSCRIPTION is orchestrator-direct paperwork.

3. **INTERSTITIAL is orchestrator-exclusive** (`feedback_sonnet_never_drafts_interstitial`). DO NOT touch `INTERSTITIAL-REALIZATIONS.md`.

4. **SCORE-write is part of the stone** (`feedback_score_present_check_before_closure`). Author `SCORE-STONE-241.14.fix.md` at strike-end.

5. **FM 16 firewall awareness** — simple bash patterns, vanilla cargo/grep.

## Current substrate state (for accurate rewrite)

- Walker: `walk_for_restricted_call` (in `src/check.rs`; reads from `CheckEnv.binding_metadata`)
- Storage: `SymbolTable.binding_metadata: HashMap<String, HashMap<String, WatAST>>` → mirrored to `CheckEnv.binding_metadata`
- Restriction key in metadata: `:restricted-to` (value = Vector of prefix-keywords AST)
- Helper for inventory channel: `restrictions_to_binding_metadata_ast` (at `src/runtime.rs`; converts `&[String]` prefix-vec to AST)
- User-facing forms post-Stone-241.14:
  - `(:wat::core::def :name {:restricted-to [<prefix-kw>...]} expr)`
  - `(:wat::core::defn :name {:restricted-to [<prefix-kw>...]} [args] -> :Ret body)`
- RETIRED forms (HARD CUT at check.rs; RETIREMENT_TABLE entries 8th + 9th):
  - `:wat::core::def-restricted`
  - `:wat::core::defn-restricted`

## What to do

### S1 — Rewrite module-level doc comments (lines 1-49)

Current state (stale):
- Line 4: describes `(:wat::core::def-restricted :name [prefixes] expr)` as live wat-side form
- Lines 28-33: describes populating `SymbolTable.defined_value_restrictions` + `CheckEnv.defined_value_restrictions` mirror + `validate_def_restricted_caller_namespace` walker

Target state (honest):
- Open with the metadata-map mechanism: `(def :name {:restricted-to [<prefix-kw>...]} expr)` (Stone 241.14) is the user-facing form
- Describe RestrictionEntry inventory channel as the Rust-side analog: `inventory::submit!(RestrictionEntry { ... })` at module scope; entries gathered at link time
- Wiring: startup pipeline iterates `inventory::iter::<RestrictionEntry>`; converts each entry to AST via `restrictions_to_binding_metadata_ast`; inserts into `SymbolTable.binding_metadata[wat_name][":restricted-to"]`. The `CheckEnv` mirrors via `from_symbols`. The `walk_for_restricted_call` walker (in `src/check.rs`) reads from `CheckEnv.binding_metadata` and validates caller FQDN matches the prefix-list.
- Historical note: Arc 198 originally minted `:wat::core::def-restricted` + `:wat::core::defn-restricted` forms with parallel storage `SymbolTable.defined_value_restrictions`. Stone 241.14 retired the forms (HARD CUT) and unified storage into `binding_metadata` (per Stone 241.6/7's metadata-map mechanism). RestrictionEntry inventory channel + `#[restricted_to(...)]` proc-macro surface preserved; populate-target migrated transparently.
- Stones 2/3/4 references in current doc (lines 10-16): rewrite to "Arc 198 slice 2 minted RestrictionEntry struct (Stone 1); `#[restricted_to(...)]` proc-macro attribute (Stone 2); applied to `eval_kernel_*_join_result` (Stone 3); retired arc 170 Stone B walker rule (Stone 4). All four stones SHIPPED in arc 198." (Mark as historical-already-shipped instead of "subsequent stones plug in" future-tense.)

### S2 — Rewrite struct + field doc comments (lines 51-67)

Current state (stale):
- Line 65: "Semantics match arc 198 slice 1's wat-side `def-restricted` form."

Target state (honest):
- "Semantics: prefix-keywords whitelist. Each entry is either a namespace prefix (ending in `::` — caller FQDN must START WITH the entry) or exact match (no trailing `::` — caller FQDN must EQUAL the entry). Originally minted at arc 198 slice 1 (wat-side `def-restricted` form); migrated to Stone 241.14's `binding_metadata` path. The Rust-side declaration surface (`#[restricted_to(...)]` proc-macro + this inventory channel) is unchanged."

### S3 — Audit for similar stale framings in other files

Quick grep:
```
grep -rn "defined_value_restrictions" src/ --include="*.rs"
grep -rn "validate_def_restricted_caller_namespace" src/ --include="*.rs"
grep -rn "wat-side .*def-restricted form" src/ --include="*.rs"
```

If matches surface beyond `restriction_entry.rs` AND beyond historical-comments + HARD-CUT-rejection-arms, those need similar treatment. Per Stone 241.14.fix scope expansion: include the additional doc-stale sites if found (per `feedback_trap_door_build_the_dependency` — build the missing piece forward).

If matches stay bounded to `restriction_entry.rs` only, no scope expansion needed.

### S4 — cargo doc verification

After doc rewrites:
```
cargo doc --release --no-deps 2>&1 | tail -10
```

Verify NO new warnings (intra-doc links + missing-docs etc.).

### S5 — Author SCORE-STONE-241.14.fix.md

Per `feedback_score_present_check_before_closure`. Path: `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.14.fix.md`. Mirror SCORE-STONE-241.14.md shape (compressed since scope is doc-only). Include:
- Header (Mode A; runtime; doc rewrite size; cargo doc clean)
- Scorecard rows (grep returns documented; lib preserved; cargo doc clean; SCORE present)
- Structural verification (post-stone restriction_entry.rs references binding_metadata + walk_for_restricted_call + :restricted-to; stale phrases gone)
- Diff summary (lines rewritten; lines added; lines removed)
- Honest deltas (any T2 scope-expansion sites discovered; any T1 intra-doc link breakages)
- What this unblocks (Stone 241.17 INSCRIPTION can ship arc 241 closure cleanly)

## Discipline

- DOC-ONLY scope — `src/restriction_entry.rs` doc comments only
- No code changes; struct + inventory + proc-macro surface preserved
- Stone 241.17 (INSCRIPTION) OFF-LIMITS
- No probe (doc-only stones don't have behavior to disconfirm)
- Lib preserved at 890/0
- cargo doc must build without new warnings
- SCORE doc authored at end
- DO NOT write to INTERSTITIAL

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.14.fix.md` — D1-D6 + T1-T2 + STOP triggers
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.14.fix.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.14.md` — the prior stone whose cascade missed this item (context: what Stone 241.14 did + what it should have updated)
5. `/home/watmin/work/holon/wat-rs/src/restriction_entry.rs` — THE FILE to edit
6. `/home/watmin/work/holon/wat-rs/src/check.rs` — find walk_for_restricted_call (for accurate naming in rewrite)
7. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — find binding_metadata + restrictions_to_binding_metadata_ast (for accurate naming)

## Cadence

1. **Baseline:** `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0)
2. **S1:** rewrite module-level doc comments in restriction_entry.rs
3. **S2:** rewrite struct + field doc comments
4. **S3:** audit other src/ files for similar stale framings (T2)
5. **S4:** cargo doc verification
6. **S5:** author SCORE-STONE-241.14.fix.md
7. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Lib < 890 (shouldn't change — doc-only)
2. **30 min elapsed**
3. holon-rs touched (STOP-5)
4. Files outside `src/restriction_entry.rs` modified (unless T2 surfaces additional doc-stale sites that warrant inclusion)
5. Substrate code changes to RestrictionEntry struct or inventory channel
6. cargo doc warnings introduced
7. Sonnet writes to INTERSTITIAL
8. SCORE doc NOT authored at end
9. Stone 241.17 scope touched

## Post-strike return

Return one paragraph: restriction_entry.rs doc rewrite scope (lines changed); module-level + struct + field doc updates; T2 scope-expansion sites discovered (count + list if any); cargo doc result; lib 890/0 preserved; SCORE doc path.

Arc 241 INSCRIPTION (Stone 241.17) opens after this. Strike clean.
