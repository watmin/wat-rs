# BRIEF — Stone 241.17 — `:wat::core::defmacro` signature migration to canonical (closes arc 177)

You are sonnet. **Stone 241.17 of arc 241 — absorbs arc 177's scope.** Defmacro signature migrates from arc 010/150 paren-pair-with-type to canonical Vector-of-triples mirroring defn shape (arc 166). After this: Stone 241.18 INSCRIPTION closes BOTH arc 241 AND arc 177.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## CRITICAL doctrine (pre-authorized — read these BEFORE strike)

1. **HARD CUT IS TOTAL** (`feedback_hard_cut_admits_no_bypasses`). Old paren-pair shape DIES; no compatibility shim. The keyword `:wat::core::defmacro` STAYS alive (form keeps existing); only the SIGNATURE SHAPE migrates. The rejection is shape-internal, not keyword-replacement (no new RETIREMENT_TABLE entry needed).

2. **`parse_defmacro_signature` DELETED entirely.** ~80+ lines die. Replaced by routing through `parse_argspec_triples` (Stone 241.1's canonical parser).

3. **Stone 241.18 (INSCRIPTION) OFF-LIMITS.** Sonnet does NOT touch INSCRIPTION work.

4. **INTERSTITIAL is orchestrator-exclusive** (`feedback_sonnet_never_drafts_interstitial`).

5. **SCORE-write is part of the stone** (`feedback_score_present_check_before_closure`). Author `SCORE-STONE-241.17.md` at strike-end.

6. **FM 16 sonnet bash firewall awareness** — simple bash patterns; vanilla cargo/grep.

## Shape migration

### OLD shape (paren-pair-with-type; 3 items)

```scheme
(:wat::core::defmacro
  (:my::macro (x :Type1) (y :Type2) -> :ReturnType)
  body)
```

3 items: `defmacro` head + signature-list (containing `(name (param :Type) ... -> :Ret)`) + body.

### NEW shape (canonical Vector-triple; 6 items mirroring defn)

```scheme
(:wat::core::defmacro :my::macro
  [x <- :Type1, y <- :Type2]
  -> :ReturnType
  body)
```

6 items: `defmacro` head + macro-name keyword + argspec Vector + `->` symbol + return-type keyword + body.

### NEW shape with rest-binder (mirrors defn rest-binder)

```scheme
(:wat::core::defmacro :my::variadic-wrap
  [& items <- :AST<wat::core::Vector<wat::WatAST>>]
  -> :AST<wat::core::nil>
  `(:wat::core::Vector ~@items))
```

## What to do

### S1 — Rewrite `parse_defmacro_form` to route through canonical

`src/macros.rs:320` — `parse_defmacro_form`:
- Accept 6+ items (head + name + argspec + `->` + return-type + body) and optionally 7 items (with metadata-map at items[2])
- Item 2 (argspec Vector) routes through `parse_argspec_triples` (already used by fn/defn/defclause)
- Item 4 is return-type keyword (after `->` Symbol at item 3)
- Item 5 (or 6 if metadata-map) is body
- MacroDef construction: name (item 1) + params/rest_param (from canonical parser output) + body (item 5/6) + span

### S2 — DELETE `parse_defmacro_signature`

`src/macros.rs:355` — entire function (~80+ lines). DELETED.

### S3 — HARD-CUT-rejection for old paren-pair shape

When item count is 3 AND item 1 is a List (the old signature-list shape), emit `MacroError::MalformedDefmacro` with structured reason:

```rust
MacroError::MalformedDefmacro {
    reason: "old defmacro signature shape (paren-pair-with-type) is retired (Stone 241.17); use canonical Vector-of-triples form: (:wat::core::defmacro :name [param <- :Type ...] -> :Ret body)".into(),
    span: list_span,
}
```

NOTE: shape-internal rejection; no RETIREMENT_TABLE entry (the keyword stays alive).

### S4 — Migrate 29 wat/ defmacro callers

Files (per `grep -rn ":wat::core::defmacro\b" wat/`):
- `wat/core.wat:180` — defn macro (LOAD-BEARING; all defn callers depend on this)
- `wat/test.wat` × 13 sites — deftest et al. test infrastructure
- `wat/Record.wat:93, 191` — record macros
- `wat/holon/*.wat` × 6 sites — algebra macros (Log/Sequential/Amplify/Bigram/Trigram)

Per-file judgment. Bulk-pattern mechanical migration:
- OLD: `(defmacro (NAME (P :T) ... & (R :T) -> :RET) BODY)`
- NEW: `(defmacro NAME [P <- :T ... & R <- :T] -> :RET BODY)`

Bulk-sed risky given multi-line patterns + variations. Per-file edit safer. Auto-fixer-with-parse-and-emit acceptable if EPHEMERAL (per Stone 241.11 precedent).

### S5 — Migrate 36 tests/ references

Files referencing `:wat::core::defmacro` per `grep -rln ":wat::core::defmacro\b" tests/`:
- Tests with wat-source strings (`r#"..."#`) containing defmacro → migrate fixtures
- Tests with AST-construction code emitting defmacro → migrate construction
- Tests asserting on defmacro structure → update assertions
- Comment-only references → preserve historical per `feedback_inscription_immutable`

### S6 — Doc cascade migration

- `docs/USER-GUIDE.md` — update active defmacro examples to new shape
- `docs/CLOJURE-ROSETTA.md` — update Rosetta-stone row
- `docs/INTENTIONS.md` — update if defmacro examples used

Preserve historical-tense references (per `feedback_inscription_immutable`).

### S7 — Reflection emitter audit

Per Stone 241.12/13/14/15/16 trap-door precedent:
```
grep -n "Keyword.*defmacro" src/
```

For any AST-construction site emitting defmacro keyword: verify output uses NEW shape; migrate emitter if old-shape.

### S8 — Probe verification

`tests/probe_arc241_stone17_defmacro_canonical.rs` (STRIKE-READY; already committed). 3 contracts; **3/3 DISCONFIRM at HEAD** verified.

Post-stone: 3/3 PASS.

### S9 — Pre-INSCRIPTION grep gate

After all migrations:
```
grep -rn ":wat::core::defmacro\s*$" wat/ | head -10
```

Verify no OLD shape remains (those sites have `(:wat::core::defmacro` followed by NEWLINE + indented signature-list as item 1; NEW shape has `(:wat::core::defmacro :name` on the same line).

Also:
```
cargo test --release --lib -p wat
cargo build --release --tests --workspace
```

### S10 — Author SCORE-STONE-241.17.md

Per `feedback_score_present_check_before_closure`. Path: `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.17.md`. Mirror SCORE-STONE-241.16.md shape (substantial substrate refactor + cascade).

## Discipline

- HARD CUT TOTAL for old paren-pair shape; no compatibility shim
- `parse_defmacro_signature` DELETED entirely
- `parse_defmacro_form` ROUTES through `parse_argspec_triples` (third major consumer after fn + defclause)
- `src/argspec/*`, `src/lib.rs` UNCHANGED
- `src/remedy/retirement.rs` UNCHANGED (shape-internal rejection; no new entry)
- Stone 241.x and arc 237/238/242 probes preserved
- holon-rs NEVER touched (STOP-5)
- Auto-fixer crate (if used) must be EPHEMERAL — DELETED before commit
- DO NOT write to INTERSTITIAL
- SCORE doc authored at end
- Pre-INSCRIPTION grep gate CLEAN post-stone
- Stone 241.18 scope OFF-LIMITS

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.17.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.17.md` — D1-D9 + T1-T7 + STOP triggers
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/177-defmacro-syntax-clojure/DESIGN.md` — the stub that this stone fills
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.16.md` — analogous cascade pattern (parse_define_form deletion + 30-site cascade)
7. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.13.md` — substrate scaffolding deletion + per-file test judgment
8. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.10.md` — substrate-mint shape reference
9. `/home/watmin/work/holon/wat-rs/src/macros.rs` — parse_defmacro_form (320) + parse_defmacro_signature (355; DELETE) + register_defmacros (274)
10. `/home/watmin/work/holon/wat-rs/src/argspec/mod.rs` — parse_argspec_triples (the canonical parser to route through)
11. `/home/watmin/work/holon/wat-rs/wat/core.wat` — defn macro at line 180 (LOAD-BEARING canonical example)
12. `/home/watmin/work/holon/wat-rs/wat/test.wat` — 13 defmacro sites (deftest et al.)
13. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone17_defmacro_canonical.rs` — 3-contract probe (3/3 disconfirms at HEAD)

## Cadence

1. **Baseline:** `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test probe_arc241_stone17_defmacro_canonical 2>&1 | tail -3` (expect 0/3)
2. **S1+S2:** rewrite parse_defmacro_form to route through canonical; delete parse_defmacro_signature
3. **S3:** add HARD-CUT-rejection arm for old paren-pair shape
4. **S4:** migrate 29 wat/ defmacro callers (start with wat/core.wat:180 defn macro — LOAD-BEARING; verify lib stays green after this single migration before proceeding)
5. **S5:** migrate 36 tests/ references (per-file judgment)
6. **S6:** doc cascade migration
7. **S7:** audit + migrate reflection emitters
8. **Cascade iteration:** cargo test --lib + cargo build after each migration phase
9. **S8:** verify probe 3/3 PASS
10. **S9:** pre-INSCRIPTION grep gate CLEAN
11. **Final verification:** lib ≥ 890; workspace test-build clean; clippy ≤ 940
12. **S10:** author SCORE doc
13. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Compile errors not traced to defmacro migration cascade
2. Lib < 890
3. **180 min elapsed**
4. holon-rs touched (STOP-5)
5. Old paren-pair shape preserved as "compatibility" path → `feedback_hard_cut_admits_no_bypasses` violation
6. `parse_defmacro_signature` PRESERVED (D3 violation)
7. Files outside permitted scope (`src/macros.rs` / `src/closure_extract.rs` if reflection emitters touched / `wat/*.wat` (29 callers) / test files referencing defmacro / doc files / `tests/probe_arc241_stone17_*` / SCORE doc)
8. Stone 241.17 probe < 3/3
9. Stone 241.x or arc 237/238/242 probes regress
10. Clippy > 940
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.17.md NOT authored at end → `feedback_score_present_check_before_closure` violation
14. Stone 241.18 scope touched (INSCRIPTION) → D9 violation

## Post-strike return

Return one paragraph: parse_defmacro_form rewritten at <file:line> routing through parse_argspec_triples; parse_defmacro_signature DELETED (line count); HARD-CUT-rejection arm at <file:line>; 29 wat/ migrations (per-file count); 36 tests/ migrations (count + per-file judgment summary); doc migration count; reflection emitter audit result; pre-INSCRIPTION grep gate CLEAN; Stone 241.17 probe 3/3; lib 890/0; clippy count; SCORE doc path.

Stone 241.18 (INSCRIPTION; orchestrator-direct) opens after this. arc 177 closes via absorption; arc 241 closes via INSCRIPTION; def-family parser unification GENUINELY COMPLETE. Strike clean.
