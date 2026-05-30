# BRIEF — Stone 241.14 — `:wat::core::def-restricted` + `:wat::core::defn-restricted` ABSORB INTO METADATA-MAP (Enemy 4 of 4)

You are sonnet. **Stone 241.14 of arc 241. Enemy 4 of 4 in the define-family death campaign.** Honors broken Stone 241.6 D10 + line-182 commitment (orphaned 25 days when Stone 241.10's scope shifted to remedy apparatus). **Per user direction: def and defn are the ONLY ways to declare bindings post-stone. Both def-restricted (substrate primitive) AND defn-restricted (wat macro) die.** Restrictions live as `:restricted-to` key in metadata-map on def/defn.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## CRITICAL doctrine (pre-authorized — read these BEFORE strike)

1. **HARD CUT IS TOTAL** (`feedback_hard_cut_admits_no_bypasses`). Both `:wat::core::def-restricted` and `:wat::core::defn-restricted` die EVERYWHERE. The `defined_value_restrictions` parallel storage is DELETED entirely. `binding_metadata` (Stone 241.6) becomes the sole restriction store. NO "the field stays for compatibility" framings. NO "the macro stays as sugar" framings — user direction: def + defn are the ONLY definers.

2. **The orphaned commitment** — Stone 241.6 DESIGN D10 + line 182 explicitly committed `:wat::core::def-restricted` + `:wat::core::defn-restricted` HARD CUT to "Stone 241.10 or similar." Stone 241.10 absorbed remedy-apparatus scope; the commitment was orphaned. Stone 241.14 lands the work 25 days late. The INSCRIPTION (Stone 241.16) will acknowledge this orphan explicitly per `feedback_defer_by_naming`.

3. **Stone 241.15 scope OFF-LIMITS.** Do NOT touch `is_mutation_head`, `parse_define_form`, `register_define`, `is_define_form`, or `:wat::core::define` eval-time scaffolding. Those are Enemy 3 (Stone 241.15) scope.

4. **INTERSTITIAL is orchestrator-exclusive** (`feedback_sonnet_never_drafts_interstitial`). DO NOT write to `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` or any INTERSTITIAL artifact. Memory files OK.

5. **SCORE-write is part of the stone** (`feedback_score_present_check_before_closure`). Author `SCORE-STONE-241.14.md` as the FINAL step before returning. Orchestrator verifies SCORE-doc-present before commit.

6. **FM 16 sonnet bash firewall awareness** — keep bash patterns simple, one per line, vanilla cargo/git/grep. No chained pipes. If "bash denied" claim surfaces, run `which cargo` to verify.

## What to do

### S1 — Migrate walker to read from `binding_metadata`

Walker at `src/check.rs:3823` (`walk_for_def_restricted_call`) currently reads via `env.get_defined_value_restriction(head)`. Migrate to read from `env.get_binding_metadata(head)` (or equivalent reflection over `binding_metadata`).

Extract prefix list from the metadata's `:restricted-to` key. The value is `WatAST::List` with `:wat::core::Vector` head; subsequent items are prefix keywords. Write small helper `extract_prefix_list_from_metadata` adjacent to the walker.

Rename walker: `walk_for_def_restricted_call` → `walk_for_restricted_call` (drop the "def_" prefix; the mechanism is no longer def-specific). **But** keep `CheckError::DefRestrictedCallerNotAllowed` variant name unchanged per `feedback_inscription_immutable` — historical variant names preserved.

### S2 — DELETE `defined_value_restrictions` storage entirely

- `SymbolTable.defined_value_restrictions` field — DELETED (`src/runtime.rs:1724`)
- `CheckEnv.defined_value_restrictions` field — DELETED (`src/check.rs:2050`)
- `set_defined_value_restriction` / `get_defined_value_restriction` methods — DELETED
- All populate-paths in `register_runtime_defs_form` (runtime.rs:2676, 2996, 4430, 4504) — DELETED
- `CheckEnv::from_symbols` mirror copy (check.rs:2095) — DELETED
- Debug print line for the field (runtime.rs:1788-1789) — DELETED

### S3 — HARD-CUT-rejection arm for `:wat::core::def-restricted`

Mirror Stone 241.11/241.12/241.13 pattern at `src/check.rs`:

```rust
":wat::core::def-restricted" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.14); use ':wat::core::def' with metadata-map: `(def :name {{:restricted-to [<prefix-kw>...]}} expr)`", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

DELETE the existing `:wat::core::def-restricted` parser arm at `src/check.rs:5615+` (`infer_def_restricted`), `src/check.rs:9978-10072` (`extract_def_restricted_binding`), and the runtime parser code at `src/runtime.rs:4306+`.

### S4 — HARD-CUT-rejection arm for `:wat::core::defn-restricted`

Mirror S3 pattern. The macro at `wat/core.wat:202-209` is DELETED (the HARD-CUT arm catches any residual callers).

```rust
":wat::core::defn-restricted" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.14); use ':wat::core::defn' with metadata-map: `(defn :name {{:restricted-to [<prefix-kw>...]}} [<args>] -> :<Ret> body)`", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

### S5 — Append 8th + 9th RETIREMENT_TABLE entries

`src/remedy/retirement.rs`:

```rust
// Stone 241.14 — def-restricted absorbed into def + metadata-map.
(":wat::core::def-restricted",    ":wat::core::def"),
(":wat::core::defn-restricted",   ":wat::core::defn"),
```

### S6 — Migrate `RestrictionEntry` inventory channel to populate `binding_metadata`

Current: `src/restriction_entry.rs` defines `RestrictionEntry { wat_name, prefixes }`; iteration at `src/freeze.rs:835` populates `defined_value_restrictions`.

Target: keep struct + `inventory::collect!` channel + `#[restricted_to(...)]` proc-macro attribute UNCHANGED. Migrate the iteration to populate `binding_metadata[wat_name]` with `:restricted-to` key → Vector of prefix keywords WatAST.

This preserves arc 170 Stone B's restrictions on `Thread/join-result` + `Process/join-result` without disruption to Rust-side declarations.

### S7 — Cascade migration of user-surface forms

**Tests:**

- `tests/wat_arc198_def_restricted.rs` — 5 tests use def-restricted/defn-restricted. **MIGRATE per S7 examples below** (access-control SEMANTICS preserved by new mechanism; tests remain as regression guards for new path).
- `tests/wat_arc170_stone_b_walker_collapse.rs` — 1 reference; per-site judgment

Migration shape (def-restricted → def):

```scheme
;; OLD:
(:wat::core::def-restricted :my::kernel::restricted-fn
  :restricted-to [:wat::kernel::]
  (:wat::core::fn [] -> :wat::core::i64 42))

;; NEW:
(:wat::core::def :my::kernel::restricted-fn
  {:restricted-to [:wat::kernel::]}
  (:wat::core::fn [] -> :wat::core::i64 42))
```

Migration shape (defn-restricted → defn):

```scheme
;; OLD:
(:wat::core::defn-restricted :my::kernel::restricted-fn
  :restricted-to [:wat::kernel::]
  [] -> :wat::core::i64 42)

;; NEW:
(:wat::core::defn :my::kernel::restricted-fn
  {:restricted-to [:wat::kernel::]}
  [] -> :wat::core::i64 42)
```

**Docs:**

- `docs/USER-GUIDE.md:734-795` — `:wat::core::def-restricted` / `defn-restricted` section. REWRITE to document the metadata-map approach. The OLD form names should be referenced as RETIRED with citation to arc 241.14 + the metadata-map shape.
- `docs/CONVENTIONS.md:33-34` — Wat (sugar) entry currently references `defn-restricted`. REPLACE with metadata-map shape.

### S8 — `wat/core.wat` macro DELETION

`wat/core.wat:202-209` — the `:wat::core::defn-restricted` macro DEFINITION. DELETED. The HARD-CUT arm at check.rs catches residual callers.

`wat/core.wat:187-201` — the macro's documentation comment block. UPDATE to historical voice ("Arc 198 defined defn-restricted as a sugar over def-restricted; Stone 241.14 retired both. Restrictions now live as `:restricted-to` key in metadata-map on def/defn.").

### S9 — Reflection emitter audit

Per Stone 241.12/241.13 trap-door precedent:

```
grep -n "Keyword.*def-restricted\|Keyword.*defn-restricted" src/
```

For each AST-construction site emitting these forms: migrate to emit `:wat::core::def` / `:wat::core::defn` with metadata-map.

### S10 — Probe verification

`tests/probe_arc241_stone14_restricted_absorbed.rs` (STRIKE-READY; already committed). 6 contracts; **5/6 DISCONFIRM at HEAD** (C01 is preservation — allowed caller passes both via no-enforcement at HEAD and via metadata-driven enforcement post-stone).

Post-stone: 6/6 PASS.

### S11 — Pre-INSCRIPTION grep gate (Stone 241.14-specific scope)

After all migrations, run:
```
grep -rn ":wat::core::def-restricted\b" src/ tests/ wat/
grep -rn ":wat::core::defn-restricted\b" src/ tests/ wat/
```

Acceptable categories post-stone:
1. `src/check.rs` — HARD-CUT-rejection arms (2)
2. `src/remedy/retirement.rs` — RETIREMENT_TABLE entries (2)
3. Historical comments in any file
4. Stone 241.14 probe source (tests the rejection)

Goal: 0 ACTIVE uses outside acceptable categories.

### S12 — Author SCORE-STONE-241.14.md

Per `feedback_score_present_check_before_closure`. Path: `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.14.md`. Mirror SCORE-STONE-241.13.md shape:
- Header (Mode A; runtime; substrate deletion size; test migration count)
- Phase A scorecard
- Migration cascade audit (walker rewrite; storage deletion; macro deletion; cascade)
- HARD-CUT arms verbatim (both)
- RETIREMENT_TABLE post-stone (9 entries verbatim)
- Pre-INSCRIPTION grep verification (both retired forms)
- Honest deltas
- Calibration
- What this unblocks (Stone 241.15 — Enemy 3)
- NO Vigilia section (D6 — no namespaced home)

## Discipline

- HARD CUT TOTAL for both forms; no "stays as sugar" framings
- `defined_value_restrictions` storage DELETED; `binding_metadata` is sole
- `RestrictionEntry` inventory channel STAYS (only populate-target changes)
- `walk_for_def_restricted_call` renamed to `walk_for_restricted_call`; error variant name preserved
- `src/argspec/*`, `src/lib.rs` UNCHANGED
- `src/remedy/retirement.rs` MODIFIED (append 8th + 9th entries)
- Stone 241.x and arc 237/238/242 probes preserved
- holon-rs NEVER touched (STOP-5)
- Auto-fixer crate (if used) must be EPHEMERAL — DELETED before commit (per Stone 241.10/241.11 precedent)
- DO NOT write to INTERSTITIAL (D7)
- SCORE doc authored at end (D8)
- Pre-INSCRIPTION grep gate (S11) CLEAN post-stone
- Stone 241.15 scope OFF-LIMITS (D9 — no touching is_mutation_head, parse_define_form, register_define, is_define_form)

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.14.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.14.md` — D1-D9 + T1-T8 + STOP triggers + battle-plan position (Enemy 4 of 4)
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.13.md` — Stone 241.13 calibration (substrate deletion + per-test judgment; analogous cascade pattern)
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.12.md` — trap-door absorption pattern + bandaid-rip-with-receipts
7. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.10.md` — substrate-mint shape (walker enhancement is structurally similar to the remedy mint)
8. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.6.md` — the orphaned commitment (D10 + line 182); also Stone 241.6 details the binding_metadata storage that Stone 241.14 elevates to sole-store
9. `/home/watmin/work/holon/wat-rs/src/restriction_entry.rs` — the RestrictionEntry inventory channel (full read; understand the wiring; plan the migration target)
10. `/home/watmin/work/holon/wat-rs/src/check.rs` — find walk_for_def_restricted_call (line 3823), CheckEnv.defined_value_restrictions (line 2050), set/get methods (lines 2188-2195), infer_def_restricted (line 5615+), extract_def_restricted_binding (line 9978+)
11. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — find SymbolTable.defined_value_restrictions (line 1724), populate-paths in register_runtime_defs_form (lines 2676, 2996, 4430, 4504), parser code at line 4306+
12. `/home/watmin/work/holon/wat-rs/src/freeze.rs` — find RestrictionEntry iteration at line 835
13. `/home/watmin/work/holon/wat-rs/wat/core.wat` — find defn-restricted macro at lines 187-209
14. `/home/watmin/work/holon/wat-rs/src/remedy/retirement.rs` — RETIREMENT_TABLE shape
15. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone14_restricted_absorbed.rs` — 6-contract probe (5/6 disconfirms at HEAD; C01 preservation)

## Cadence

1. **Baseline:** `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test probe_arc241_stone14_restricted_absorbed 2>&1 | tail -3` (expect 1/5 — C01 preservation passes)
2. **S1:** migrate walker to read from binding_metadata; rename to walk_for_restricted_call
3. **S2:** delete defined_value_restrictions storage + methods (cascade per substrate-as-teacher; mechanical)
4. **S6:** migrate RestrictionEntry inventory channel to populate binding_metadata
5. **S3+S4:** add HARD-CUT arms (both def-restricted + defn-restricted)
6. **S5:** append 8th + 9th RETIREMENT_TABLE entries
7. **S8:** delete wat/core.wat defn-restricted macro; update comment block
8. **S9:** audit + migrate reflection emitters (likely zero work)
9. **S7:** cascade migrate test sites + docs
10. **Cascade iteration:** `cargo test --release --lib -p wat` after each phase; `cargo build` after each substrate edit
11. **S10:** verify probe 6/6 PASS
12. **S11:** pre-INSCRIPTION grep gates CLEAN (both forms)
13. **Final verification:** lib ≥ 890; workspace test-build clean (`cargo build --release --tests --workspace`); clippy ≤ 925
14. **S12:** author `SCORE-STONE-241.14.md`
15. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Compile errors not traced to restriction migration cascade
2. Lib < 890
3. **180 min elapsed**
4. holon-rs touched (STOP-5)
5. `:wat::core::def-restricted` or `:wat::core::defn-restricted` survives as ACTIVE substrate use post-stone (outside HARD-CUT arms + retirement entries + historical comments + probe source)
6. `defined_value_restrictions` field/method PRESERVED (D2 violation — the field MUST be DELETED)
7. Files outside permitted scope (per D9 + S2/S3/S4/S6/S7/S8/S9 inventory)
8. Stone 241.14 probe < 6/6
9. Stone 241.x or arc 237/238/242 probes regress (except `wat_arc198_def_restricted.rs` which IS migrating)
10. Clippy > 925 (looser gate; substrate refactor causes line-shift; arc 109 sweeps to zero)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → D7 + `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.14.md NOT authored at end → D8 + `feedback_score_present_check_before_closure` violation
14. Stone 241.15 scope touched (is_mutation_head / parse_define_form / register_define / is_define_form) → D9 violation
15. Arc 170 Stone B Thread/join-result + Process/join-result restrictions silently broken (T2 violation per DESIGN)

## Post-strike return

Return one paragraph: walker migrated to binding_metadata reads at <file:line>; defined_value_restrictions storage DELETED (sites count); RestrictionEntry inventory channel migrated to populate binding_metadata; HARD-CUT arms at <file:line> (2 arms); RETIREMENT_TABLE = 9 entries (2 added); wat/core.wat defn-restricted macro DELETED; reflection emitter audit result; test migration count (`wat_arc198_def_restricted.rs` 5 sites + other); doc migration count (USER-GUIDE + CONVENTIONS); pre-INSCRIPTION grep CLEAN (both forms); Stone 241.14 probe 6/6; lib delta from baseline; clippy count; auto-fixer status; SCORE doc path.

Stone 241.15 (Enemy 3 — define eval-time residue) opens after this. Strike clean.
