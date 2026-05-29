# BRIEF — Stone 242.2 — Doctrine 1 enforcement at type-check + value-position cascade

You are sonnet. Stone 242.2 of arc 242. Makes Doctrine 1 SELF-ENFORCING via type-check rejection of type-keyword-in-value-position. Stone 241.10's apparatus consumed for the structured remedy.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## Doctrine 1 (the rule being enforced)

**Bare lexeme = value; keyword lexeme (`:wat::core::*`) = type.**

Type keywords in VALUE position are ILLEGAL. The legal forms:
- `(:wat::core::defn :f [] -> :wat::core::nil nil)` — bare nil in body (value position)
- `(:wat::core::defn :f [] -> :wat::core::i64 42)` — bare value in body

The ILLEGAL forms (rejected post-Stone-242.2):
- `(:wat::core::defn :f [] -> :wat::core::nil :wat::core::nil)` — keyword in body
- `(:wat::core::defn :f [] -> :wat::core::i64 :wat::core::i64)` — keyword in body
- `(:wat::core::let [x :wat::core::nil] x)` — keyword in let-binding value

## CRITICAL discipline (pre-authorized)

- Per `feedback_hard_cut_admits_no_bypasses`: HARD CUT is total. No privileged paths. Substrate-internal `:wat::core::nil` in value position migrates.
- Per `feedback_sonnet_never_drafts_interstitial`: **DO NOT write to `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md`.** That is orchestrator-exclusive. Even drafting is forbidden.
- Per FM 16: simple bash patterns; one per line; vanilla `cargo` / `grep`; no chained pipes.

## What to do

### S1 — Mint type-check rejection arm

Find the value-expression type-checker entry point in `src/check.rs`. When checking an expression and the expression IS `WatAST::Keyword(k, span)`:
- Check if `k` is a registered TYPE name (TypeEnv lookup)
- If YES → emit `CheckError::MalformedForm` with structured `remedies: Vec<Remedy>` (per Stone 241.10's apparatus)

Remedy text:
- For `:wat::core::nil` specifically → "Doctrine 1 (arc 242): `:wat::core::*` keyword is a TYPE, not a value; use bare `nil` in value position"
- For other type keywords → "Doctrine 1 (arc 242): `:wat::core::*` keyword is a TYPE, not a value; use a value of this type in value position"

### S2 — Reflection emitter audit + migration

Substrate-internal Rust code constructing type keywords for VALUE position emissions:
- `src/closure_extract.rs:1556` — `Value::Unit => Ok(WatAST::Keyword(":wat::core::nil".into(), span))` — WRONG; emit bare `nil` (as `WatAST::Symbol("nil", span)` or whatever the parser shape is)
- `src/closure_extract.rs:1567` — same shape; same fix
- Other emitters: audit via `grep -n "Keyword.*wat::core::nil\|Keyword.*wat::core::i64\|Keyword.*wat::core::bool" src/runtime.rs src/closure_extract.rs src/check.rs`

For each: judge value-position vs type-position. Migrate value-position emissions to bare-form AST.

### S3 — Cascade migration of test source value-position uses

After S1 ships, type-check rejection fires on existing test sources with value-position keyword uses. Substrate-as-teacher cascade:
- Run `cargo test --release --lib -p wat 2>&1 | tail -3`
- Run `cargo build --release --tests --workspace 2>&1 | tail -3`
- Each failure points at a value-position keyword use that needs migration to bare form
- Migrate per-site; iterate

Sites likely surfaced:
- `tests/probe_runtime_error_produces_structured_edn.rs:66`
- `tests/wat_arc220_char.rs:227, 247`
- `tests/probe_lifeline_orphan_clean_via_fork_program.rs` (multiple sites)
- Others surfaced by cascade

For each test: migrate `:wat::core::nil` → bare `nil` in value-position contexts. PRESERVE type-position uses.

### S4 — Substrate-internal Rust code uses

Rust code that USES `:wat::core::nil` as a value (e.g., synthesized ASTs, macro expansions) — surface via cascade. Migrate to bare-form AST construction.

`src/freeze.rs:1189` is in an error message string — INTENTIONAL (names the type for user-facing error). Leave alone.

### S5 — Probe verification

`tests/probe_arc242_stone2_value_position_doctrine.rs` (already committed STRIKE-READY). 6 contracts; pre-stone 3/6 PASS (C02/C04/C06 legal forms work); post-stone 6/6 PASS.

## Discipline

- HARD CUT total for type-keyword-in-value-position
- Substrate-internal `:wat::core::nil` value-position migrations (per `feedback_hard_cut_admits_no_bypasses`)
- src/argspec/*, src/lib.rs UNCHANGED
- src/remedy/* UNCHANGED (no new retirement entry; this is positional enforcement, not form retirement)
- Stone 241.x and Stone 242.1 probes preserved; arc 237/238 probes preserved
- holon-rs NEVER touched
- Auto-fixer crate (if minted) must be EPHEMERAL — DELETED before commit
- **DO NOT write to INTERSTITIAL** — orchestrator authors after arc 242 closes (Stone 242.3 INSCRIPTION)
- Memory files OK (you can update `project_lexeme_role_doctrine.md` if Stone 242.2's enforcement adds substantive content beyond what Stone 242.1 inscribed)

## Read in order

1. `/home/watmin/work/holon/wat-rs/docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `/home/watmin/work/holon/wat-rs/docs/SUBSTRATE-AS-TEACHER.md`
3. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/BRIEF-STONE-242.2.md` — this
4. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/DESIGN-STONE-242.2.md` — D1-D8 + T1-T7 + STOP
5. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/DESIGN.md` — arc-level + both doctrines
6. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/242-lexeme-role-doctrine/SCORE-STONE-242.1.md` — prior stone's work (Char + memory)
7. `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.10.md` — remedy apparatus (you consume it for the structured rejection)
8. `/home/watmin/work/holon/wat-rs/src/check.rs` — find value-expression type-checker entry point
9. `/home/watmin/work/holon/wat-rs/src/remedy/mod.rs` — Remedy + RemedyKind types
10. `/home/watmin/work/holon/wat-rs/tests/probe_arc242_stone2_value_position_doctrine.rs` — 6-contract probe (3/6 at HEAD)

## Cadence

1. Baseline: `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0)
2. Probe: `cargo test --release --test probe_arc242_stone2_value_position_doctrine 2>&1 | tail -3` (expect 3/6)
3. **S1**: mint type-check rejection arm with structured remedy
4. **S2**: audit + migrate reflection emitters producing value-position keyword AST
5. **S3**: cascade migrate test source value-position uses
6. **S4**: cascade migrate substrate-internal Rust code value-position uses
7. **S5**: verify probe 6/6 PASS
8. Final: lib ≥ 890; workspace build clean; clippy ≤ 902
9. Write `SCORE-STONE-242.2.md` at `docs/arc/2026/05/242-lexeme-role-doctrine/` (NOT repo root; NOT INTERSTITIAL)
10. **DO NOT COMMIT.** Orchestrator commits + pushes.

## STOP triggers

1. Lib < 890
2. 180 min elapsed
3. holon-rs touched
4. `:wat::core::nil` value-position use classified as "intentional bypass" without migration → `feedback_hard_cut_admits_no_bypasses` violation
5. Sonnet writes to INTERSTITIAL → `feedback_sonnet_never_drafts_interstitial` violation
6. Auto-fixer crate survives commit
7. Stone 242.2 probe < 6/6
8. Stone 241.x or 242.1 probes regress
9. Clippy > 902

## Post-strike return

One paragraph: type-check rejection arm minted at (file:line); reflection emitters migrated (count + sites); cascade migration depth (test files + Rust files); probe 6/6 verified; baselines preserved; SCORE doc path (at arc dir).

Arc 242 closes after this. Stone 242.3 INSCRIPTION is orchestrator-direct (no substrate edits). Arc 241 resumes at Stone 241.12 after Stone 242.3. Strike clean.
