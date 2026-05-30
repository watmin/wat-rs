# DESIGN — Stone 241.14.fix — `src/restriction_entry.rs` doc-comment rewrite (Stone 241.14 missed item)

**Status:** STRIKE-READY (2026-05-29 very late). Small doc-only stone fixing a missed item from Stone 241.14's cascade. User flagged 2026-05-29 very late after audit found stale doc comments in `src/restriction_entry.rs`.

## What Stone 241.14 missed

Stone 241.14 SHIPPED at `839cf9e6`: def-restricted + defn-restricted absorbed into binding_metadata; `defined_value_restrictions` parallel storage DELETED; walker renamed `walk_for_def_restricted_call` → `walk_for_restricted_call`; RestrictionEntry inventory channel migrated to populate `binding_metadata`.

The CODE in `src/restriction_entry.rs` is correct — struct + `inventory::collect!` + `#[restricted_to(...)]` proc-macro all unchanged in surface; populate-target migrated transparently to `binding_metadata`.

But the FILE-LEVEL doc comments describe a substrate that no longer exists:

| Line | Stale content |
|---|---|
| 4 | *"The wat-side `(:wat::core::def-restricted :name [prefixes] expr)` form"* — references RETIRED form as if live |
| 10-16 | "Subsequent stones plug in" with stones 2/3/4 description — historical roadmap (mostly acceptable but could mark as "already shipped") |
| 28-33 | *"insert into `SymbolTable.defined_value_restrictions`... `CheckEnv.defined_value_restrictions` mirror... `validate_def_restricted_caller_namespace` walker"* — references DELETED fields + DELETED-and-RENAMED walker |
| 65 | *"Semantics match arc 198 slice 1's wat-side `def-restricted` form"* — references RETIRED form |

This is a `feedback_hard_cut_admits_no_bypasses`-adjacent failure (the form is dead but documentation describes it as live). **Documentation zombie.**

## Current substrate state (post-Stone-241.16) for accurate rewrite

- Walker: `walk_for_restricted_call` (at `src/check.rs`; reads from `CheckEnv.binding_metadata`)
- Storage: `SymbolTable.binding_metadata` → mirrored to `CheckEnv.binding_metadata`
- Restriction key: `:restricted-to` (value = Vector of prefix-keywords AST)
- Helper: `restrictions_to_binding_metadata_ast` (converts `&[String]` prefix-vec to AST for inventory channel)
- User-facing forms: `(def :name {:restricted-to [<prefix-kw>...]} expr)` + `(defn :name {:restricted-to [<prefix-kw>...]} [args] -> :Ret body)`
- RETIRED forms: `:wat::core::def-restricted` + `:wat::core::defn-restricted` (HARD-CUT at check.rs; RETIREMENT_TABLE entries 8th + 9th)

## What this stone delivers

### S1 — Rewrite module-level doc comments

`src/restriction_entry.rs` lines 1-49 (module-level docs):
- Drop description of `(:wat::core::def-restricted :name [prefixes] expr)` as live form
- Replace with description of `(def :name {:restricted-to [...]} expr)` metadata-map mechanism
- Update wiring description: `inventory::iter::<RestrictionEntry>` populates `SymbolTable.binding_metadata[wat_name][":restricted-to"]` (not `defined_value_restrictions`)
- Update walker reference: `walk_for_restricted_call` (not `validate_def_restricted_caller_namespace`)
- Historical note: "Arc 198 originally minted def-restricted + defn-restricted forms with parallel storage `defined_value_restrictions`; Stone 241.14 retired the forms + unified storage into `binding_metadata`. The RestrictionEntry inventory channel + `#[restricted_to(...)]` proc-macro surface preserved; populate-target migrated transparently."
- Stones 2/3/4 historical reference: keep but mark "already shipped" since they did (per arc 198 INSCRIPTION)

### S2 — Rewrite struct doc + field doc comments

Lines 51-67 (struct + field docs):
- Drop "Semantics match arc 198 slice 1's wat-side `def-restricted` form" current-tense framing
- Replace with: "Semantics: prefix-keywords whitelist (caller FQDN must start with prefix entry ending in `::`, OR equal exact entry). Migrated from arc 198's def-restricted parser to Stone 241.14's binding_metadata path; the Rust-side declaration surface (`#[restricted_to(...)]`) is unchanged."

### S3 — Verify cargo doc builds clean

Doc rewrites can introduce broken intra-doc links. Sonnet runs `cargo doc --release --no-deps` to verify no new warnings.

### S4 — Author SCORE doc

Per `feedback_score_present_check_before_closure`. `SCORE-STONE-241.14.fix.md` at strike-end. Mirror SCORE-STONE-241.14.md shape (small variant since scope is doc-only).

## Locked decisions

### D1 — Doc-only scope; NO code changes

Sonnet edits ONLY `src/restriction_entry.rs` doc comments. The struct + inventory channel + proc-macro surface stays. No tests need migration; no lib changes.

### D2 — No probe (FM 2-bis doesn't apply to doc-only stones)

Documentation has no runtime behavior to disconfirm. Structural verification (grep returns 0 stale phrases post-stone) substitutes.

### D3 — Historical context preserved per `feedback_inscription_immutable`

Lines describing "Arc 198 originally..." stay; current-tense references to the RETIRED form change. Historical preserves; current-tense migrates.

### D4 — SCORE doc lighter than substantive stones

Less to capture; smaller doc; mirror shape but compress.

### D5 — INTERSTITIAL orchestrator-exclusive (`feedback_sonnet_never_drafts_interstitial`)

### D6 — Stone 241.17 scope OFF-LIMITS (INSCRIPTION orchestrator-direct)

## Trap-door audit

### T1 — Intra-doc links may break

`/// [`crate::...`]` links to functions/types — if any link to since-deleted symbols (e.g., `validate_def_restricted_caller_namespace`), cargo doc warnings.

Resolution: cargo doc check; fix broken links.

### T2 — Pattern-match other files with similar stale framings

While we're here, audit for other doc-comment stale patterns from Stone 241.14. Quick grep:
```
grep -rn "defined_value_restrictions\|validate_def_restricted_caller_namespace\|wat-side .*def-restricted form" src/ --include="*.rs"
```

If matches surface beyond restriction_entry.rs (excluding historical-comments + HARD-CUT-rejection-arms), Stone 241.14.fix's scope expands to include them.

## STOP triggers — REJECTION

1. Compile errors (shouldn't happen — doc-only)
2. Lib < 890
3. **30 min elapsed** (this stone is SMALL)
4. holon-rs touched (STOP-5)
5. Files outside `src/restriction_entry.rs` modified (unless T2 surfaces additional doc-stale patterns that warrant in-scope fixes)
6. Substrate code changes to RestrictionEntry struct or inventory channel (D1 violation)
7. cargo doc warnings introduced
8. Sonnet writes to INTERSTITIAL → D5 violation
9. SCORE-STONE-241.14.fix.md NOT authored → `feedback_score_present_check_before_closure` violation
10. Stone 241.17 scope touched (INSCRIPTION) → D6 violation

## FM 2-bis evidence

NOT APPLICABLE — doc-only stone. Structural verification per EXPECTATIONS substitutes:
- `grep -n "wat-side .*def-restricted form" src/restriction_entry.rs` → 0 matches post-stone
- `grep -n "defined_value_restrictions\|validate_def_restricted_caller_namespace" src/restriction_entry.rs` → 0 matches post-stone
- `grep -n "binding_metadata\|walk_for_restricted_call\|:restricted-to" src/restriction_entry.rs` → ≥ 1 match each post-stone (proves the doc rewrite landed)

## Calibration

**Target band: 10-30 min Mode A.** SMALLEST stone in arc 241. Doc-only; bounded to ~70-line file.

## What this unblocks

**Stone 241.17 — INSCRIPTION closes arc 241** (orchestrator-direct paperwork). Now genuinely "everything closed" — including the doc-staleness orphan that almost slipped through.

**The `feedback_defer_by_naming` doctrine memory** inscribed in Stone 241.17 explicitly references THIS .fix stone as a recent worked example: "Stone 241.14 missed a doc-comment update during cascade; user surfaced post-strike via direct inspection ('i see def-stricted still'); Stone 241.14.fix landed the missed item. PATTERN: cascade audit must include module-level doc comments, not just code paths + immediate user-facing strings."

Arc 241 closes clean. The cemetery is truly tidied.
