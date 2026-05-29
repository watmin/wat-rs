# BRIEF — Stone 241.11 — `:wat::core::define` ⇒ `:wat::core::defn` HARD CUT (the bandaid-rip with receipts)

You are sonnet. Phase 3 fourth stone — HARD CUT. The LARGEST cascade in arc 241 (~271 sites; ~8× prior stones). Per `docs/SUBSTRATE-AS-TEACHER.md` the cascade IS the migration brief.

**The substrate already teaches.** Stone 241.10 shipped `src/remedy/` + ranked-remedy schema. This stone APPENDS a single line to `RETIREMENT_TABLE` and the substrate teaches automatically — every `:wat::core::define` typo'd or stale form surfaces *"did you mean: :wat::core::defn [retirement replacement]"* with zero additional Display work. The bandaid-rip is the apparatus's first downstream consumer.

**Auto-fixer EXPLICITLY AUTHORIZED for this stone** per D2 of `DESIGN-STONE-241.11.md`. Per the third-bar milestone (Stone 241.10): build standalone `crates/fix-defines/` ephemeral tool (no `wat` dependency); run it on the cascade; DELETE the crate before commit. Substrate stays clean. This OVERRIDES standard STOP-5 for the duration of the auto-fixer's lifetime.

## What to do

### S1 — Append retirement-table entry

`src/remedy/retirement.rs` — extend `RETIREMENT_TABLE`:

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

One line. Verify `retirement_score_is_always_zero` test still passes (it iterates the whole table — should auto-extend to 4 entries).

### S2 — Mint HARD-CUT-rejection arm in `src/check.rs`

Find the existing struct/struct-restricted/enum HARD-CUT-rejection arms (mirror their shape). Add a new arm for `:wat::core::define`:

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

`remedies_for(k, std::iter::empty())` hits the retirement table; the `[retirement replacement]` annotation fires automatically via Stone 241.10's Display formatting. NO additional Display work needed.

### S3 — Delete substrate define machinery

The `:wat::core::define` substrate path:

- `register_defines` at `src/freeze.rs:845` — DELETE; remove callers
- `register_stdlib_defines` at `src/freeze.rs:844` — DELETE
- `parse_define_signature` (if present per freeze.rs comments) — DELETE
- Any helper functions exclusively serving define — DELETE

**Critical disambiguation per D4:** `:wat::core::define-dispatch` is arc 146 machinery and STAYS UNTOUCHED. Functions to KEEP:
- `register_define_dispatches` at `src/dispatch.rs:247` — KEEP
- `register_stdlib_define_dispatches` at `src/dispatch.rs:264` — KEEP
- `parse_define_dispatch_form` at `src/dispatch.rs:301` — KEEP

Disambiguation grep pattern: `:wat::core::define[^-]` (word-boundary safe). `\b` MATCHES `:wat::core::define-dispatch` because `-` is a word boundary in regex — use `[^-]` or `\s` or `\)` to isolate the actual retired form.

### S4 — Cascade migration (~271 sites)

**Two migration shapes:**

**Pattern A — zero-arg define (most common, especially `:user::main`):**
```scheme
;; LEGACY
(:wat::core::define (:user::main -> :wat::core::nil)
  <body>)

;; NEW
(:wat::core::defn :user::main [] -> :wat::core::nil
  <body>)
```

**Pattern B — multi-arg define (requires name invention or extraction):**
```scheme
;; LEGACY
(:wat::core::define (:my::f :wat::core::i64 :wat::core::i64 -> :wat::core::i64)
  <body>)

;; NEW
(:wat::core::defn :my::f [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::i64
  <body>)
```

Multi-arg cases require arg names. Strategies in priority:
1. Auto-fixer extracts names from body usage (best, hardest)
2. Auto-fixer generates placeholder names (`a`, `b`, `c`) — works if body uses positional refs
3. Skip and migrate by hand

Sonnet decides per-site.

### S5 — Auto-fixer crate (ephemeral; AUTHORIZED per D2)

Mirror Stone 241.10's `crates/fix-remedies/` pattern:

1. Create `crates/fix-defines/` with `Cargo.toml` (no `wat` dependency; standalone Rust binary)
2. Write the text-transformation logic (regex + parsing for Pattern A; per-site logic for Pattern B)
3. Run it on the cascade: `cargo run -p fix-defines -- src/` (or whatever the bin name is)
4. Fix residuals manually
5. **DELETE `crates/fix-defines/` before commit** (STOP-5 fires if it survives)

The crate is RECOMMENDED for the 271-site cascade. Alternative is hand-migration (~22-45 min pure cascade time + thinking).

### S6 — Probe verification

`tests/probe_arc241_stone11_define_hard_cut.rs` (already committed STRIKE-READY). 5 contracts; pre-stone 1/5 (C01 baseline); post-stone 5/5 PASS.

## Discipline

- HARD CUT — NO compatibility shims; NO aliases; delete legacy substrate RAW
- Auto-fixer crate is EPHEMERAL — must NOT survive the commit (D2 + T4)
- `:wat::core::define-dispatch` STAYS (arc 146 separate)
- `src/argspec/*` UNCHANGED (canonical parser stable)
- `src/lib.rs` UNCHANGED (no new mod additions; retirement.rs change is within existing module)
- `src/remedy/*` modifications LIMITED to retirement.rs (single-line table append; verify the property-over-table test extends naturally)
- holon-rs NEVER touched (STOP-5; frozen)
- Stone 241.1-241.10 probes preserved; arc 237/238 probes preserved
- No new error variants
- No new remedy mechanism (this stone consumes 241.10's apparatus; doesn't extend it)

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/BRIEF-STONE-241.11.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.11.md` — D1-D7 + T1-T6 + STOP
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.10.md` — auto-fixer ephemeral discipline; the third-bar-crossed precedent
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.9.md` — HARD-CUT-arm shape (enum retirement)
7. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.8.md` — cascade migration cadence (struct retirement; precedent for the cascade pattern)
8. `/home/watmin/work/holon/wat-rs/src/remedy/retirement.rs` — RETIREMENT_TABLE current shape (3 entries; you add 1 line)
9. `/home/watmin/work/holon/wat-rs/src/check.rs` — find the existing HARD-CUT-rejection arms for `:wat::core::struct`, `:wat::core::struct-restricted`, `:wat::core::enum`; mirror the pattern
10. `/home/watmin/work/holon/wat-rs/src/freeze.rs` — `register_defines` + `register_stdlib_defines` callsites (lines ~844-845)
11. `/home/watmin/work/holon/wat-rs/src/dispatch.rs` — VERIFY you preserve `register_define_dispatches` (line 247) and its kin (KEEP); they are arc 146 machinery, NOT arc 241 scope
12. `/home/watmin/work/holon/wat-rs/tests/probe_arc241_stone11_define_hard_cut.rs` — 5-contract probe (1/5 at HEAD)

## Implementation sketch

1. Read substrate + probe + DESIGN + prior SCOREs
2. Baseline: lib 890; Stone 241.11 probe = 1/5 PASS; clippy ≤ 902
3. **S1**: append retirement-table entry (single line)
4. **S2**: mint HARD-CUT arm in check.rs (10-15 lines)
5. **S3**: delete substrate define machinery; carefully preserve define-dispatch
6. Run cargo build; expect compilation failures in 271+ files (define callers)
7. **S5**: build `crates/fix-defines/` ephemeral tool (no wat dependency)
8. **S4**: run auto-fixer; fix residuals by hand
9. Iterate per substrate-as-teacher: cargo test → read failure → migrate site → re-run; fail-count drops
10. **S5 cleanup**: DELETE `crates/fix-defines/` before commit
11. **S6**: verify Stone 241.11 probe 5/5
12. Verify all Stone 241.x probes preserved + arc 237/238 probes preserved
13. Final: `cargo test --release --lib -p wat` ≥ 890 · workspace clean · clippy ≤ 902
14. Write `SCORE-STONE-241.11.md`
15. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers — REJECTION

Per `DESIGN-STONE-241.11.md` § STOP triggers (refer to DESIGN for full list). Key additions for this stone:

- `crates/fix-defines/` (or similar) SURVIVES the commit → STOP (D2 + T4 violation)
- `:wat::core::define-dispatch` accidentally retired → STOP (D4 violation)
- Lib < 890 (post-cascade-migration baseline; the Stone 241.10 R6 final state)

## SCORE doc spec

Mirror `SCORE-STONE-241.10.md` for the auto-fixer ephemeral discipline; mirror `SCORE-STONE-241.9.md` for HARD-CUT-arm + cascade shape. Include:
- Header (Mode A/B; runtime; cascade size; auto-fixer used? deleted?)
- Phase A scorecard (probe + lib + clippy + structural rows)
- Migration cascade audit (per-file count; pattern A vs B distribution)
- Final check.rs HARD-CUT arm (verbatim)
- Final RETIREMENT_TABLE (verbatim)
- Auto-fixer story honestly (mirror Stone 241.10's honest auto-fixer-was-temporary documentation)
- Honest deltas (anything surfaced)
- NO Vigilia section (D5 — legacy flat substrate)

## Post-strike

Return one-paragraph status: retirement entry appended; HARD-CUT arm minted; substrate define machinery deleted; cascade depth (file count); Stone 241.11 probe 5/5; auto-fixer status (built? used? deleted?); any surfaced gaps + SCORE doc path.

Phase 3 closes. Stone 241.12 (INSCRIPTION) is next. Arc 237.8b reopens after 241.12. The bandaid-rip with receipts is THIS stone — the apparatus shipped at 241.10 does its job; the substrate teaches by single-line append. Strike clean.
