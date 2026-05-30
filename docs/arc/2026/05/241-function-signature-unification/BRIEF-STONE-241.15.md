# BRIEF — Stone 241.15 — ZOMBIE PURGE: `:wat::core::try` + lowercase `option::expect` + lowercase `result::expect` HARD CUT

You are sonnet. **Stone 241.15 of arc 241. Wipe-the-board-of-distractions stone per user direction.** Three retired-but-operational forms die together. After this, Stone 241.16 (Enemy 3 — define eval-time residue) gets focused attention; Stone 241.17 INSCRIPTION closes arc 241.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## CRITICAL doctrine (pre-authorized — read these BEFORE strike)

1. **HARD CUT IS TOTAL** (`feedback_hard_cut_admits_no_bypasses`). Three zombies die. No "stays for help table" / "stays as sugar" / "soft retirement preserved" framings. The soft-deprecation arms at check.rs:1832-1888 (which fire arc-109-slice-1j warnings without rejecting) get SUPERSEDED by HARD-CUT-rejection arms with structured retirement remedies.

2. **Eval/infer functions UNCHANGED.** `eval_try` / `eval_option_expect` / `eval_result_expect` / `infer_try` all take an op-param naming which head the user wrote. Canonical forms (`:wat::core::Result/try` / `:wat::core::Option/expect` / `:wat::core::Result/expect`) share the SAME functions. Stone 241.15 ONLY deletes the dispatch arms that route retired heads. The functions stay.

3. **Stone 241.16 scope OFF-LIMITS.** Do NOT touch `is_mutation_head`, `parse_define_form`, `register_define`, `is_define_form` — those are Stone 241.16 (Enemy 3) scope.

4. **INTERSTITIAL is orchestrator-exclusive** (`feedback_sonnet_never_drafts_interstitial`). DO NOT write to `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`. Memory files OK.

5. **SCORE-write is part of the stone** (`feedback_score_present_check_before_closure`). Author `SCORE-STONE-241.15.md` at strike-end.

6. **FM 16 sonnet bash firewall awareness** — keep bash patterns simple, one per line, vanilla cargo/git/grep. No chained pipes.

## What to do

### S1 — Three HARD-CUT-rejection arms at `src/check.rs`

Mirror Stone 241.11/241.12/241.13/241.14 pattern. Place all three arms together for visual consistency:

```rust
// Stone 241.15 — Zombie A: :wat::core::try HARD CUT.
":wat::core::try" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.15); use ':wat::core::Result/try' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}

// Stone 241.15 — Zombie B: :wat::core::option::expect HARD CUT.
":wat::core::option::expect" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.15); use ':wat::core::Option/expect' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}

// Stone 241.15 — Zombie C: :wat::core::result::expect HARD CUT.
":wat::core::result::expect" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.15); use ':wat::core::Result/expect' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

### S2 — Append 10th + 11th + 12th RETIREMENT_TABLE entries

`src/remedy/retirement.rs`:

```rust
// Stone 241.15 — zombie purge: arc-109-slice-1j retirements now HARD CUT.
(":wat::core::try",                    ":wat::core::Result/try"),
(":wat::core::option::expect",         ":wat::core::Option/expect"),
(":wat::core::result::expect",         ":wat::core::Result/expect"),
```

### S3 — DELETE dispatch arms in `src/runtime.rs`

`src/runtime.rs:5694-5699` — delete THREE arms that route retired heads:
- `":wat::core::try" => eval_try(":wat::core::try", args, list_span, env, sym)` — DELETED
- `":wat::core::option::expect" => eval_option_expect(...)` — DELETED  
- `":wat::core::result::expect" => eval_result_expect(...)` — DELETED

The canonical dispatch arms (`Result/try`, `Option/expect`, `Result/expect`) STAY. Eval functions themselves UNCHANGED.

### S4 — DELETE deprecation-arm helper functions in `src/check.rs`

`src/check.rs:1832-1843`, `1851-1866`, `1874-1888` — three small helper functions detecting "is callee the retired head; emit warning." These were SOFT deprecation infrastructure. Stone 241.15 supersedes with HARD CUT. DELETE the helpers + their callers.

### S5 — DELETE dispatch arms in `src/check.rs`

`src/check.rs:5866-5874` — `":wat::core::try" => { ... infer_try(":wat::core::try", ...) }` dispatch arm DELETED. The `:wat::core::Result/try` arm at line 5918 STAYS.

`src/check.rs:2703-2734, 2823-2839` — dispatcher routing helpers currently route BOTH retired AND canonical heads (via `|| head_str == ":wat::core::option::expect"` clauses). Surgically REMOVE the retired-head clauses; canonical-routing stays.

### S6 — DELETE special_forms.rs registry entries

`src/special_forms.rs`:
- Lines 209-211 (comment block + `:wat::core::try` entry with `<retired-use-Result/try>` arity hint) — DELETED
- Line 349 (`:wat::core::try` reference; judge context) — likely DELETED
- Line 214 (`:wat::core::option::expect` entry) — DELETED
- Line 219 (`:wat::core::result::expect` entry) — DELETED

### S7 — Doc cascade migration

**`docs/USER-GUIDE.md`** — 7 sites total:
- Lines 1084, 1115, 2608, 2625, 3345, 3466 — `:wat::core::try` → `:wat::core::Result/try`
- Line 1097 — `:wat::core::option::expect` → `:wat::core::Option/expect`
- Line 1102 — `:wat::core::result::expect` → `:wat::core::Result/expect`

**`docs/SERVICE-PROGRAMS.md`** — 11 sites (lines 133, 134, 135, 194, 211, 213, 303, 305, 350, 351, 352): all `:wat::core::option::expect` → `:wat::core::Option/expect`

**`docs/CLOJURE-ROSETTA.md:35`** — `:wat::core::try` Rosetta-stone row → `:wat::core::Result/try`

**`docs/WAT-CHEATSHEET.md`** — lines 217-218 reference lowercase expect forms → PascalCase canonical

Per Stone 241.13/14 pattern: bulk sed CAN work if patterns are unambiguous. The strings `:wat::core::try` / `:wat::core::option::expect` / `:wat::core::result::expect` have no overlap with `:wat::core::Result/try` / `:wat::core::Option/expect` / `:wat::core::Result/expect`. Per-pattern bulk sed safe; verify each file's diff post-substitution.

### S8 — Reflection emitter audit

Per Stone 241.12/13/14 trap-door precedent:

```
grep -n "Keyword.*::try\b\|Keyword.*::option::expect\|Keyword.*::result::expect" src/
```

For any AST-construction site emitting retired forms: migrate to canonical names.

### S9 — Probe verification

`tests/probe_arc241_stone15_zombie_purge.rs` (STRIKE-READY; already committed). 6 contracts; **6/6 DISCONFIRM at HEAD** verified.

Post-stone: 6/6 PASS.

### S10 — Pre-INSCRIPTION grep gate

After all migrations:
```
grep -rn ":wat::core::try\b" src/ tests/ wat/
grep -rn ":wat::core::option::expect\b" src/ tests/ wat/
grep -rn ":wat::core::result::expect\b" src/ tests/ wat/
```

Acceptable categories post-stone:
1. `src/check.rs` — HARD-CUT-rejection arms (3)
2. `src/remedy/retirement.rs` — RETIREMENT_TABLE entries (3)
3. Historical comments + DELETED-arm markers
4. Stone 241.15 probe source (tests the rejection)

Goal: 0 ACTIVE uses outside acceptable categories.

### S11 — Author SCORE-STONE-241.15.md

Per `feedback_score_present_check_before_closure`. Path: `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.15.md`. Mirror SCORE-STONE-241.14.md shape.

## Discipline

- HARD CUT TOTAL for all 3 zombies; no soft-deprecation paths surviving
- Eval/infer functions UNCHANGED (shared with canonical forms via op-param)
- `src/argspec/*`, `src/lib.rs` UNCHANGED
- `src/remedy/retirement.rs` MODIFIED (append 10th/11th/12th entries)
- Stone 241.x and arc 237/238/242 probes preserved
- holon-rs NEVER touched (STOP-5)
- Auto-fixer crate (if used) must be EPHEMERAL — DELETED before commit
- DO NOT write to INTERSTITIAL (per `feedback_sonnet_never_drafts_interstitial`)
- SCORE doc authored at end (per `feedback_score_present_check_before_closure`)
- Pre-INSCRIPTION grep gate (S10) CLEAN post-stone
- Stone 241.16 scope OFF-LIMITS — no touching `is_mutation_head`/`parse_define_form`/etc.

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.15.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.15.md` — D1-D8 + T1-T6 + STOP triggers
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.14.md` — analogous cascade pattern (substrate-scaffolding deletion + HARD-CUT arms + RETIREMENT_TABLE entries)
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.13.md` — substrate scaffolding deletion + per-file test judgment
7. `/home/watmin/work/holon/wat-rs/src/check.rs` — find soft-deprecation arms (1832-1888), dispatch arms (5866-5874, 2703-2734, 2823-2839)
8. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — find dispatch arms (5694-5699) + eval functions (14772+, 14874+, 14924+ — DO NOT TOUCH)
9. `/home/watmin/work/holon/wat-rs/src/special_forms.rs` — find registry entries (211, 214, 219, 349)
10. `/home/watmin/work/holon/wat-rs/src/remedy/retirement.rs` — RETIREMENT_TABLE shape
11. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone15_zombie_purge.rs` — 6-contract probe (6/6 disconfirms at HEAD)

## Cadence

1. **Baseline:** `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test probe_arc241_stone15_zombie_purge 2>&1 | tail -3` (expect 0/6)
2. **S1+S2:** add 3 HARD-CUT arms + 3 RETIREMENT_TABLE entries (small)
3. **S3:** delete dispatch arms in runtime.rs (3 arms)
4. **S5:** delete dispatch arms in check.rs (1 standalone + surgical removal from 2 routing helpers)
5. **S4:** delete soft-deprecation helper functions in check.rs (3 functions + callers)
6. **S6:** delete special_forms.rs entries (3-4 entries)
7. **S8:** audit + migrate reflection emitters (likely zero)
8. **S7:** doc cascade (USER-GUIDE + SERVICE-PROGRAMS + CLOJURE-ROSETTA + WAT-CHEATSHEET)
9. **S9:** verify probe 6/6 PASS
10. **S10:** pre-INSCRIPTION grep gates CLEAN (all 3 forms)
11. **Final verification:** lib ≥ 890; workspace test-build clean; clippy ≤ 930
12. **S11:** author `SCORE-STONE-241.15.md`
13. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Compile errors not traced to zombie deletion cascade
2. Lib < 890
3. **120 min elapsed**
4. holon-rs touched (STOP-5)
5. Any retired form classified as "stays for help table" / "stays as sugar" / "soft retirement preserved" without HARD CUT → `feedback_hard_cut_admits_no_bypasses` violation
6. Canonical forms break (`Result/try` / `Option/expect` / `Result/expect`) — eval/infer functions damaged
7. Files outside permitted scope (`src/check.rs` / `src/runtime.rs` / `src/special_forms.rs` / `src/remedy/retirement.rs` / `src/closure_extract.rs` if reflection emitters touched / docs migration files / `tests/probe_arc241_stone15_*` / SCORE doc)
8. Stone 241.15 probe < 6/6
9. Stone 241.x or arc 237/238/242 probes regress
10. Clippy > 930 (looser gate; substrate refactor; arc 109 sweeps to zero)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.15.md NOT authored at end → `feedback_score_present_check_before_closure` violation
14. Stone 241.16 scope touched (`is_mutation_head` / `parse_define_form` / `register_define` / `is_define_form`) → D8 violation

## Post-strike return

Return one paragraph: 3 HARD-CUT arms at <file:line>; RETIREMENT_TABLE = 12 entries (3 added); dispatch arms deleted (count + locations); soft-deprecation helper functions deleted (count); special_forms entries deleted (count); reflection emitter audit result; doc migration count (USER-GUIDE + SERVICE-PROGRAMS + CLOJURE-ROSETTA + WAT-CHEATSHEET); pre-INSCRIPTION grep CLEAN (all 3 forms); Stone 241.15 probe 6/6; lib 890/0 (preserved); clippy count; SCORE doc path.

Stone 241.16 (Enemy 3 — define eval-time residue) opens after this with the board clean. Strike clean.
