# DESIGN — Stone 241.15 — ZOMBIE PURGE: `:wat::core::try` + lowercase `option::expect` + lowercase `result::expect` HARD CUT

**Status:** STRIKE-READY (2026-05-29 very late). **Wipe-the-board-of-distractions stone** per user direction. Three retired-but-operational forms die together. Mirror Stone 241.13 + 241.14 substrate-scaffolding-deletion pattern.

## User direction (load-bearing)

User direction 2026-05-29 very late: *"annihilate the zombies - before define is entertained - wipe the board of distractions."*

Battle plan reordering: zombies BEFORE Enemy 3 (define eval-time residue). Rationale: clear the small distractions first; Enemy 3 then gets focused attention. The audit found 3 clear zombies; bundling them per `feedback_momentum_ordering` (apparatus identical; cascade bounded).

## What's a zombie

A **zombie** = retired form name that's marked deprecated in the substrate's registries but STILL OPERATIONAL via dispatch + eval. Direct violation of `feedback_hard_cut_admits_no_bypasses` ("Retired forms die EVERYWHERE in substrate").

Three confirmed zombies (full audit 2026-05-29 very late):

| Zombie | Retirement source | Replacement | Substrate sites |
|---|---|---|---|
| **A — `:wat::core::try`** | arc 109 slice 1j (~Stone 058-033) | `:wat::core::Result/try` | dispatch arms (check.rs:5866 + runtime.rs:5694) + `eval_try` shares op-parameter (canonical form lives in same fn) + `infer_try` shares same pattern + deprecation-arm at check.rs:1832-1843 + special_forms registry 211/349 |
| **B — `:wat::core::option::expect`** | arc 109 slice 1j | `:wat::core::Option/expect` (PascalCase Type/method) | dispatch arm (runtime.rs:5695) + shared eval/infer via op-param + deprecation-arm check.rs:1851-1866 + dispatcher routing 2703-2734 + special_forms registry 214 |
| **C — `:wat::core::result::expect`** | arc 109 slice 1j | `:wat::core::Result/expect` (PascalCase Type/method) | dispatch arm (runtime.rs:5698) + shared eval/infer via op-param + deprecation-arm check.rs:1874-1888 + dispatcher routing 2823-2839 + special_forms registry 219 |

**Key insight (simplifies the work):** the `eval_try` / `eval_option_expect` / `eval_result_expect` functions ALREADY ACCEPT an `op` parameter naming which head the user wrote. The canonical PascalCase forms share the same eval functions. So Stone 241.15 doesn't need to delete eval logic — only the DISPATCH ARMS that route the retired heads.

## What this stone delivers

### S1 — HARD-CUT-rejection arms at `src/check.rs` (3 arms)

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

### S3 — DELETE dispatch arms in runtime.rs

`src/runtime.rs:5694-5699`:
- DELETE `":wat::core::try" => eval_try(":wat::core::try", ...)` arm
- DELETE `":wat::core::option::expect" => eval_option_expect(":wat::core::option::expect", ...)` arm
- DELETE `":wat::core::result::expect" => eval_result_expect(":wat::core::result::expect", ...)` arm

The canonical forms (`Result/try`, `Option/expect`, `Result/expect`) keep their dispatch arms. Eval functions themselves UNCHANGED (they share op-param logic; just the retired-name dispatch arms go).

### S4 — DELETE deprecation-arm helper functions in check.rs

`src/check.rs:1832-1843`, `1851-1866`, `1874-1888` — three small helper functions that detect "is callee the retired head and emit warning." These were soft-deprecation infrastructure. Post-stone: HARD-CUT-rejection arms (S1) supersede. DELETE the helpers + their callers.

### S5 — DELETE dispatch arms in check.rs

`src/check.rs:5866-5874` — `":wat::core::try" => { ... infer_try(":wat::core::try", ...) }` dispatch arm DELETED. The `:wat::core::Result/try` arm at line 5918 STAYS (canonical form).

Similar for option::expect / result::expect dispatchers in check.rs (lines 2703-2734, 2823-2839): the dispatcher currently routes BOTH retired AND canonical heads. POST-STONE: route only canonical; retired forms HARD-CUT-rejected before reaching this code.

### S6 — DELETE special_forms.rs registry entries

`src/special_forms.rs`:
- Line 211 (`:wat::core::try` entry with `"<retired-use-Result/try>"` arity hint) — DELETED
- Line 209 (preceding comment about reflection uniformity) — DELETED (the rationale was "keep retired in help table"; post-stone the form is HARD CUT so this rationale doesn't apply)
- Line 349 (other reference to `:wat::core::try`) — judge per context; likely DELETE
- Line 214 (`:wat::core::option::expect` registry) — DELETED
- Line 219 (`:wat::core::result::expect` registry) — DELETED

### S7 — Doc cascade migration

**`docs/USER-GUIDE.md`** — 5+ sites:
- Line 1084, 1115, 2608, 2625, 3345, 3466 reference `:wat::core::try`: replace with `:wat::core::Result/try` in examples
- Line 1097: `:wat::core::option::expect` → `:wat::core::Option/expect`
- Line 1102: `:wat::core::result::expect` → `:wat::core::Result/expect`

**`docs/SERVICE-PROGRAMS.md`** — 8 sites:
- Lines 133, 134, 135, 194, 211, 213, 303, 305, 350, 351, 352 — all `:wat::core::option::expect` examples → `:wat::core::Option/expect`

**`docs/CLOJURE-ROSETTA.md:35`** — `:wat::core::try` Rosetta-stone row → `:wat::core::Result/try`

**`docs/WAT-CHEATSHEET.md`** — lines 217, 218 reference lowercase expect forms → PascalCase

### S8 — Reflection emitter audit

Per Stone 241.12/13/14 trap-door precedent:
```
grep -n "Keyword.*::try\b\|Keyword.*::option::expect\|Keyword.*::result::expect" src/
```

For any AST-construction site emitting these retired forms: migrate to canonical names.

### S9 — Probe verification

`tests/probe_arc241_stone15_zombie_purge.rs` (NEW). FM 2-bis disconfirming, 6 contracts.

### S10 — Author SCORE doc

Per `feedback_score_present_check_before_closure`. `SCORE-STONE-241.15.md` at strike-end.

## Locked decisions

### D1 — Three zombies bundled in ONE stone

Apparatus identical (HARD-CUT arm + RETIREMENT_TABLE entry + dispatch deletion + doc migration). Cascade bounded (no active wat/test callers). Calibration favors bundling small related work per `feedback_momentum_ordering`.

### D2 — Eval/infer functions UNCHANGED

`eval_try` / `eval_option_expect` / `eval_result_expect` / `infer_try` / etc. take op-param and share logic with canonical forms. ONLY the dispatch arms that ROUTE retired heads die. The functions themselves stay (they serve the canonical forms too).

### D3 — Deprecation-arm helpers DELETED

`check_deprecated_try_head` / `check_deprecated_option_expect_head` / `check_deprecated_result_expect_head` (or equivalent at check.rs:1832-1888) were the SOFT deprecation mechanism. Stone 241.15 supersedes with HARD CUT; the helpers retire.

### D4 — RETIREMENT_TABLE grows to 12 entries

Stone 241.14 made it 9; Stone 241.15 adds 10/11/12.

### D5 — Vigilia NOT required (no namespaced home)

### D6 — INTERSTITIAL orchestrator-exclusive (`feedback_sonnet_never_drafts_interstitial`)

### D7 — SCORE-write at end (`feedback_score_present_check_before_closure`)

### D8 — Stone 241.16 scope OFF-LIMITS

Sonnet does NOT touch `is_mutation_head`, `parse_define_form`, `register_define`, `is_define_form` — those are Stone 241.16 (Enemy 3 — define eval-time residue) scope.

## Trap-door audit

### T1 — Eval functions are shared with canonical forms; ensure deletion path doesn't break canonical

The `eval_try` / `eval_option_expect` / `eval_result_expect` functions accept op-param naming which head the user wrote. If sonnet accidentally deletes the function body or changes its signature, the canonical forms break. Resolution: only DELETE the dispatch arms (lines 5694, 5695, 5698); leave eval functions untouched. Run canonical-form tests post-strike.

### T2 — Dispatcher routing helpers (check.rs:2703-2734, 2823-2839) route BOTH retired AND canonical heads

Sonnet must surgically remove the `|| head_str == ":wat::core::option::expect"` (and similar) clauses. The canonical-routing path stays.

### T3 — special_forms.rs:349 unclear context

Line 349 has another `:wat::core::try` reference. Sonnet reads context; judges DELETE vs KEEP. Likely DELETE if part of an active table.

### T4 — Doc cascade is largest part of the work

USER-GUIDE.md + SERVICE-PROGRAMS.md combined have 15+ migration sites. Bulk sed could work IF the patterns are unambiguous (no overlap with `Result/try` / `Option/expect` / `Result/expect` strings that already exist in the docs). Per-site review recommended.

### T5 — Reflection emitters (Stone 241.12/13/14 trap-door class)

Grep first. Likely zero.

### T6 — Sonnet "stays as sugar / kept for help table" temptation

Per D1 + `feedback_hard_cut_admits_no_bypasses`. STOP if surfaces. The retirement is TOTAL.

## STOP triggers — REJECTION

1. Compile errors not traced to zombie deletion cascade
2. Lib < 890 (post-241.14 baseline)
3. **120 min elapsed** (this stone is SMALLER than 241.13/14; bounded scope)
4. holon-rs touched (STOP-5)
5. Any retired form classified as "stays for help table" / "stays as sugar" / "soft retirement preserved" without HARD CUT → D1 + `feedback_hard_cut_admits_no_bypasses` violation
6. Canonical forms (`Result/try` / `Option/expect` / `Result/expect`) BREAK due to eval/infer function damage (T1 violation)
7. Files outside permitted scope (`src/check.rs` / `src/runtime.rs` / `src/special_forms.rs` / `src/remedy/retirement.rs` / `src/closure_extract.rs` if reflection emitters touched / docs migration files / `tests/probe_arc241_stone15_*` / SCORE doc)
8. Stone 241.15 probe < 6/6
9. Stone 241.x or arc 237/238/242 probes regress
10. Clippy > 930 (looser gate; substrate refactor; arc 109 sweeps to zero)
11. Auto-fixer crate survives commit
12. Sonnet writes to INTERSTITIAL → D6 + `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.15.md NOT authored at end → D7 + `feedback_score_present_check_before_closure` violation
14. Stone 241.16 scope touched (`is_mutation_head` / `parse_define_form` / etc.) → D8 violation

## FM 2-bis evidence

`tests/probe_arc241_stone15_zombie_purge.rs` (NEW; 6 contracts; verified disconfirms at HEAD before BRIEF spawns).

## Calibration

**Target band: 60-120 min Mode A.**

Stone 241.15 scope decomposition:
- 3 HARD-CUT arms — **~10 min**
- 3 RETIREMENT_TABLE entries — **~5 min**
- Dispatch arm deletions (runtime.rs + check.rs) — **~10-15 min**
- Deprecation-arm helper deletions — **~10 min**
- special_forms.rs entry deletions — **~5 min**
- Doc cascade (USER-GUIDE + SERVICE-PROGRAMS + CLOJURE-ROSETTA + WAT-CHEATSHEET) — **~20-30 min**
- Reflection emitter audit — **~5 min**
- Pre-INSCRIPTION grep + final verification — **~10 min**
- SCORE doc authoring — **~10-15 min**

Within-band: 60-120 min. Under-band likely (Stone 241.13's 25-min + Stone 241.14's 26-min both substantially under-band; apparatus is mature).

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.14.md (analogous cascade pattern — substrate deletion + RETIREMENT_TABLE + HARD-CUT arm); SCORE-STONE-241.13.md (substrate scaffolding deletion pattern; clippy-down-with-deletion).

## What this unblocks

**Stone 241.16** — Enemy 3 (`:wat::core::define` eval-time residue completion; closes Stone 241.11's partial HARD CUT). With zombies dead, Enemy 3 gets focused attention.

**Stone 241.17** — INSCRIPTION closes arc 241 + `feedback_defer_by_naming` doctrine memory inscribed.

**Arc 237.8b** — reopens after Stone 241.17 per `feedback_no_regression_until_arc_done`

**One-canonical-path doctrine** — three more violations annihilated. `:wat::core::Result/try` is THE try form. `:wat::core::Option/expect` + `:wat::core::Result/expect` are THE expect forms. PascalCase Type/method canonical; lowercase-namespace duplicates DEAD.

**Future zombie audits** — the pattern from this stone (audit → annihilate → doc cascade) is the template for `feedback_wat_llm_first_design` enforcement. Any future "registered but retired" form gets HARD CUT, not kept.
