# BRIEF — Stone 241.12 — `:wat::core::defalias` mint + `:wat::runtime::define-alias` HARD CUT (Enemy 1 of 3)

You are sonnet. **Stone 241.12 of arc 241. Enemy 1 of 3 in the define-family death campaign.** Mints native `:wat::core::defalias` + HARD-CUTs `:wat::runtime::define-alias`. After this, Stone 241.13 (define-dispatch HARD CUT) + Stone 241.14 (define eval-time completion) + Stone 241.15 (INSCRIPTION) close arc 241.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## CRITICAL doctrine (pre-authorized — read these BEFORE strike)

1. **HARD CUT IS TOTAL** (`feedback_hard_cut_admits_no_bypasses`). The retired form `:wat::runtime::define-alias` dies EVERYWHERE in the substrate. There is NO privileged path. There is NO substrate-internal bypass. There is NO "stdlib uses it internally so it's OK." Stone 241.11.fix round 2 was KILLED for exactly this framing; do not repeat. If you find yourself classifying a use as "privileged path" or "intentional bypass" — STOP. The use migrates.

2. **ONE form, not two layers** (user direction 2026-05-29 late). Defalias is NATIVE substrate (parsed + registered in Rust); the `wat/runtime.wat:18` macro implementation DIES. Do NOT implement defalias as a wat-macro that compiles to `:wat::runtime::define-alias`.

3. **INTERSTITIAL is orchestrator-exclusive** (`feedback_sonnet_never_drafts_interstitial`). DO NOT write to `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` (or any INTERSTITIAL artifact). Even "draft for orchestrator review" framing is the violation in writing. Memory files OK (you can update `project_*` memory files if Stone 241.12 substantively extends a doctrine).

4. **SCORE-write is part of the stone** (`feedback_score_present_check_before_closure`). Author `SCORE-STONE-241.12.md` as the FINAL step before returning. Orchestrator verifies SCORE-doc-present before commit; missing SCORE = discipline-gap that requires reconstruction. Arc 242's Stone 242.2 surfaced this lesson — don't repeat.

5. **FM 16 sonnet bash firewall awareness** — keep bash patterns simple, one per line, vanilla cargo/git/grep. No chained pipes. If bash claim "denied" surfaces, run `which cargo` to verify (it's not denied; the firewall trips on complex patterns).

## What to do

### S1 — Mint `:wat::core::defalias` as NATIVE substrate form

Native parsing + registration in Rust. NOT a wat-macro compiling to runtime mechanism. Place the dispatch routing near existing def*-prefix forms in `src/types.rs` / `src/check.rs` (mirror `defstruct` / `defenum` placement).

Form shape:
```scheme
(:wat::core::defalias :new::name :original::name)
```

Two positional keyword args:
- `args[0]` — NEW name (the alias)
- `args[1]` — ORIGINAL name (the existing binding)

Both names exist post-defalias. Additive (no destruction of original). Works for both user-defined and built-in/intrinsic original bindings.

The substrate runtime mechanism (parse + register alias) lives in Rust. Look at the existing `:wat::runtime::define-alias` implementation in `src/runtime.rs` (referenced from comments at lines 4702, 11427, 13222, 13647) and `wat/runtime.wat:18` (the macro DEFINITION) — port the alias-registration logic into native Rust code; throw away the wat-macro layer.

### S2 — `:wat::runtime::define-alias` HARD CUT

The substrate has ONE alias form. The runtime form gets:

**HARD-CUT-rejection arm at `src/check.rs`** (mirror Stone 241.11's pattern):

```rust
":wat::runtime::define-alias" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.12); use ':wat::core::defalias' instead", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

Or use the walker-arm pattern from Stone 242.1's Char HARD CUT if positional behavior fits better. Judge per substrate inspection.

### S3 — Append 6th RETIREMENT_TABLE entry

`src/remedy/retirement.rs` extends to 6 entries:

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    (":wat::core::enum",              ":wat::core::defenum"),
    (":wat::core::define",            ":wat::core::defn"),
    (":wat::core::Char",              ":wat::core::char"),
    // Stone 241.12 — defalias replaces runtime define-alias.
    (":wat::runtime::define-alias",   ":wat::core::defalias"),
];
```

### S4 — Cascade migration of 13 surface callers (verified fresh grep)

**wat-source built-in (6 callers):**
- `wat/core.wat:52, 53, 54, 55` — 4 sites (dissoc/keys/values/concat aliases to HashMap/Vector long names)
- `wat/list.wat:16, 17` — 2 sites (reduce/fold aliases of foldl)

**Test source (6 callers):**
- `tests/wat_arc143_define_alias.rs:69, 95, 121` — 3 sites
- `tests/wat_arc144_uniform_reflection.rs:363` — 1 site
- `tests/wat_arc201_structured_signature_types.rs:299` — 1 site
- `tests/wat_arc221b_macro_support_keyword_shape.rs:206` — 1 site

**Substrate macro impl (1 site):**
- `wat/runtime.wat:18` — the macro DEFINITION of define-alias; DELETED per S1's native-implementation directive

Mechanical migration; form shape stays identical; only the head keyword changes from `:wat::runtime::define-alias` to `:wat::core::defalias`.

### S5 — Audit reflection emitters

Run:
```
grep -n "Keyword.*runtime::define-alias" src/runtime.rs src/closure_extract.rs src/check.rs
```

For each AST-construction site producing `:wat::runtime::define-alias` Keyword: migrate to emit `:wat::core::defalias` keyword.

### S6 — Fold in Stone 241.11.fix round 1's lost work (consistency pass)

Stone 241.11.fix round 1 (~17 min wall-clock per cliffnotes calibration) shipped 14 test migrations + 1 doc update before being lost during the 241.12 WIP discard. Fresh audit shows actual scope is larger:

**Tests with `:wat::core::define` in fixtures (~24 sites):**

```
grep -rn ":wat::core::define\b" tests/ --include="*.rs"
```

Classify each:

- **INTENTIONAL (preserve):**
  - `tests/probe_arc241_stone11_define_hard_cut.rs` — tests the HARD CUT itself; all uses preserve
  - `tests/wat_eval_result.rs:219` — assertion on error message content; preserve string literal
  - `tests/wat_arc144_special_forms.rs:210-211` — special-form table assertions; review per current registry state (may need update if defn replaced define in registry)
  - `tests/probe_declaration_form_lift.rs:127` — declaration-form list reference; review
- **CONSISTENCY (migrate to `:wat::core::defn`):**
  - `tests/probe_closure_body_prelude_lift.rs:129, 130, 161, 191, 224, 226, 277, 278`
  - `tests/wat_arc170_program_contracts.rs:346` (+ comment at 416)
  - `tests/wat_eval_result.rs:96, 171, 195`
  - `tests/probe_spawn_process_parent_type.rs:134, 184, 245`
  - `tests/arc112_slice2b_process_send_recv.rs:60`
  - `tests/arc112_scheme_probe.rs:37`
  - `tests/wat_arc170_closure_extraction.rs:1064` — Rust string literal `if head == ":wat::core::define"`; review whether assertion target updates
  - `tests/wat_arc144_uniform_reflection.rs:121-122` — assertion on reflection AST naming; review whether reflection emits defn now

**Docs with `:wat::core::define` examples (~10 sites; migrate function-shape examples to `:wat::core::defn`):**
- `docs/CIRCUIT.md:20`
- `docs/CONVENTIONS.md:763`
- `docs/SERVICE-PROGRAMS.md:51, 77, 109, 118, 185, 237, 293, 383`

Per-site judgment based on context (function-shape → defn; alias-shape rare → defalias; INTENTIONAL preserve per rejection-testing purpose).

### S7 — Probe verification

`tests/probe_arc241_stone12_defalias.rs` (STRIKE-READY-v2; already committed). 5 contracts; pre-stone **5/5 FAIL at HEAD** (verified via `cargo test --release --test probe_arc241_stone12_defalias` 2026-05-29 very late). Post-stone: 5/5 PASS.

### S8 — Pre-INSCRIPTION grep gate (Stone 241.12-specific scope)

After all migrations, run:
```
grep -rn ":wat::runtime::define-alias\b" src/ tests/ wat/
```

Acceptable categories post-stone:
1. `src/check.rs` — HARD-CUT-rejection arm
2. `src/remedy/retirement.rs` — RETIREMENT_TABLE entry
3. Historical comments in any file (e.g., comments describing the retirement)
4. Stone 241.12 probe source (tests the rejection)

Goal: 0 ACTIVE uses outside acceptable categories.

### S9 — Author SCORE-STONE-241.12.md

Per `feedback_score_present_check_before_closure`. Path: `docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.12.md` (NOT repo root; NOT INTERSTITIAL).

Mirror SCORE-STONE-241.11.md shape. Include:
- Header (Mode A; runtime; cascade size; auto-fixer used + deleted?)
- Phase A scorecard (probe + lib + clippy + structural)
- Migration cascade audit (13 surface callers per-site; consistency pass count; doc updates count)
- HARD CUT arm verbatim
- RETIREMENT_TABLE post-stone (6 entries verbatim)
- Pre-INSCRIPTION grep verification (target: 0 active matches)
- Honest deltas (anything surfaced)
- Calibration (predicted vs actual; per-class runtime)
- What this unblocks (Stone 241.13 — Enemy 2)
- NO Vigilia section (D6 — no namespaced home)

## Discipline

- HARD CUT TOTAL — no internal bypasses; no privileged paths (per `feedback_hard_cut_admits_no_bypasses`)
- ONE form, not two layers — native defalias; wat/runtime.wat:18 macro DELETED
- `src/argspec/*`, `src/lib.rs` UNCHANGED
- `src/remedy/retirement.rs` MODIFIED (append 6th entry per S3); other `src/remedy/*` unchanged
- Stone 241.x probes preserved; arc 237/238/242 probes preserved
- holon-rs NEVER touched (STOP-5)
- No new error variants
- Auto-fixer crate (if used) must be EPHEMERAL — DELETED before commit (per Stone 241.10/241.11 precedent)
- DO NOT write to INTERSTITIAL (D9; `feedback_sonnet_never_drafts_interstitial`)
- SCORE doc authored at end (D8; `feedback_score_present_check_before_closure`)
- Pre-INSCRIPTION grep gate (S8) CLEAN post-stone

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.12.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.12.md` — D1-D9 + T1-T7 + STOP
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.11.md` — cascade discipline + auto-fixer ephemeral pattern + bandaid-rip-with-receipts
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.10.md` — substrate-mint shape (defalias parser structurally similar to defstruct/defenum)
7. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/SCORE-STONE-242.1.md` — HARD-CUT walker-arm placement
8. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/SCORE-STONE-242.2.md` — SCORE-discipline lesson (the gap that produced D8)
9. `/home/watmin/work/holon/wat-rs/src/types.rs` — find existing def*-prefix dispatch (defstruct/defenum)
10. `/home/watmin/work/holon/wat-rs/src/check.rs` — find existing HARD-CUT-rejection arms (struct/struct-restricted/enum/define/Char)
11. `/home/watmin/work/holon/wat-rs/src/runtime.rs` — find existing `:wat::runtime::define-alias` runtime mechanism (referenced at lines 4702, 11427, 13222, 13647); port to native defalias parser+registrar
12. `/home/watmin/work/holon/wat-rs/wat/runtime.wat` — read the macro DEFINITION of define-alias (line 18); plan deletion
13. `/home/watmin/work/holon/wat-rs/src/remedy/retirement.rs` — RETIREMENT_TABLE shape
14. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone12_defalias.rs` — 5-contract probe (5/5 disconfirms at HEAD)

## Cadence

1. **Baseline:** `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test probe_arc241_stone12_defalias 2>&1 | tail -3` (expect 0/5)
2. **S1:** mint `:wat::core::defalias` native parser + registrar (port from existing runtime mechanism)
3. **S4 partial:** delete `wat/runtime.wat:18` macro impl
4. **S2:** add check.rs HARD-CUT arm for `:wat::runtime::define-alias`
5. **S3:** append 6th RETIREMENT_TABLE entry
6. **S4 remainder:** migrate 13 surface callers (wat/core.wat × 4, wat/list.wat × 2, test source × 6, then verify wat/runtime.wat:18 deleted accounted for)
7. **S5:** audit + migrate reflection emitters
8. **S6:** fold in Stone 241.11.fix round 1 lost work (~24 test sites + ~10 docs; per S6 classification)
9. **Cascade iteration:** per `docs/SUBSTRATE-AS-TEACHER.md` — read failure → migrate site → re-run
10. **S7:** verify probe 5/5 PASS
11. **S8:** pre-INSCRIPTION grep gate CLEAN
12. **Final verification:** lib ≥ 890; workspace test-build clean (`cargo build --release --tests --workspace`); clippy ≤ 902
13. **S9:** author `SCORE-STONE-241.12.md` at `docs/arc/2026/05/241-function-signature-unification/`
14. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

1. Compile errors not traced to defalias mint or alias cascade
2. Lib < 890
3. **150 min elapsed**
4. holon-rs touched (STOP-5)
5. `:wat::runtime::define-alias` use classified as "privileged path" / "intentional bypass" / "substrate-internal exception" without migration → D7 + `feedback_hard_cut_admits_no_bypasses` violation
6. `:wat::runtime::define-alias` survives as ACTIVE substrate use post-stone (outside HARD-CUT arm + retirement entry + historical comments)
7. Files outside permitted scope (`src/types.rs` / `src/check.rs` / `src/freeze.rs` / `src/runtime.rs` / `src/remedy/retirement.rs` / `src/closure_extract.rs` / `src/stdlib.rs` / `wat/*.wat` / cascade target tests + docs / `tests/probe_arc241_stone12_*` / SCORE doc)
8. Stone 241.12 probe < 5/5
9. Stone 241.x or arc 237/238/242 probes regress
10. Clippy > 902
11. Auto-fixer crate survives commit (Stone 241.10/241.11 ephemeral discipline)
12. Sonnet writes to INTERSTITIAL → D9 + `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.12.md NOT authored at end → D8 + `feedback_score_present_check_before_closure` violation

## Post-strike return

Return one paragraph: defalias minted (native; parser+registrar at <file:line>); 13 surface callers migrated (wat/core.wat × 4, wat/list.wat × 2, tests × 6, wat/runtime.wat:18 DELETED); reflection emitters audited + migrated (count); consistency pass count (test sites + doc sites); HARD-CUT arm at <file:line>; RETIREMENT_TABLE = 6 entries; pre-INSCRIPTION grep CLEAN (active uses = 0); Stone 241.12 probe 5/5; lib 890/0 (or higher); clippy ≤ 902; auto-fixer status (built? used? DELETED?); SCORE doc path at arc dir.

Arc 241 continues at Stone 241.13 (Enemy 2 — define-dispatch) after this. Strike clean.
