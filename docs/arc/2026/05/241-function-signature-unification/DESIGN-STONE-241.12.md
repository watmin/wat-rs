# DESIGN — Stone 241.12 — `:wat::core::defalias` mint + alias-cascade completion (define-alias dies to allow define to die)

**Status:** READY (sub-DESIGN). NEW Stone 241.12 — mints the missing def*-prefix-family surface form for binding aliases. INSCRIPTION moves to Stone 241.13. Renumbering per user direction ("renumbering feels honest"); precedent: 241.10 absorbed scope mid-design.

## Why this stone

Stone 241.11.fix round 2 was KILLED mid-strike because the substrate's internal `:wat::core::define` uses for ALIAS bindings cannot honestly migrate to `:wat::core::defn` (wrong shape — alias is not function binding) nor `:wat::core::def` (loses alias semantics). User direction 2026-05-29 late: *"define must die - there is no option - there is def and defn"* + *"define-alias dies to allow define to die."*

The substrate's def*-prefix family is missing one member. Intueri cast 2026-05-29 late locked **`defalias`** (L0 + REMARKABLE per `feedback_namespaced_home_vigilia_gate` precedent applied at form-naming layer): term-of-art across Emacs Lisp / Common Lisp / Clojure / Racket; near-instant cold-read recognition; closes the loop with existing `:wat::runtime::define-alias` substrate mechanism (the runtime IS named `define-alias`; surface mirrors at def\*-prefix tier).

Without `defalias`, the substrate cannot complete the HARD CUT discipline doctrine (per `feedback_hard_cut_admits_no_bypasses`): every retired form must die EVERYWHERE; no privileged paths. Stone 241.12 mints the missing form that makes that doctrine satisfiable.

## What this stone delivers

### S1 — Mint `:wat::core::defalias` user-facing surface form

New entry in `classify_type_decl` (or equivalent dispatch site at the user-surface layer) at `src/types.rs` or `src/check.rs` (wherever def*-prefix forms are routed). The form's shape:

```scheme
(:wat::core::defalias :new::name :original::name)
```

Two positional keyword args:
- `args[0]` — the NEW name (the alias being defined)
- `args[1]` — the ORIGINAL name (the existing binding being aliased)

Both names exist post-stone; alias is additive (no destruction of original).

### S2 — `:wat::runtime::define-alias` DIES; `:wat::core::defalias` is the ONLY alias form

**User direction 2026-05-29 late: *"at the end of this work :wat::runtime::define-alias is dead - :wat::core::defalias is the only way to do name aliasing."***

The substrate has ONE alias form, not two layers. Stone 241.12 mints `:wat::core::defalias` AND retires `:wat::runtime::define-alias` (26 callers per `grep -rn ":wat::runtime::define-alias\b"`). The 26 callers migrate to `:wat::core::defalias`.

This satisfies `feedback_hard_cut_admits_no_bypasses` at the runtime-tier layer too — no substrate-internal alias mechanism survives separately from the user-facing form. ONE form; ONE mechanism; ONE name.

Implementation: defalias becomes the parser + the registration code path. There is no separate compile-to-runtime-form step. The substrate uses defalias directly.

### S3 — Cascade migration of substrate `:wat::core::define` uses that are aliases

Stone 241.11.fix round 2 surfaced (before being killed) substrate-internal `:wat::core::define` uses that are FUNCTIONALLY aliases — they bind a new name to an existing function/value rather than creating a new function. These were the "Category B: 0 deletions because privileged path" sites that round 2 misclassified.

Audit pattern: `:wat::core::define` whose body is a SINGLE keyword referencing an existing binding (not a function literal, not a value expression — just a name redirect). These migrate to `:wat::core::defalias`.

Example shape (illustrative):
```scheme
;; LEGACY (substrate-internal use of define as alias)
(:wat::core::define :ns::new-name :ns::existing-name)

;; NEW (defalias)
(:wat::core::defalias :ns::new-name :ns::existing-name)
```

Sonnet's per-site audit determines which `:wat::core::define` uses qualify as aliases.

### S4 — Reflection emitters producing `:wat::core::define` AST for aliases

Reflection emitters in `src/runtime.rs` (and possibly elsewhere) produce `(:wat::core::define ...)` AST. Where those emissions are aliases (per S3 pattern), migrate to emit `(:wat::core::defalias ...)` AST. Sonnet audits + migrates.

### S5 — Cascade migration of `:wat::runtime::define-alias` (26 callers)

Per S2 (`:wat::runtime::define-alias` dies), the 26 callers migrate to `:wat::core::defalias`. Mechanical migration; the form shape stays the same (two positional keywords); only the head changes.

```scheme
;; LEGACY (substrate runtime mechanism — DIES this stone)
(:wat::runtime::define-alias :new::name :original::name)

;; NEW (only alias form post-stone)
(:wat::core::defalias :new::name :original::name)
```

### S6 — HARD-CUT-rejection arm for `:wat::runtime::define-alias`

Mirror the Stone 241.8/241.9/241.11 HARD-CUT-arm pattern. Add to `src/check.rs`:

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

### S7 — Append retirement-table entry

`src/remedy/retirement.rs` `RETIREMENT_TABLE` extends to 5 entries:

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    // Stone 241.9 — defenum replaces enum
    (":wat::core::enum",              ":wat::core::defenum"),
    // Stone 241.11 — defn replaces define
    (":wat::core::define",            ":wat::core::defn"),
    // Stone 241.12 — defalias replaces runtime define-alias
    (":wat::runtime::define-alias",   ":wat::core::defalias"),
];
```

NOTE: `:wat::core::define-alias` never existed as user-facing form (0 references); no separate entry needed for it.

```rust
(":wat::core::struct",            ":wat::core::defstruct"),
(":wat::core::struct-restricted", ":wat::core::defstruct"),
(":wat::core::enum",              ":wat::core::defenum"),
(":wat::core::define",            ":wat::core::defn"),
```

The substrate teaches via the existing mechanism. defalias's only retirement-table interaction: if the migration introduces any cases where a user wrote `define` for an alias and gets the standard `did you mean :wat::core::defn?` remedy — that's still correct (defn is the function-form replacement; defalias is the alias-form alternative). Per the bandaid-rip-with-receipts pattern: the structured retirement remedy serves whatever the user typed.

### S6 — Complete the Stone 241.11.fix gap

The pre-INSCRIPTION grep (per FM 11 + Stone S11 of recovery doc) must return 0 non-acceptable `:wat::core::define` matches after Stone 241.12 ships. Acceptable patterns:
1. `src/check.rs` HARD-CUT-rejection error text
2. `src/remedy/retirement.rs` table entries
3. Historical comments
4. `tests/probe_arc241_stone11_define_hard_cut.rs` test source (tests the HARD CUT)
5. `src/remedy/mod.rs` retirement_lookup fixtures
6. Predicates retaining the form name ONLY for HARD-CUT-rejection routing

All other substrate references migrate. Per `feedback_hard_cut_admits_no_bypasses`: no privileged paths.

### S7 — Probe verification

`tests/probe_arc241_stone12_defalias.rs` (NEW). FM 2-bis disconfirming. Contracts:
1. `(defalias :new::name :original::name)` startup clean (defalias works)
2. After defalias, `:new::name` and `:original::name` resolve to the SAME binding (additive)
3. Legacy `(:wat::core::define :new :existing)` for aliases still works at HEAD (the substrate's internal uses) — post-stone these MAY be HARD-CUT-rejected (sonnet judges; this is a substrate-internal concern, not user-facing HARD CUT)
4. Reflection emitters produce defalias AST for alias bindings (post-stone)

## Locked decisions

### D1 — Mint `:wat::core::defalias` as user-facing surface form

Per intueri cast 2026-05-29 late: `defalias` is L0 + REMARKABLE. Term-of-art across Lisp family; cold read recognition near-instant; mirrors existing `:wat::runtime::define-alias` runtime mechanism. Locked.

### D2 — Form shape: `(defalias :new-name :original-name)`

Two positional keyword args. Both names exist post-stone (alias is additive). No metadata-map this stone (defalias is simple; no per-binding metadata expected; future extension if surfaced).

### D3 — `:wat::runtime::define-alias` DIES (user direction)

User direction 2026-05-29 late: *"at the end of this work :wat::runtime::define-alias is dead - :wat::core::defalias is the only way to do name aliasing."*

The 26 callers migrate to `:wat::core::defalias`. The runtime form is HARD-CUT-rejected at check.rs (mirror Stone 241.8/9/11 arm shape). The retirement-table grows to 5 entries.

This satisfies `feedback_hard_cut_admits_no_bypasses` AT THE RUNTIME LAYER too — no substrate-internal alias mechanism survives separately from the user-facing form. ONE form; ONE mechanism; ONE name.

### D4 — Substrate cascade: migrate `:wat::core::define`-as-alias uses to `:wat::core::defalias`

The substrate-internal alias uses (which round 2 misclassified as "privileged path") migrate to defalias. The DOCTRINE: HARD CUT is total; no privileged paths.

### D5 — Retirement-table grows to 5 entries

Per S7 + D3: `:wat::runtime::define-alias` retires; retirement entry added. RETIREMENT_TABLE grows from 4 to 5 entries (struct, struct-restricted, enum, define, runtime define-alias). Note: `:wat::core::define-alias` user-surface form never existed (0 references); no separate entry for that name.

### D6 — Vigilia NOT required (D7 default)

Per `feedback_namespaced_home_vigilia_gate` D7 default: this stone does NOT mint a new namespaced home; substrate edits live in legacy flat substrate. SCORE-green commit. The lesson from Song #44 (vigilia must fire from orchestrator independently) does NOT apply here — vigilia is gate for namespaced homes, not for substrate flat edits.

### D7 — Per `feedback_hard_cut_admits_no_bypasses`: no privileged paths

The BRIEF EXPLICITLY pre-authorizes this doctrine. Sonnet MUST NOT classify any substrate `:wat::core::define` use as "privileged path" or "internal bypass." Either it's one of the 6 acceptable categories (S6) or it migrates.

## Trap-door audit

### T1 — Cascade size for substrate-internal alias uses unknown

Round 2 didn't ship the actual count. Sonnet's first action: audit. Predict 10-30 sites (the substrate is structured; alias uses cluster in stdlib + reflection paths).

### T2 — Reflection emitters may produce alias AST contextually

Emitters that produce `(:wat::core::define ...)` AST may not statically be aliases — they emit whatever shape the input form had. The migration may need a CONDITIONAL: if emitting an alias shape (body is single keyword), emit defalias; otherwise emit defn or def. Sonnet judges per-emitter.

### T3 — Define-alias in arc 143 docs

Arc 143's slice docs reference "define-alias" — these are historical artifacts (the arc was named "define-alias" per its directory). Comments + retired-arc docs stay UNTOUCHED (per `feedback_inscription_immutable`). They are documentation of a historical arc, not active substrate.

### T4 — Pre-INSCRIPTION grep gate is the discipline test

Stone 241.12's completion criterion includes the pre-INSCRIPTION grep returning 0 non-acceptable matches. This is the gate Stone 241.11.fix round 2 failed; Stone 241.12 closes the gap. Per `feedback_no_pre_existing_excuse`: don't deflect; investigate root cause and migrate.

### T5 — Sonnet may surface other trap-doors during the audit

Per `feedback_trap_door_build_the_dependency`: when the audit reveals a substrate gap (e.g., a category of substrate-internal define use that doesn't map cleanly to defalias OR defn OR def), BUILD the missing piece forward. Don't declare incoherent.

### T6 — Stone 241.11.fix's 14 test migrations stay

Round 1 of Stone 241.11.fix migrated 14 test sites + 1 doc update. Those stay (good work; user-facing test source migrations are correct). Round 2's work was killed before any changes landed.

## STOP triggers — REJECTION

1. Compile errors not traced to defalias mint or alias cascade migration
2. Lib < 890 (the Stone 241.11 ship baseline)
3. **120 min elapsed** (this stone is smaller than 241.10 / 241.11 because the cascade is bounded by the actual alias-use count)
4. holon-rs touched
5. Sonnet classifies a substrate `:wat::core::define` use as "privileged path" or "intentional bypass" without migrating — D7 + `feedback_hard_cut_admits_no_bypasses` violation
6. `:wat::core::define-alias` retirement-table entry added (D5 violation — that form never existed)
7. Files outside `src/runtime.rs`, `src/check.rs`, `src/freeze.rs`, `src/types.rs`, `wat/core.wat`, stdlib wat files, the alias-cascade target files, `tests/probe_arc241_stone12_*`, SCORE doc, `src/remedy/*` (NO modification expected)
8. Stone 241.12 probe < N/N PASS
9. Stone 241.1-241.11 + arc 237/238 probes regress
10. Clippy > 902
11. Auto-fixer crate (if minted) SURVIVES the commit (D2+T4 from Stone 241.11 precedent applied)
12. Pre-INSCRIPTION grep returns ANY non-acceptable matches post-stone (the discipline gate; Stone 241.13 INSCRIPTION cannot ship until clean)

## FM 2-bis evidence

`tests/probe_arc241_stone12_defalias.rs` (NEW). At HEAD: `:wat::core::defalias` doesn't exist → success-path contracts FAIL; substrate alias uses unmigrated → pre-INSCRIPTION grep returns non-acceptable matches.

Post-stone: all contracts PASS + pre-INSCRIPTION grep CLEAN.

## Calibration

**Target band: 60-120 min Mode A.**
**Upper bound: 150 min (STOP-3).**

Stone 241.12 is structurally LIGHTER than 241.10 (no schema upgrade; no namespaced home; no vigilia) and 241.11 (cascade is bounded by substrate alias uses, not 271 user-facing sites). The dominant runtime variable is the substrate-internal alias cascade audit + migration.

Per `feedback_stone_briefs_cite_prior_score`: BRIEF cites SCORE-STONE-241.11.md for cascade discipline + ephemeral auto-fixer pattern (if needed); SCORE-STONE-241.10.md for substrate-mint shape (defalias parsing is structurally similar to defstruct/defenum parsing — keyword + simple positional args).

## What this unblocks

**Stone 241.13** — INSCRIPTION closes arc 241. Pre-INSCRIPTION grep enforced. With defalias minted + alias cascade complete, the grep gate passes.

**Arc 237.8b** — reopens after Stone 241.13 per `feedback_no_regression_until_arc_done`.

**Future HARD CUTs** — the def*-prefix family is now COMPLETE (def / defn / defclause / defmacro / defstruct / defenum / defalias; defrecord queued arc 227; deftypealias queued arc 109). The pattern is foundational; any future form retirement consumes the bandaid-rip-with-receipts protocol shipped at 241.10.

The def*-prefix family was never complete without defalias. Stone 241.12 completes it. Then arc 241 closes.
