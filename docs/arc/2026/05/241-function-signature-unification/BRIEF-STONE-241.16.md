# BRIEF — Stone 241.16 — `:wat::core::define` EVAL-TIME RESIDUE COMPLETION (Enemy 3 of 4)

You are sonnet. **Stone 241.16 of arc 241. LAST scheme-style retirement before broader clojure conversion arcs (172/173/174/175/176/177/181).** Completes Stone 241.11's partial HARD CUT — startup-check rejects today; eval-time scaffolding survived deliberately. Stone 241.16 deletes the eval-time residue. After this: Stone 241.17 INSCRIPTION closes arc 241.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## CRITICAL doctrine (pre-authorized — read these BEFORE strike)

1. **HARD CUT IS TOTAL** (`feedback_hard_cut_admits_no_bypasses`). Stone 241.11 left eval-time scaffolding deliberately ("defense-in-depth"). Stone 241.16 deletes it. NO "defense-in-depth via keeping define recognized" framings. The bypass-rejection mechanism preserves (refuses unknown mutation forms); the SPECIFIC head `:wat::core::define` is no longer in the recognized set.

2. **`parse_define_form` DELETED entirely.** ~30 error-construction sites die with it. Per Stone 241.13 precedent (`src/dispatch.rs` 445-line deletion).

3. **Bypass-rejection test fixtures MIGRATE** (do not preserve with define). Tests at `src/freeze.rs:1651/1807/1985` use `:wat::core::define` as a convenient known-mutation-head for bypass tests. Migrate to use `:wat::core::defstruct` or another known mutation head.

4. **INTERSTITIAL is orchestrator-exclusive** (`feedback_sonnet_never_drafts_interstitial`). DO NOT write to `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`. Memory files OK.

5. **SCORE-write is part of the stone** (`feedback_score_present_check_before_closure`). Author `SCORE-STONE-241.16.md` at strike-end.

6. **FM 16 sonnet bash firewall awareness** — keep bash patterns simple, vanilla cargo/git/grep.

## What to do

### S1 — DELETE `parse_define_form` entirely (the big deletion)

`src/runtime.rs:4399+` — full function deletion. ~30 error-construction sites (`head: ":wat::core::define".into()` style) die with it. Cascade per substrate-as-teacher.

### S2 — DELETE `is_define_form` + caller

`src/runtime.rs:3547-3551` — function deletion + the caller at line 3551.

### S3 — DELETE `:wat::core::define` arms from form predicates

- `src/freeze.rs:1312` `is_mutation_form` — remove `| ":wat::core::define"` arm
- `src/freeze.rs:1355` `is_declaration_form` — remove `| ":wat::core::define"` arm
- `src/runtime.rs:27427` `is_mutation_head` — remove `| ":wat::core::define"` arm

### S4 — DELETE check.rs processing arms

- `src/check.rs:2884` `if define_head == ":wat::core::define"` — sandbox-scope inner-form scan. Branch is unreachable post-startup-check; DELETED.
- `src/check.rs:3141-3142` `":wat::core::define" => {` arm — investigate context; if this is a pre-Stone-241.11 processing arm, DELETED. If it IS Stone 241.11's HARD-CUT-rejection arm, KEEP.

### S5 — Investigate check.rs:7049 arm

```
src/check.rs:7049:            ":wat::core::define" => {
```

Read context to determine if this is the Stone 241.11 HARD-CUT-rejection arm (KEEP; update to include "Stone 241.16" marker per probe C01) OR pre-Stone-241.11 processing (DELETED).

**If KEEP with marker update:** modify the existing reason string to include "Stone 241.16" alongside "Stone 241.11" (e.g., "is retired (Stone 241.11; eval-time residue completed Stone 241.16)"). The probe C01 will pass.

**If REPLACED by NEW arm:** add a new Stone 241.16 HARD-CUT arm; remove the Stone 241.11 arm. New reason string mentions Stone 241.16.

Sonnet judges shape; both produce probe-passing behavior.

### S6 — DELETE special_forms.rs entries

- Line 175: registry entry `insert(&mut m, ":wat::core::define", ...)` — DELETED
- Line 331: spot-check test reference (`":wat::core::define"` in the audited-forms list) — REMOVED from list

### S7 — Migrate error message strings

Three sites mention `(:wat::core::define ...)` in error messages as a "use the proper form" hint:
- `src/check.rs:913` sandbox-scope-leak hint — replace `(:wat::core::define {} ...)` with `(:wat::core::defn {} ...)`
- `src/runtime.rs:2513` same migration
- `src/check.rs:2414` error format string `":wat::core::define (body)"` — update to defn reference (or remove if obsolete)

### S8 — Migrate test fixtures

**Bypass-rejection tests** at `src/freeze.rs:1651, 1807, 1985`:

Current: programmatically construct AST with `:wat::core::define` head; verify eval-time refuses.
Post-stone: migrate fixture head to `:wat::core::defstruct` (or another known mutation form). MECHANISM preserves (eval refuses unknown mutation forms); specific head changes.

**`tests/wat_arc144_special_forms.rs:210-211`:**

```rust
assert_special_form(":wat::core::define", ":wat::core::define");
let (_, sig, _) = three_probes(":wat::core::define");
```

Post-stone: define is NOT a special form. Either:
- DELETE these two lines (the test scope shrinks)
- MIGRATE to assert HARD CUT (assert that registry lookup returns None for define)

Recommend MIGRATE to assert HARD-CUT-absence per substrate-as-teacher pattern (the test continues to assert the discipline).

**`tests/wat_arc144_uniform_reflection.rs:103, 121`:**

Current: STALE assertion that reflection AST head is `:wat::core::define`.
Post-stone: reflection emits `:wat::core::defn`. UPDATE the assertion.

**`tests/probe_let_splice_define.rs`, `tests/probe_do_splice_define.rs`:**

Per-file judgment. Likely DELETED (pre-Stone-241.11 tests) OR MIGRATED to defn-headed forms preserving the splice-test mechanism.

### S9 — Update documentation comments (preserve historical, update current-tense)

Comments at `runtime.rs:23, 1413, 1428, 2101` reference define. UPDATE current-tense usage to defn; preserve historical "Stone 241.11 retired" comments per `feedback_inscription_immutable`.

### S10 — Reflection emitter audit

Per Stone 241.12/13/14/15 precedent:
```
grep -n "Keyword.*::define\b" src/
```

Any AST-construction site emitting `:wat::core::define` keyword migrates or dies.

### S11 — Probe verification

`tests/probe_arc241_stone16_define_eval_residue.rs` (STRIKE-READY; already committed). 4 contracts; 1/4 disconfirms at HEAD (C01 — Stone 241.16 marker absent); 3/4 are PRESERVATION (C02-C04 verify Stone 241.11 HARD CUT continuity).

Post-stone: 4/4 PASS.

### S12 — Pre-INSCRIPTION grep gate

After all migrations:
```
grep -rn ":wat::core::define\b" src/ tests/ wat/
```

Acceptable categories post-stone:
1. `src/check.rs` — HARD-CUT-rejection arm (Stone 241.11/241.16 combined; with Stone 241.16 marker)
2. `src/remedy/retirement.rs` — RETIREMENT_TABLE entry (preserved from Stone 241.11)
3. Historical comments (e.g., "Stone 241.11 retired"; "Stone 241.16 eval-time completed")
4. Stone 241.11 + 241.16 probe sources (test the rejection)
5. Stone 241.12 migration history comments in tests

Goal: 0 ACTIVE uses outside acceptable categories.

### S13 — Author SCORE-STONE-241.16.md

Per `feedback_score_present_check_before_closure`. Path: `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.16.md`. Mirror SCORE-STONE-241.15.md shape.

## Discipline

- HARD CUT TOTAL — no "defense-in-depth preservation" framings
- `parse_define_form` DELETED entirely (~30 sites die with it)
- Bypass-rejection tests migrate to alternative mutation heads
- Stone 241.11's HARD-CUT-rejection arm (if at check.rs:7049) KEPT or REPLACED with Stone 241.16 marker
- `src/argspec/*`, `src/lib.rs` UNCHANGED
- `src/remedy/retirement.rs` UNCHANGED (Stone 241.11's entry preserved; no new entries this stone)
- Stone 241.x + arc 237/238/242 probes preserved (except wat_arc144_special_forms.rs + wat_arc144_uniform_reflection.rs which migrate)
- holon-rs NEVER touched (STOP-5)
- Auto-fixer crate (if used) must be EPHEMERAL — DELETED before commit
- DO NOT write to INTERSTITIAL
- SCORE doc authored at end
- Pre-INSCRIPTION grep gate (S12) CLEAN post-stone
- Stone 241.17 scope is INSCRIPTION-only (orchestrator-direct paperwork); sonnet does NOT touch

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.16.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.16.md` — D1-D8 + T1-T6 + STOP triggers + battle-plan position (LAST scheme-style retirement)
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.15.md` — zombie purge pattern (most recent; cascade scale comparable)
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.13.md` — substrate scaffolding deletion (445-line file; analogous to parse_define_form deletion scope)
7. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.11.md` — the original define HARD CUT (this completes its work)
8. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — parse_define_form (4399+), is_define_form (3547), register_defines (2591), is_mutation_head (27427)
9. `/home/watmin/work/holon/wat-rs/src/check.rs` — define processing arms (2884, 3141, 7049); error message hints (913, 2414)
10. `/home/watmin/work/holon/wat-rs/src/freeze.rs` — is_mutation_form (1312), is_declaration_form (1355), bypass-rejection tests (1651, 1807, 1985)
11. `/home/watmin/work/holon/wat-rs/src/special_forms.rs` — registry entry (175) + audited-forms test (331)
12. `/home/watmin/work/holon/wat-rs/src/remedy/retirement.rs` — Stone 241.11 entry (preserved; no changes this stone)
13. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone16_define_eval_residue.rs` — 4-contract probe (1 disconfirms + 3 preservation)

## Cadence

1. **Baseline:** `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test probe_arc241_stone16_define_eval_residue 2>&1 | tail -3` (expect 3/4)
2. **S5:** investigate check.rs:7049 context; judge KEEP-with-marker-update vs REPLACE
3. **S1:** delete parse_define_form entirely (~30 sites die)
4. **S2:** delete is_define_form + caller
5. **S3:** delete define arms from form predicates (3 sites)
6. **S4:** delete check.rs processing arms (2884, 3141 if not HARD CUT)
7. **S6:** delete special_forms.rs entries (175, 331)
8. **S7:** migrate error message hints (3 sites)
9. **S10:** audit + migrate reflection emitters (likely zero)
10. **S8:** migrate test fixtures (bypass-rejection × 3; wat_arc144_special_forms; wat_arc144_uniform_reflection; per-file judgment on probe_let/do_splice_define)
11. **S9:** update current-tense doc comments
12. **Cascade iteration:** cargo test --lib + cargo build after each phase
13. **S11:** verify probe 4/4 PASS
14. **S12:** pre-INSCRIPTION grep gate CLEAN
15. **Final verification:** lib ≥ 890; workspace test-build clean; clippy ≤ 935
16. **S13:** author SCORE doc
17. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Compile errors not traced to define eval-time deletion cascade
2. Lib < 890
3. **180 min elapsed**
4. holon-rs touched (STOP-5)
5. `:wat::core::define` use classified as "defense-in-depth preservation" without deletion → `feedback_hard_cut_admits_no_bypasses` violation
6. `parse_define_form` PRESERVED (D2 violation — DELETED is the action)
7. Files outside permitted scope (see S5/S7/S8 inventory)
8. Stone 241.16 probe < 4/4
9. Stone 241.x or arc 237/238/242 probes regress (except `tests/wat_arc144_special_forms.rs` + `tests/wat_arc144_uniform_reflection.rs` which migrate)
10. Clippy > 935 (looser gate; substrate refactor; arc 109 sweeps to zero)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.16.md NOT authored at end → `feedback_score_present_check_before_closure` violation
14. Stone 241.17 scope touched (INSCRIPTION paperwork) → D8 violation

## Post-strike return

Return one paragraph: parse_define_form DELETED (line count); is_define_form + caller DELETED; form predicates (is_mutation_form + is_declaration_form + is_mutation_head) — define arms removed; check.rs processing arms — DELETED or HARD CUT (specify per-arm); check.rs:7049 disposition (KEPT with marker update OR REPLACED); special_forms entries DELETED (count); error message migrations (count); test fixture migrations (bypass-rejection × 3; wat_arc144 × 2; probe_let/do_splice_define disposition); reflection emitter audit result; pre-INSCRIPTION grep CLEAN; Stone 241.16 probe 4/4; lib delta; clippy count; SCORE doc path.

Stone 241.17 (INSCRIPTION; orchestrator-direct paperwork) opens after this. The def-family death campaign completes. Arc 241 closes. Scheme → Clojure conversion at the define layer DONE. Strike clean.
