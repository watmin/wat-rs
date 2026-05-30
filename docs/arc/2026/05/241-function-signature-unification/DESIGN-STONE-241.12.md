# DESIGN — Stone 241.12 — `:wat::core::defalias` mint + `:wat::runtime::define-alias` HARD CUT

**Status:** STRIKE-READY-v2 (2026-05-29 very late). Stone 241.12 mints the missing def*-prefix-family surface form for binding aliases AND retires `:wat::runtime::define-alias` per user direction. INSCRIPTION moves to Stone 241.15 (was 241.13 in v1 — see § Battle plan). **Enemy 1 of 3** in the define-family death campaign.

## STRIKE-READY-v2 context (2026-05-29 very late)

V1 artifacts at commit `e803e0f9` were contaminated:

1. **Probe carried "26 callers stays / surface compiles to runtime" framing** — privileged-path violation per `feedback_hard_cut_admits_no_bypasses`
2. **Probe assertions too weak** — bare `is_ok()` on startup tolerated unknown-form-as-no-op; failed to disconfirm at HEAD on C01/C02 (sharpened in v2)
3. **DESIGN+BRIEF S3/S4 sections audited substrate-internal `:wat::core::define` uses** — moot since Stone 241.11 already HARD-CUT define at startup; the real target this stone is `:wat::runtime::define-alias`
4. **No three-enemy battle plan context** — Stone 241.12 didn't know it was first of three define-family deaths
5. **No explicit SCORE-write directive** — new `feedback_score_present_check_before_closure` (2026-05-29 very late) inscribed after arc 242 paperwork-gap surfaced
6. **Caller count wrong** — V1 said "26 callers"; fresh grep shows 13 active surface callers

V2 corrects all six. Probe rewritten + verified 5/5 disconfirm at HEAD. All artifacts realigned.

## Battle plan — three enemies, one campaign

Per user direction 2026-05-29 very late: *"define-alias, define-dispatch, define all die - we need to figure out the teardown order - if we need defer define being killed to make defclause first to migrate the dispatch callers so be it"* + *"this is the fight now - we know we have multiple enemies - we manage the fight - we are the best - we prove it relentlessly."*

| Stone | Enemy | Replacement | Cascade scope | Status |
|---|---|---|---|---|
| **241.12 (this)** | `:wat::runtime::define-alias` | `:wat::core::defalias` (mint native) | 13 surface callers + macro impl at `wat/runtime.wat:18` + fold-in Stone 241.11.fix round 1 lost work (~24 tests + ~10 docs) | STRIKE-READY-v2 |
| 241.13 | `:wat::core::define-dispatch` | `:wat::core::defclause` (exists; Stone 237.2 SHIPPED `bdd9eb6c`) | substrate scaffolding deletion only (`src/dispatch.rs` ~330 lines + `special_forms.rs:194` + `freeze.rs` walker arms); wat-source callers already migrated to ∀T intrinsics per arc 237.7 | DESIGN TBD after 241.12 ships |
| 241.14 | `:wat::core::define` residue | (already retired Stone 241.11) | substrate eval-time scaffolding deletion (`is_mutation_head` arm + `parse_define_form` + `register_define` walker + `is_define_form` check); 2-3 freeze.rs test fixtures migrate to a different known mutation head | DESIGN TBD after 241.13 ships |
| 241.15 | INSCRIPTION | — | Arc 241 closes; arc 237.8b reopens | — |

**Independence verdict:** all three enemies are TECHNICALLY independent. Defclause already exists (Stone 237.2 SHIPPED `bdd9eb6c`); dispatch callers already migrated per arc 237.7's evacuation of length/empty?/contains?/get/conj/assoc to Rust ∀T intrinsics; the user's contingency "defer define killing to make defclause first" is MOOT (defclause is already live).

**Serial chosen over bundled** per four-questions discipline: bundled fails Obvious + Simple (three concerns mixed); serial passes all four — each stone has one enemy, one cascade, one SCORE, calibration discipline per enemy, rollback per enemy.

**Order rationale:** 241.12 first because (a) momentum per `feedback_momentum_ordering` (STRIKE-READY artifacts exist already), (b) smallest dependency surface (Enemy 1 doesn't need Enemy 2 or 3 to land first), (c) the existing 241.12 work products survive the rewrite.

## Why this stone

Stone 241.11.fix round 2 was KILLED mid-strike because the substrate's internal `:wat::core::define` uses for ALIAS bindings cannot honestly migrate to `:wat::core::defn` (wrong shape) nor `:wat::core::def` (loses alias semantics). User direction 2026-05-29 late: *"define must die - there is no option - there is def and defn"* + *"define-alias dies to allow define to die."*

The substrate's def*-prefix family is missing one member. Intueri cast 2026-05-29 late locked **`defalias`** (L0 + REMARKABLE): term-of-art across Emacs Lisp / Common Lisp / Clojure / Racket; near-instant cold-read recognition; mirrors the existing `:wat::runtime::define-alias` substrate mechanism.

User direction 2026-05-29 late: *"at the end of this work :wat::runtime::define-alias is dead - :wat::core::defalias is the only way to do name aliasing."*

This satisfies `feedback_hard_cut_admits_no_bypasses` AT THE RUNTIME LAYER too — no substrate-internal alias mechanism survives separately from the user-facing form. ONE form; ONE mechanism; ONE name.

## What this stone delivers

### S1 — Mint `:wat::core::defalias` as NATIVE substrate form

Native parsing + registration in Rust (NOT a wat-macro compiling to runtime mechanism). The user's "ONE form, not two layers" directive demands native implementation — the `wat/runtime.wat:18` macro IMPLEMENTATION of define-alias dies; defalias is parsed + registered directly in Rust.

Form shape:
```scheme
(:wat::core::defalias :new::name :original::name)
```

Two positional keyword args:
- `args[0]` — NEW name (the alias)
- `args[1]` — ORIGINAL name (the existing binding)

Both names exist post-defalias. Additive (no destruction of original). Works for both user-defined and built-in/intrinsic original bindings (probe C03 exercises the built-in pattern from `wat/core.wat`).

Place the dispatch routing near existing def*-prefix forms (`defstruct`/`defenum` routing at `src/types.rs` / `src/check.rs`).

### S2 — `:wat::runtime::define-alias` HARD CUT

The substrate has ONE alias form. The runtime form gets:
- HARD-CUT-rejection arm at `src/check.rs` (mirror Stone 241.8/9/11 + arc 242.1 shape)
- 6th RETIREMENT_TABLE entry
- 13 active surface callers migrate (cascade — see S3)
- Substrate macro impl at `wat/runtime.wat:18` — DELETED (replaced by native defalias parser/registrar)

Mirror Stone 241.11 HARD-CUT-arm pattern:

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

Or use the walker-arm pattern from Stone 242.1's Char HARD CUT if positional behavior matches better. Sonnet judges per substrate inspection.

### S3 — Cascade migration of 13 surface callers (verified fresh grep)

**wat-source built-in (6 callers):**
- `wat/core.wat:52` — `(:wat::runtime::define-alias :wat::core::dissoc :wat::core::HashMap/dissoc)`
- `wat/core.wat:53` — `(:wat::runtime::define-alias :wat::core::keys :wat::core::HashMap/keys)`
- `wat/core.wat:54` — `(:wat::runtime::define-alias :wat::core::values :wat::core::HashMap/values)`
- `wat/core.wat:55` — `(:wat::runtime::define-alias :wat::core::concat :wat::core::Vector/concat)`
- `wat/list.wat:16` — `(:wat::runtime::define-alias :wat::list::reduce :wat::core::foldl)`
- `wat/list.wat:17` — `(:wat::runtime::define-alias :wat::list::fold :wat::core::foldl)`

**Test source (6 callers):**
- `tests/wat_arc143_define_alias.rs:69, 95, 121` — 3 sites (the original arc 143 acceptance tests)
- `tests/wat_arc144_uniform_reflection.rs:363` — 1 site (length canary)
- `tests/wat_arc201_structured_signature_types.rs:299` — 1 site
- `tests/wat_arc221b_macro_support_keyword_shape.rs:206` — 1 site

**Substrate macro impl (1 site):**
- `wat/runtime.wat:18` — the macro DEFINITION of define-alias; DELETED per S1's native-implementation directive

Mechanical migration; the form shape stays identical (two positional keywords); only the head keyword changes.

### S4 — Append RETIREMENT_TABLE entry (6th)

`src/remedy/retirement.rs` `RETIREMENT_TABLE` extends to 6 entries:

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted.
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    // Stone 241.9 — defenum replaces enum.
    (":wat::core::enum",              ":wat::core::defenum"),
    // Stone 241.11 — defn replaces define.
    (":wat::core::define",            ":wat::core::defn"),
    // Stone 242.1 — char (lowercase) replaces Char (Doctrine 2).
    (":wat::core::Char",              ":wat::core::char"),
    // Stone 241.12 — defalias replaces runtime define-alias.
    (":wat::runtime::define-alias",   ":wat::core::defalias"),
];
```

### S5 — Fold in Stone 241.11.fix round 1's lost work

Stone 241.11.fix round 1 (~17 min wall-clock per cliffnotes calibration table) shipped 14 test migrations + 1 doc update before being lost during the 241.12 WIP discard. Fresh audit 2026-05-29 very late shows the count was UNDER-counted; actual scope:

**Tests with `:wat::core::define` in fixtures (~24 sites across many files):**
- **INTENTIONAL (preserve):**
  - `tests/probe_arc241_stone11_define_hard_cut.rs` — tests the HARD CUT itself
  - `tests/wat_eval_result.rs:219` — assertion on error message content (`message.contains(":wat::core::define")`)
  - `tests/wat_arc144_uniform_reflection.rs:121-122` — assertion on reflection AST naming (may need review; if reflection emits :wat::core::defn now, this assertion is stale)
  - `tests/wat_arc144_special_forms.rs:210-211` — special-form table assertions (review per current registry state)
  - `tests/probe_declaration_form_lift.rs:127` — declaration-form list reference
- **CONSISTENCY (migrate to `:wat::core::defn`):**
  - `tests/probe_closure_body_prelude_lift.rs:129, 130, 161, 191, 224, 226, 277, 278`
  - `tests/wat_arc170_program_contracts.rs:346`
  - `tests/wat_eval_result.rs:96, 171, 195`
  - `tests/probe_spawn_process_parent_type.rs:134, 184, 245`
  - `tests/arc112_slice2b_process_send_recv.rs:60`
  - `tests/arc112_scheme_probe.rs:37`
  - `tests/wat_arc170_closure_extraction.rs:1064` (Rust string literal comparing head to `:wat::core::define` — review whether assertion target should update)
- **POTENTIAL ALIAS-SHAPE (rare; sonnet judges per-site):** any `(:wat::core::define :name :existing)` pattern → `:wat::core::defalias`

**Docs with `:wat::core::define` examples (~10 sites):**
- `docs/CIRCUIT.md:20`
- `docs/CONVENTIONS.md:763`
- `docs/SERVICE-PROGRAMS.md:51, 77, 109, 118, 185, 237, 293, 383`

All function-shape examples migrate to `:wat::core::defn`. The doc count was undercounted in cliffnotes ("1 doc update"); actual is ~10.

### S6 — Probe verification

`tests/probe_arc241_stone12_defalias.rs` (STRIKE-READY-v2). 5 contracts; all FAIL at HEAD (verified 2026-05-29 very late):
- C01: defalias alias name resolves (callable from fn body)
- C02: defalias additive (both alias + original callable)
- C03: defalias works for built-in bindings (wat/core.wat pattern)
- C04: define-alias HARD-CUT-rejected at startup
- C05: rejection carries structured retirement remedy naming defalias

Post-stone: 5/5 PASS.

### S7 — Authorial SCORE doc

Per new memory `feedback_score_present_check_before_closure` (inscribed 2026-05-29 very late from arc 242's paperwork-gap closure): every shipped stone gets a SCORE-STONE-N.md. Sonnet AUTHORS it at end of strike, not orchestrator post-hoc. This is part of the deliverable set; closure verification depends on it.

## Locked decisions

### D1 — Mint `:wat::core::defalias` as NATIVE substrate form (not wat-macro)

Per user directive "ONE form, not two layers." The `wat/runtime.wat:18` macro DIES. Defalias is parsed + registered directly in Rust, mirror to defstruct/defenum native routing.

### D2 — Form shape: `(defalias :new-name :original-name)`

Two positional keyword args. Both names exist post-stone (alias additive). No metadata-map this stone (defalias is simple).

### D3 — `:wat::runtime::define-alias` DIES totally (user direction)

Substrate has ONE alias form. 13 active surface callers migrate. HARD-CUT arm at check.rs. 6th RETIREMENT_TABLE entry.

### D4 — Native implementation, NOT wat-macro compiling to runtime mechanism

Per S1 + D1. The substrate runtime mechanism (parse + register alias) lives in Rust; no wat-source macro intermediates. Removes the two-layer model entirely.

### D5 — Fold in Stone 241.11.fix round 1 lost work

Tests + docs consistency pass per S5. ~24 test sites + ~10 doc sites. Per-site classification (INTENTIONAL preserve / CONSISTENCY migrate to defn / POTENTIAL ALIAS-SHAPE to defalias).

### D6 — Vigilia NOT required (D7 default per `feedback_namespaced_home_vigilia_gate`)

This stone does NOT mint a new namespaced home; substrate edits live in legacy flat substrate. SCORE-green commit.

### D7 — Per `feedback_hard_cut_admits_no_bypasses`: no privileged paths

Sonnet MUST NOT classify any substrate use as "privileged path" / "intentional bypass" / "substrate-internal exception" / "stdlib needs it." All 13 surface callers migrate; no exceptions.

### D8 — Per `feedback_score_present_check_before_closure`: SCORE-write is part of the stone

The BRIEF EXPLICITLY directs sonnet to author `SCORE-STONE-241.12.md` as the FINAL step before returning. Orchestrator verifies SCORE-doc-present before commit.

### D9 — Per `feedback_sonnet_never_drafts_interstitial`: INTERSTITIAL is orchestrator-exclusive

Sonnet MUST NOT write to `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` (or any INTERSTITIAL artifact). Even "draft for orchestrator review" framing is the violation in writing. Orchestrator authors INTERSTITIAL after Stone 241.12 ships.

## Trap-door audit

### T1 — Built-in aliasing semantics (D1 native implementation)

Built-in bindings live in Rust registries; defalias must register the alias in the SAME registry so resolution works uniformly. Probe C03 exercises this; if it doesn't pass after the native parser+registrar mint, sonnet has hit T1.

Resolution: per `feedback_trap_door_build_the_dependency` — extend the registry to handle alias entries; do NOT special-case built-ins out of scope.

### T2 — `wat/runtime.wat:18` macro deletion may break other macros that compose with define-alias

If any wat-source code depends on the MACRO form (not the keyword form) of define-alias — e.g., other macros calling it — those break. Audit via grep.

Resolution: per S3, all callers migrate to the new native form. If macros compose, they migrate too.

### T3 — Cascade migration touches `wat/runtime.wat` (substrate bootstrap root)

`wat/runtime.wat` is the substrate's BOOTSTRAP — careful with what we delete. The macro definition at line 18 is replaced (or removed entirely if no other consumers need it).

### T4 — Test fixtures using `:wat::core::define` as alias-shape (rare)

Round 1's lost work was function-shape migrations primarily. If any test fixture uses `(define :name :existing)` as an ALIAS pattern, sonnet judges → `:wat::core::defalias` migration. Likely zero such sites (alias was always done via `:wat::runtime::define-alias`).

### T5 — Reflection emitters producing `:wat::runtime::define-alias` AST

If any Rust reflection code constructs `(:wat::runtime::define-alias ...)` AST for emission, it migrates to `(:wat::core::defalias ...)` AST. Audit via `grep -n "Keyword.*runtime::define-alias" src/`.

### T6 — Sonnet self-audit of "privileged path" temptation (D7 + Stone 241.11.fix round 2 lesson)

Per `feedback_hard_cut_admits_no_bypasses`. STOP triggers list this explicitly. Round 2 was killed for this; do not repeat. Sonnet's per-site judgment must terminate in "migrate" or "INTENTIONAL preserve per S5 classification" — never "privileged."

### T7 — Sonnet auto-fixer crate temptation

Stone 241.10/241.11 ephemeral discipline applies. If sonnet decides to build an auto-fixer for the consistency pass, it must be DELETED before commit. Acceptable: build, use, delete (per 241.11 precedent).

## STOP triggers — REJECTION

1. Compile errors not traced to defalias mint or alias cascade
2. Lib < 890 (Stone 241.11 + arc 242 baseline)
3. **150 min elapsed** (Stone 241.12 upper bound)
4. holon-rs touched (STOP-5)
5. Substrate `:wat::runtime::define-alias` use classified as "privileged path" / "intentional bypass" without migration → D7 + `feedback_hard_cut_admits_no_bypasses` violation (Stone 241.11.fix round 2 was killed for exactly this)
6. `:wat::runtime::define-alias` survives as ACTIVE substrate use post-stone (any caller still using it outside HARD-CUT arm + retirement entry + historical comments) — D3 + user direction violation
7. Files outside permitted scope (`src/types.rs` / `src/check.rs` / `src/freeze.rs` / `src/runtime.rs` / `src/remedy/retirement.rs` / `wat/*.wat` / cascade target tests + docs / `tests/probe_arc241_stone12_*` / SCORE doc)
8. Stone 241.12 probe < 5/5
9. Stone 241.x or arc 237/238/242 probes regress
10. Clippy > 902
11. Auto-fixer crate survives commit (Stone 241.10/241.11 ephemeral discipline)
12. Sonnet writes to INTERSTITIAL → D9 + `feedback_sonnet_never_drafts_interstitial` violation
13. SCORE-STONE-241.12.md NOT authored at end of strike → D8 + `feedback_score_present_check_before_closure` violation

## FM 2-bis evidence

`tests/probe_arc241_stone12_defalias.rs` STRIKE-READY-v2; 5 contracts; **5/5 disconfirm at HEAD verified 2026-05-29 very late**. Each contract fails for the specific gap it tests:
- C01-C03: `UnresolvedReference :app::salutation` / `:user::my-length` (defalias unknown; aliases not registered)
- C04: startup succeeds (`:wat::runtime::define-alias` is still the live macro)
- C05: no rejection error to inspect

## Calibration

**Target band: 60-150 min Mode A.**

Stone 241.12's cascade (13 surface callers + ~24 test fixtures + ~10 docs) is bounded but larger than the bare 13-caller count suggests because of S5 fold-in. The dominant runtime variables:
- Native defalias parser + registrar implementation (~50-100 lines mirroring defstruct/defenum patterns)
- 13-caller mechanical migration (~20 min)
- S5 consistency pass (~24 test sites + ~10 docs; ~30-45 min)
- HARD CUT arm + RETIREMENT_TABLE append (~10 min)
- SCORE doc authoring (~10-15 min)

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.11.md for cascade discipline + auto-fixer ephemeral pattern + bandaid-rip-with-receipts; SCORE-STONE-241.10.md for substrate-mint shape (defalias parser structurally similar to defstruct/defenum parsing); SCORE-STONE-242.1.md for HARD-CUT walker-arm placement decision; SCORE-STONE-242.2.md for the SCORE-discipline lesson.

## What this unblocks

**Stone 241.13** — Enemy 2 (`:wat::core::define-dispatch` HARD CUT; pure substrate scaffolding deletion)

**Stone 241.14** — Enemy 3 (`:wat::core::define` eval-time residue completion)

**Stone 241.15** — INSCRIPTION closes arc 241

**Arc 237.8b** — reopens after Stone 241.15 per `feedback_no_regression_until_arc_done`

**The def*-prefix family completes** — def / defn / defclause / defmacro / defstruct / defenum / defalias all shipping native; defrecord queued arc 227; deftypealias queued arc 109.
