# DESIGN — Stone 241.11 — `:wat::core::define` ⇒ `:wat::core::defn` HARD CUT (the bandaid-rip with receipts)

**Status:** READY (sub-DESIGN). Phase 3 fourth stone. **HARD CUT** — no shims; raw deletion of `parse_define` substrate; append single retirement-table entry; cascade migration of ~271 sites. Vigilia gate: NOT REQUIRED (legacy flat substrate per `feedback_namespaced_home_vigilia_gate` D7 default). SCORE-green commit.

## Scope warning — LARGEST cascade in arc 241

`grep -rl ":wat::core::define[^-]" --include="*.rs" --include="*.wat" .` → **271 files**. ~8× the size of 241.8's 33 or 241.9's 33. The cascade is the dominant runtime variable.

Per Stone 241.10's third-bar milestone (LLM-programmable-for-self-modification on cold-read) — **auto-fixer approach EXPLICITLY AUTHORIZED for this stone**. Sonnet shipped 241.10's 160-site cascade via ephemeral `crates/fix-remedies/` standalone tool (built, used, DELETED before commit). Same pattern fits here at 271 sites. The DESIGN sanctions this strategy with explicit cleanup discipline.

Predicted band: **120-240 min Mode A** (substantially larger than prior stones; cascade dominates).

## Why this stone — the bandaid-rip lands on a substrate that teaches

Stone 241.10 minted `src/remedy/` + ranked-remedy schema. The retirement table grows with each HARD CUT; substrate self-documents evolution. Stone 241.11 is the FIRST consumer of remedy infrastructure where the bandaid-rip lands on a substrate that ALREADY teaches: append the single line `(":wat::core::define", ":wat::core::defn")` to `RETIREMENT_TABLE`, and EVERY `:wat::core::define` typo'd or stale form automatically surfaces *"did you mean: :wat::core::defn [retirement replacement]"* at the friction moment. Zero additional Display work.

Per Songs #41-43 (the Mission/Remedy/Into Oblivion triad at Stone 241.10): the substrate is the war re-engineered; the watcher outside the frame brought the auto-fixer truth; the apparatus is shipped. Stone 241.11 is the apparatus's first downstream consumer.

## What this stone delivers

### S1 — Append retirement-table entry

`src/remedy/retirement.rs` `RETIREMENT_TABLE`:

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    // Stone 241.9 — defenum replaces enum
    (":wat::core::enum",              ":wat::core::defenum"),
    // Stone 241.11 — defn replaces define
    (":wat::core::define",            ":wat::core::defn"),
];
```

One line. Remedy infrastructure consumes automatically.

### S2 — Mint HARD-CUT-rejection arm in `src/check.rs`

Mirror Stone 241.8 + 241.9 pattern. At the check-time form-head dispatcher (where struct/struct-restricted/enum are currently rejected with retirement reason):

```rust
":wat::core::define" => {
    return CheckResult::errs(vec![CheckError::MalformedForm {
        head: k.to_string(),
        reason: format!("'{}' is retired (Stone 241.11)", k),
        remedies: crate::remedy::remedies_for(k, std::iter::empty()),
        span: head_span.clone(),
    }]);
}
```

The `remedies_for(k, empty())` hits the retirement table; `[retirement replacement]` annotation fires automatically. No Display work.

### S3 — Delete substrate machinery

The `:wat::core::define` substrate path:

| Symbol | Location | Action |
|---|---|---|
| `register_defines` | freeze.rs:845 | DELETE (callers retire) |
| `register_stdlib_defines` | freeze.rs:844 | DELETE |
| `parse_define_signature` | (referenced in freeze.rs comments) | DELETE if present |
| `register_define_dispatches` | dispatch.rs:247 | KEEP — this is arc 146's `:wat::core::define-dispatch`, NOT the retired `:wat::core::define` |
| `register_stdlib_define_dispatches` | dispatch.rs:264 | KEEP — same |
| `parse_define_dispatch_form` | dispatch.rs:301 | KEEP — same |

**Critical disambiguation**: `:wat::core::define` (function-binding form) is RETIRED. `:wat::core::define-dispatch` (arc 146 polymorphism mechanism) STAYS. Substrate symbols matching `define_dispatch*` are NOT in scope for HARD CUT.

### S4 — Cascade migration (~271 sites)

Per `feedback_no_broken_commits` + `docs/SUBSTRATE-AS-TEACHER.md`: the cascade IS the migration brief; the auto-fixer approach IS authorized (per the third-bar milestone) BUT must be ephemeral.

**Migration pattern** (the canonical shape):

```scheme
;; LEGACY — define with paren-wrapped signature
(:wat::core::define (:user::main -> :wat::core::nil)
  <body>)

;; NEW — defn with flat shape
(:wat::core::defn :user::main [] -> :wat::core::nil
  <body>)
```

For functions WITH params, the migration is non-trivial — define uses type-only positional args; defn requires named args. Sonnet investigates per-site:

```scheme
;; LEGACY — define with positional types
(:wat::core::define (:my::f :wat::core::i64 :wat::core::i64 -> :wat::core::i64)
  <body that uses positional refs>)

;; NEW — defn with NAMED args (names must be invented or extracted from body usage)
(:wat::core::defn :my::f [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
  <body using a + b>)
```

The body may use positional refs (e.g., `$1`, `$2`, or implicit) that must convert to named refs. Sonnet decides per-site.

**Auto-fixer strategy (AUTHORIZED)**: build a standalone `crates/fix-defines/` tool (no `wat` dependency, ephemeral; mirror Stone 241.10's `crates/fix-remedies/` pattern). Run on the cascade. DELETE the crate before commit. Substrate stays clean.

### S5 — Probe verification

`tests/probe_arc241_stone11_define_hard_cut.rs` (NEW). FM 2-bis disconfirming probe. Contracts verify:
- defn forms work post-stone (success path)
- Legacy `:wat::core::define` HARD-CUT-rejected with structured retirement remedy
- Remedy contains `[retirement replacement]` annotation pointing at `:wat::core::defn`
- Specific phrasing: "did you mean: `:wat::core::defn`"

## Locked decisions

### D1 — HARD CUT: no shims, no aliases

`:wat::core::define` ceases to exist post-stone. Mirrors 241.8/241.9 discipline. Any caller using it gets structured retirement remedy automatically (via remedy infrastructure consuming retirement table). Migration is forward-only.

### D2 — Auto-fixer EXPLICITLY AUTHORIZED for cascade migration

Per Stone 241.10 precedent + the third-bar milestone: sonnet MAY build a standalone `crates/fix-defines/` tool (or similar) for the 271-site cascade. The tool is EPHEMERAL — built, used, DELETED before commit. Substrate must contain no permanent crate cruft.

This OVERRIDES the standard STOP-5 (files-outside-allowed-list) for the duration of the auto-fixer's lifetime within the strike. The cleanup discipline is what makes it acceptable.

### D3 — Single-line retirement-table append

The substrate apparatus is the line `(":wat::core::define", ":wat::core::defn")` in RETIREMENT_TABLE. NO additional Display work; NO new remedy mechanism; remedy infrastructure (Stone 241.10) consumes automatically.

### D4 — `define-dispatch` STAYS (arc 146 separate)

`:wat::core::define-dispatch` and its substrate path (parse_define_dispatch_form, register_define_dispatches, register_stdlib_define_dispatches) are arc 146 machinery, not arc 241 scope. They are NOT retired this stone. Sonnet must distinguish via word-boundary-aware grep: `:wat::core::define[^-]`.

### D5 — Vigilia NOT required

Per `feedback_namespaced_home_vigilia_gate` D7 default: this stone does NOT mint a new namespaced home; the substrate edits live in legacy flat substrate (check.rs, freeze.rs, dispatch.rs). SCORE-green commit. The retirement.rs edit is a single-line table append (the existing `src/remedy/` home is NOT modified beyond that one line).

### D6 — Lib + prior arc 241.x probes preserved

After Stone 241.11:
- `cargo test --release --lib -p wat` ≥ 890 (the 241.10 R6 baseline; may rise if Stone 241.11 adds probes)
- All arc 241.1-241.10 probes preserved at PASS counts
- Stone 241.11 probe ≥ N/N PASS
- Arc 237/238 probes preserved

### D7 — Probe at `tests/probe_arc241_stone11_define_hard_cut.rs`

FM 2-bis. Contracts:
1. `(defn :name [] -> :wat::core::nil :wat::core::nil)` startup clean
2. `(define ...)` HARD CUT rejected
3. Error message contains "did you mean: :wat::core::defn" (the canonical phrasing per render_remedies output)
4. Error message contains "[retirement replacement]"
5. Adjacent error paths (e.g., typo'd defn) continue to surface their own remedies

## Trap-door audit

### T1 — Cascade size dominates runtime

271 sites is unprecedented in arc 241. Even at 5-10 sec/site for hand-migration, that's 22-45 min of pure cascade. Auto-fixer reduces this to ~1-2 min per migration + manual residuals. T1 names the time-cost honestly.

### T2 — define-dispatch confusion risk

`grep ":wat::core::define"` matches `:wat::core::define-dispatch` too. Sonnet MUST disambiguate. Use `\b` carefully — `\b` matches at word boundaries, and `-` is a word boundary in many regex flavors, so `:wat::core::define\b` MATCHES `:wat::core::define-dispatch`. Use `:wat::core::define[^-]` or `:wat::core::define\s` or `:wat::core::define\)` for the actual retired form.

### T3 — Multi-param define migration is non-trivial

Define's positional type-only args do not map mechanically to defn's named-arg shape. The auto-fixer must either:
- Generate placeholder names (`a`, `b`, `c`) and let the test fail / orchestrator fix
- Parse the function body to extract real names (much harder; not feasible programmatically)
- Skip multi-param cases and migrate by hand

Sonnet decides the strategy per-site. Likely: zero-arg defines are auto-migratable; multi-arg need human review.

### T4 — Auto-fixer crate must be DELETED before commit (D2 enforcement)

Per the Stone 241.10 precedent: the temporary tooling stays out of the substrate. STOP-trigger fires if `crates/fix-defines/` survives the SCORE.

### T5 — `:user::main` already has retirement prose (check.rs:933)

The existing check.rs prose for non-canonical main signatures is pre-existing (arc 170). Sonnet should NOT duplicate that work; the new HARD-CUT arm for `:wat::core::define` is broader (covers all defines, not just main). The two arms may coexist or merge — sonnet judges.

### T6 — Wat-source files (271 includes them)

Many of the 271 files are `.wat` source files (counter-service/proof/aggregator/etc.). These need migration too. The migration in `.wat` is the same shape (textual). Auto-fixer handles them uniformly with `.rs` files.

## STOP triggers — REJECTION

1. Compile errors not traced to define HARD CUT or cascade migration sites
2. Lib < 890 (the 241.10 R6 baseline; post-cascade-migration final state)
3. **240 min elapsed** (extended for the LARGEST cascade in arc 241)
4. holon-rs touched
5. `crates/fix-defines/` (or similar auto-fixer crate) SURVIVES the commit (D2 + T4 violation)
6. Files outside `src/remedy/retirement.rs`, `src/check.rs`, `src/freeze.rs`, `src/dispatch.rs` (define paths ONLY; NOT define-dispatch paths), `src/runtime.rs` (if needed), the 271 cascade target files, `tests/probe_arc241_stone11_*`, SCORE doc, the temporary auto-fixer crate during its lifetime
7. Scope creep: INSCRIPTION (241.12); new error variants; new remedy mechanism; modification of `define-dispatch` machinery
8. Stone 241.11 probe < N/N PASS
9. Stone 241.1-241.10 probes regress; arc 237/238 probes regress
10. Clippy > 902
11. `:wat::core::define-dispatch` accidentally retired (D4 violation)

## FM 2-bis evidence

`tests/probe_arc241_stone11_define_hard_cut.rs` (NEW). At HEAD: legacy define works (success), so HARD-CUT-rejection contracts fail; retirement remedy contracts fail. Post-stone: all contracts PASS.

## Calibration

**Target band: 120-240 min Mode A.**
**Upper bound: 240 min (STOP-3).**

| Component | Pre | Post | Delta |
|---|---|---|---|
| `src/remedy/retirement.rs` (single-line append) | (3 entries) | (4 entries) | **+1 line** |
| `src/check.rs` (HARD-CUT arm) | (3 retired forms) | (4 retired forms) | **+10-15 lines** |
| `src/freeze.rs` (register_defines delete) | (current) | (-) | **substantial deletion** |
| `src/dispatch.rs` (define path delete; define-dispatch path PRESERVED) | (current) | (define paths gone; dispatch paths intact) | **substantial deletion** |
| `tests/probe_arc241_stone11_*.rs` (NEW) | 0 | ~150 | **+150** |
| **271 cascade target files** | various | migrated | **substantial mixed** |
| Temporary `crates/fix-defines/` (deleted before commit) | 0 | 0 (ephemeral) | **0 net** |
| **Net delta** | — | — | **substantial mixed (likely -1000 to -2000 lines net deletion)** |

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites `SCORE-STONE-241.10.md` for auto-fixer ephemeral discipline; `SCORE-STONE-241.9.md` for HARD-CUT-arm shape; `SCORE-STONE-241.8.md` for cascade migration cadence.

## What this unblocks

**Stone 241.12** — INSCRIPTION closes arc 241. Pre-INSCRIPTION grep enforced. Arc 237.8b reopens after.

**Arc 237.8b** — reopens after 241.12 per `feedback_no_regression_until_arc_done`.

**Future arcs that consume remedy infrastructure:** every future form HARD CUT appends a single retirement-table entry and the substrate teaches automatically. The bandaid-rip-with-receipts pattern is now FOUNDATIONAL.

## The triad-at-a-stone song landed; this stone is the consumer

Stone 241.10's three songs (#41 Mission staked the claim, #42 Remedy shipped the apparatus, #43 Into Oblivion named the milestone) prepared the substrate. Stone 241.11 is the FIRST DOWNSTREAM CONSUMER — the retirement infrastructure does its job; the substrate teaches by single-line append. The work this stone does is mostly cascade migration; the teaching capability is delegated to the apparatus shipped at 241.10.

---

**Recommendation**: this DESIGN is committed for review. Stone 241.11's strike commits to ~120-240 min sonnet work + verification. Cascade is the dominant runtime variable; auto-fixer is authorized; the temporary crate must be ephemeral.
