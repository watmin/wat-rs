# BRIEF — Stone 241.13 — `:wat::core::define-dispatch` HARD CUT + DispatchRegistry scaffolding deletion (Enemy 2 of 3)

You are sonnet. **Stone 241.13 of arc 241. Enemy 2 of 3 in the define-family death campaign.** Retires `:wat::core::define-dispatch` (arc 146 mechanism) + DELETES all DispatchRegistry scaffolding. `:wat::core::defclause` (Stone 237.2) is the surviving dispatch entity kind.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## CRITICAL doctrine (pre-authorized — read these BEFORE strike)

1. **HARD CUT IS TOTAL** (`feedback_hard_cut_admits_no_bypasses`). The retired form `:wat::core::define-dispatch` dies EVERYWHERE in the substrate. There is NO "infrastructure stays empty so it's fine" framing. There is NO "deletion can wait" framing. The 445-line `src/dispatch.rs` file gets DELETED. The DispatchRegistry plumbing across check.rs + freeze.rs + runtime.rs + resolve.rs + special_forms.rs gets DELETED.

2. **`:wat::core::defclause` is the replacement** (Stone 237.2 SHIPPED `bdd9eb6c`). Active wat-source callers of define-dispatch: ZERO (already evacuated to ∀T intrinsics per arc 237.7a/7b/7c). No caller migration burden — only substrate scaffolding deletion + test cleanup.

3. **Stone 241.14 scope is OFF-LIMITS.** Do NOT touch `is_mutation_head`, `parse_define_form`, `register_define`, `is_define_form`, or `freeze.rs` test fixtures using `:wat::core::define` for bypass tests. Those are Enemy 3 scope.

4. **INTERSTITIAL is orchestrator-exclusive** (`feedback_sonnet_never_drafts_interstitial`). DO NOT write to `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` or any INTERSTITIAL artifact. Memory files OK.

5. **SCORE-write is part of the stone** (`feedback_score_present_check_before_closure`). Author `SCORE-STONE-241.13.md` at the end before returning.

6. **FM 16 sonnet bash firewall awareness** — keep bash patterns simple, one per line, vanilla cargo/git/grep. No chained pipes. If "bash denied" claim surfaces, run `which cargo` to verify.

## What to do

### S1 — HARD-CUT-rejection arm at `src/check.rs`

Mirror Stone 241.11/241.12 pattern:

```rust
":wat::core::define-dispatch" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.13); use ':wat::core::defclause' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

Place adjacent to existing HARD-CUT arms (struct/struct-restricted/enum/define/Char/runtime define-alias).

### S2 — Append 7th RETIREMENT_TABLE entry

`src/remedy/retirement.rs`:

```rust
// Stone 241.13 — defclause replaces define-dispatch.
(":wat::core::define-dispatch",   ":wat::core::defclause"),
```

### S3 — DELETE `src/dispatch.rs` entirely

The entire 445-line file. Zero wat-source consumers; the registry is always empty post-arc-237.7. `git rm src/dispatch.rs`.

### S4 — DELETE DispatchRegistry plumbing across substrate

Cascade through all consumers (per `docs/SUBSTRATE-AS-TEACHER.md` — let the compiler tell you):

| File | Symbol | Action |
|---|---|---|
| `src/freeze.rs` | `use crate::dispatch::DispatchRegistry` import; `dispatchs: DispatchRegistry` field on Freeze; `set_dispatch_registry()` calls; `dispatchs()` accessor; step-2 registration walker (~30-50 lines) | DELETE |
| `src/check.rs` | `dispatch_registry: Option<Arc<...>>` field on CheckEnv (line 2031); `dispatch_registry()` method (lines 2177-2180); env init (line 2143); dispatch_registry guard at line 5618-5620 routing to `infer_dispatch_call` | DELETE |
| `src/runtime.rs` | `dispatch_registry` field on SymbolTable (line 1671); `DispatchRegistry::new()` instantiation (line 27977); dispatch form constructors at lines 13372, 13385, 13427, 13568, 13746 (~30-50 lines) | DELETE |
| `src/resolve.rs` | `sym.dispatch_registry()` consultation at line 328 (~5 lines) | DELETE |
| `src/special_forms.rs` | entry at line 194: `":wat::core::define-dispatch"` | DELETE |
| `src/freeze.rs` | walker-arm match cases at lines 1382, 1422 for `:wat::core::define-dispatch` | DELETE (the HARD-CUT arm consolidates handling) |

Use grep to find any sites the table missed:
```
grep -n "define-dispatch\|DispatchRegistry\|register_dispatch\|parse_dispatch\|dispatch_form\|infer_dispatch\|dispatch_registry" src/
```

### S5 — Test migration / deletion (per-file judgment)

Per D4: every test either migrates to defclause (current purpose) or deletes (obsolete regression guard for retired mechanism).

**`tests/wat_arc146_dispatch_mechanism.rs`** — entire arc 146 acceptance test. Mechanism RETIRING. Recommend: DELETE entirely OR repurpose 1-2 contracts to test HARD-CUT acceptance (substrate-as-teacher pattern; preferable to test the rejection + remedy quality). Use judgment; if defclause has its own acceptance tests, delete this file.

**`tests/probe_arc237_7a_length_intrinsic.rs`** — comment says "behavior regression guard - works TODAY via define-dispatch" but `length` is ∀T intrinsic post-arc-237.7a. STALE.
- Grep for other tests covering the intrinsic `length` path
- If covered elsewhere: DELETE this probe
- If not covered: REWRITE to test the intrinsic path

**`tests/probe_arc237_7b_intrinsic_typing.rs`** — same pattern as 7a for empty?/contains?/get. Same options.

**`tests/wat_arc144_uniform_reflection.rs:278-298`** — STALE assertion `line.contains("define-dispatch")` expecting empty? to be dispatch-registered. Empty? is now ∀T intrinsic. UPDATE the assertion to match current reflection (likely defclause or intrinsic naming; verify by running the test against current behavior and reading what `lookup-define` returns).

**`tests/probe_declaration_form_lift.rs`** — declaration-form lift test includes define-dispatch as one of 5 lifted forms. DROP the define-dispatch case (probe 3 + variants at lines 199-230, 340+); preserve other declaration forms. Update comment at line 8 listing the lifted forms.

**`tests/probe_def_not_special.rs:259, 283`** — uses define-dispatch as test fixture for "def is not special" pattern. MIGRATE fixture to `:wat::core::defclause` (semantically equivalent for the test's purpose).

### S6 — Update historical comments

**`wat/core.wat:8`** — current-tense comment "Each declaration uses arc 146's `:wat::core::define-dispatch`" is STALE (decls already removed). Rewrite to historical: "Originally used arc 146's `:wat::core::define-dispatch` (retired Stone 241.13)" or remove if no longer accurate.

**`src/runtime.rs:5711-5715`, `src/check.rs:20437`** — comments about "Reborn from define-dispatch (core.wat) to Rust builtin" are HISTORICAL and accurate. KEEP.

### S7 — Reflection emitter audit

Stone 241.12 surfaced an analogous trap-door: closure_extract.rs emitted retired form for prologue re-freeze. Audit for define-dispatch:

```
grep -n "Keyword.*define-dispatch" src/closure_extract.rs src/runtime.rs src/check.rs
```

If any AST-construction site emits `:wat::core::define-dispatch` keyword: migrate or delete.

### S8 — Probe verification

`tests/probe_arc241_stone13_define_dispatch_hard_cut.rs` (already committed STRIKE-READY). 2 contracts; **2/2 FAIL at HEAD** verified. Post-stone: 2/2 PASS.

### S9 — Pre-INSCRIPTION grep gate (Stone 241.13-specific scope)

After all deletions, run:
```
grep -rn ":wat::core::define-dispatch\b" src/ tests/ wat/
```

Acceptable categories post-stone:
1. `src/check.rs` — HARD-CUT-rejection arm
2. `src/remedy/retirement.rs` — RETIREMENT_TABLE entry
3. Historical comments in any file (e.g., comments describing the retirement, "Reborn from define-dispatch")
4. Stone 241.13 probe source (tests the rejection)
5. `tests/wat_arc146_dispatch_mechanism.rs` — IF repurposed for HARD-CUT acceptance (else deleted)

Goal: 0 ACTIVE uses outside acceptable categories.

### S10 — Author SCORE-STONE-241.13.md

Per `feedback_score_present_check_before_closure`. Path: `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.13.md`.

Mirror SCORE-STONE-241.12.md shape. Include:
- Header (Mode A; runtime; substrate deletion size; test files migrated/deleted)
- Phase A scorecard
- Migration cascade audit
- HARD CUT arm verbatim
- RETIREMENT_TABLE post-stone (7 entries verbatim)
- Pre-INSCRIPTION grep verification
- Per-test judgment summary (which deleted; which migrated; why)
- Honest deltas
- Calibration (predicted vs actual)
- What this unblocks (Stone 241.14 — Enemy 3)
- NO Vigilia section (D5 — no namespaced home)

## Discipline

- HARD CUT TOTAL — no "infrastructure stays empty" framings
- `src/dispatch.rs` DELETED entirely
- DispatchRegistry plumbing DELETED across substrate
- `src/argspec/*`, `src/lib.rs` UNCHANGED
- `src/remedy/retirement.rs` MODIFIED (append 7th entry per S2)
- Stone 241.x and 242.x probes preserved; arc 237/238 probes preserved (except probe_arc237_7a/7b which may be deleted/repurposed per S5)
- holon-rs NEVER touched (STOP-5)
- No new error variants
- Auto-fixer crate (if used) must be EPHEMERAL — DELETED before commit (per Stone 241.10/241.11 precedent)
- DO NOT write to INTERSTITIAL (D7)
- SCORE doc authored at end (D6)
- Pre-INSCRIPTION grep gate (S9) CLEAN post-stone
- Stone 241.14 scope OFF-LIMITS (per D8 — no touching is_mutation_head, parse_define_form, etc.)

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.13.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.13.md` — D1-D8 + T1-T7 + STOP
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.12.md` — Stone 241.12 calibration + trap-door absorption pattern (2 in-flight trap-doors)
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.11.md` — HARD CUT mass-cascade discipline
7. `/home/watmin/work/holon/wat-rs/src/dispatch.rs` — the file to DELETE (full read to understand scope)
8. `/home/watmin/work/holon/wat-rs/src/freeze.rs` — DispatchRegistry plumbing
9. `/home/watmin/work/holon/wat-rs/src/check.rs` — dispatch_registry field + guard + HARD-CUT arms
10. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — dispatch_registry field + form constructors
11. `/home/watmin/work/holon/wat-rs/src/resolve.rs` — dispatch_registry consultation
12. `/home/watmin/work/holon/wat-rs/src/special_forms.rs` — define-dispatch entry
13. `/home/watmin/work/holon/wat-rs/src/remedy/retirement.rs` — RETIREMENT_TABLE shape
14. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone13_define_dispatch_hard_cut.rs` — 2-contract probe (2/2 disconfirms at HEAD)

## Cadence

1. **Baseline:** `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test probe_arc241_stone13_define_dispatch_hard_cut 2>&1 | tail -3` (expect 0/2)
2. **S1:** add HARD-CUT arm at check.rs
3. **S2:** append 7th RETIREMENT_TABLE entry
4. **S3:** delete src/dispatch.rs (`git rm`)
5. **S4:** cascade-delete DispatchRegistry plumbing (let compiler errors guide; substrate-as-teacher)
6. **S6:** update wat/core.wat:8 historical comment
7. **S7:** audit + migrate reflection emitters (likely zero)
8. **S5:** per-test judgment — migrate/delete each of 6 test files per S5 inventory
9. **Cascade iteration:** `cargo test --release --lib -p wat` after each deletion phase; cargo build after each substrate edit
10. **S8:** verify probe 2/2 PASS
11. **S9:** pre-INSCRIPTION grep gate CLEAN
12. **Final verification:** lib ≥ 890 (note: test deletions may reduce count; track delta); workspace test-build clean; clippy ≤ 920 (looser gate per STOP-trigger 10 — substrate deletions cause line-shift re-attribution)
13. **S10:** author `SCORE-STONE-241.13.md`
14. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Compile errors not traced to dispatch deletion cascade
2. Lib < 890 (post-241.12 baseline) — note: track expected delta from test deletions; document in SCORE
3. **180 min elapsed**
4. holon-rs touched (STOP-5)
5. `:wat::core::define-dispatch` use classified as "infrastructure stays empty" / "deletion can wait" without migration → D1 + `feedback_hard_cut_admits_no_bypasses` violation
6. `src/dispatch.rs` PRESERVED (D3 violation — DELETED is the action)
7. Files outside permitted scope (`src/dispatch.rs` DELETED / `src/check.rs` / `src/freeze.rs` / `src/runtime.rs` / `src/resolve.rs` / `src/special_forms.rs` / `src/remedy/retirement.rs` / `src/closure_extract.rs` if reflection emitters touched / test files in S5 inventory / `tests/probe_arc241_stone13_*` / `wat/core.wat` for historical comment update / SCORE doc)
8. Stone 241.13 probe < 2/2
9. Stone 241.x or 242.x probes regress (except probe_arc237_7a/7b which may be deleted/repurposed per S5)
10. Clippy > 920 (looser gate; arc 109 sweeps to zero)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.13.md NOT authored at end → `feedback_score_present_check_before_closure` violation
14. Stone 241.14 scope touched (`is_mutation_head`, `parse_define_form`, `register_define`, `is_define_form`) → D8 violation

## Post-strike return

Return one paragraph: HARD CUT arm at <file:line>; 7th RETIREMENT_TABLE entry; src/dispatch.rs DELETED (line count removed); DispatchRegistry plumbing deletion sites (count); test files migrated (count, list) vs deleted (count, list); reflection emitter audit result; pre-INSCRIPTION grep CLEAN (active uses = 0); Stone 241.13 probe 2/2; lib delta from baseline (expected drop from test deletions; track); clippy count; auto-fixer status; SCORE doc path.

Stone 241.14 (Enemy 3 — define eval-time residue) opens after this. Strike clean.
