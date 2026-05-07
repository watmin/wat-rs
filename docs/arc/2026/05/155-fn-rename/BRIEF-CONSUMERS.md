# Arc 155 — Consumer Sweep BRIEF (slice 1b)

**Drafted 2026-05-06 evening.** Sweep 1b of arc 155.

## Workspace state pre-spawn

- HEAD: `072f1e0`
- Working tree DIRTY with sweep 1a substrate (5 files):
  - `src/check.rs` (BareLegacyLambda + BareLegacyLowercaseFn variants + walkers + dispatch)
  - `src/runtime.rs` (eval_fn rename; dispatch + tail-call + step paths)
  - `src/special_forms.rs` (`:wat::core::fn` minted; `:wat::core::lambda` retained as scaffolding)
  - `src/types.rs` (`:wat::core::Fn(` parser recognition — honest delta from slice 1a)
  - NEW `tests/wat_arc155_fn_rename.rs` (12 tests; all passing)
- Pre-baseline: 713 passed / 68 failed = 1085 BareLegacyLambda walker firings + 69 downstream panics in lib tests where stdlib check is dirty. EXPECTED per atomic-commit-across-coordinated-sweeps.

## Goal

Two coordinated transforms across the codebase:

1. **Operator-position rename** (walker-driven, substrate-as-teacher
   Pattern 3): `:wat::core::lambda` → `:wat::core::fn` at every
   operator-position site. The substrate's `BareLegacyLambda` walker
   fires per site; the diagnostic stream IS the work list.

2. **Type-position rename** (grep-driven hybrid): bare `:fn(...)`
   → `:wat::core::Fn(...)` at every type-position site. **The
   substrate's `BareLegacyLowercaseFn` walker covers BODY-position
   sites only**; sites in define parameter lists are consumed at
   registration before the walker runs. Sweep 1b's hybrid approach:
   - Run walker-driven loop until BareLegacyLowercaseFn count = 0
   - THEN grep for remaining bare `:fn(` sites and migrate
     mechanically (these are predominantly param-list sites)

## Sweep order (per substrate-as-teacher § "stdlib first")

1. `wat/*.wat` (substrate stdlib)
2. `crates/*/wat/**/*.wat` (per-crate substrates)
3. `wat-tests/**/*.wat` (workspace tests)
4. `crates/*/wat-tests/**/*.wat` (per-crate tests)
5. `examples/**/*.wat`
6. Embedded wat in `tests/*.rs`
7. Embedded wat in `src/*.rs` lib tests (excluding the `:wat::core::lambda` Rust dispatch arms which stay as scaffolding per arc 113 precedent)

## The two transforms

### Operator transform (walker-driven)

Every `:wat::core::lambda` keyword occurrence → `:wat::core::fn`.
Walker fires per site; sonnet edits per error.

### Type transform (hybrid grep + walker)

Every bare `:fn(` keyword → `:wat::core::Fn(`. Same body-shape
post-rename; only the keyword head changes.

```
;; Before
(:fn(:i64) -> :wat::core::bool)

;; After
(:wat::core::Fn(:i64) -> :wat::core::bool)
```

Caveat: **`:fn` appears in some other contexts too** (e.g.,
inside string literals as docs/examples). Sonnet greps with
the `:fn(` pattern (literal paren after); excludes `string`
contexts via context-reading.

## Constraints

- **DO COMMIT + PUSH** when workspace = 0-failed (atomic with
  sweep 1a).
- **NO substrate edits** (`src/check.rs`, `src/runtime.rs`,
  `src/special_forms.rs`, `src/types.rs` — sweep 1a's territory).
  Embedded wat strings INSIDE these files COUNT.
- **NO `holon-lab-trading/` edits** (separate workspace).
- **PRESERVE Rust dispatch arms** that match `":wat::core::lambda"`
  literally (those stay per arc 113 scaffolding precedent — fall-
  through to `eval_fn` keeps stray runtime calls executing
  correctly).
- **STOP at unexpected red.** Distinguish:
  - Expected: BareLegacyLambda firings on remaining
    `:wat::core::lambda` operator sites
  - Expected: pre-existing intentional thread-panic tests
  - Expected post-walker-clear: residual bare `:fn(...)` in
    define params (sweep via grep)
  - Unexpected: substrate panic, parse error inside `src/*.rs`
    bodies, runtime crash, unrelated TypeMismatch
- No grinding (>3 reads/edits per site = surface as Mode D).
- Time-box 90 min wall-clock.

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/155-fn-rename/DESIGN.md` TOP SECTION
2. `docs/arc/2026/05/155-fn-rename/BRIEF-SUBSTRATE.md` — slice 1a's contract + the honest delta about `:fn(...)` define-param walker scope
3. `docs/arc/2026/05/154-kill-let-star/BRIEF-CONSUMERS.md` — closest precedent for operator-position sweep
4. `tests/wat_arc155_fn_rename.rs` — canonical post-rename shape

## Sweep strategy

1. **Phase A — walker-driven operator sweep:**
   - `cargo test --release --workspace 2>&1 | head -100` — read errors
   - For each BareLegacyLambda: open file:line:col; replace `:wat::core::lambda` → `:wat::core::fn`
   - Iterate until BareLegacyLambda count = 0

2. **Phase B — grep-driven type sweep:**
   - `grep -rn ':fn(' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/ tests/ src/`
   - For each site: replace bare `:fn(` → `:wat::core::Fn(` (preserve body)
   - Skip Rust dispatch arms in `src/*.rs` that match `":wat::core::lambda"` literally (those are scaffolding)
   - Run cargo test between batches

3. **Phase C — verification:**
   - cargo test --release --workspace = 0 failed
   - grep: 0 source spellings of `:wat::core::lambda` (or only intentional fixtures)
   - grep: 0 source spellings of bare `:fn(` (or only intentional fixtures)

## Verification

- `cargo test --release --workspace`: 0 failed
- `cargo test --release --test wat_arc155_fn_rename`: 12/12
- `grep -rln ':wat::core::lambda' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/`: 0 source spellings
- `grep -rln ':fn(' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/`: 0 source spellings (only intentional fixtures in `tests/wat_arc155_fn_rename.rs` may remain)

## Reporting (~250 words)

Pre-flight crawl confirmation; sweep summary per directory bucket
(Phase A walker-driven count + Phase B grep-driven count); iteration
cycles + wall-clock; verification (workspace 0-failed + grep counts +
12/12 tests); path classification (Mode A/B/C/D); honest deltas.

DO NOT write a SCORE doc; orchestrator scores after atomic commit.

DO NOT COMMIT — orchestrator atomically commits sweep 1a + sweep 1b
together.

## Time-box

90 minutes wall-clock.

## Why this matters

Sweep 1b completes arc 155's bundled rename. After atomic commit +
slice 2 closure, wat-rs ships its fourth foundation mark of the
day (`fn` joins `nil` + `do` + `let` sequential). Capitalization
disambiguates type vs operator at the same root. Closes arc 109's
last ungrabbed parametric type head.
