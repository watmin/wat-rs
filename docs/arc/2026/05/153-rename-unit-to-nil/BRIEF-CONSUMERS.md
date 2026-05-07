# Arc 153 — Consumer Sweep BRIEF (slice 1b)

**Drafted 2026-05-06.** Sweep 1b of arc 153.

User direction:
> *"let's roll"*

## Workspace state pre-spawn

- HEAD: `dbe72e1` (BRIEF + EXPECTATIONS for sweep 1a; sweep 1a substrate edits in working tree, uncommitted)
- Working tree DIRTY: 3 modified (`src/check.rs`, `src/runtime.rs`, `src/types.rs`) + 1 new (`tests/wat_arc153_nil_rename.rs`) — these are sweep 1a's output, intentionally uncommitted per recovery doc § 7 atomic-commit-across-coordinated-sweeps
- Pre-baseline: `cargo test --release --workspace` shows ~35 `BareLegacyUnitName` errors in stdlib + ~69 downstream panics in lib tests (`check::tests::*`, `runtime::tests::*`) where `assert!(check(...).is_ok())` fails because stdlib type-check is not clean

## Goal

Two coordinated sweeps to converge workspace to 0 failed:

1. **Type-position sweep** (substrate-as-teacher Pattern 3 driven):
   replace `:wat::core::unit` → `:wat::core::nil` at every annotation
   site. The substrate's `BareLegacyUnitName` walker fires per site;
   sonnet edits per site; iterate cargo test until clean.

2. **Value-position sweep** (mechanical grep-driven): replace `()`
   value-position literals with `:wat::core::nil`. NOT walker-driven
   (substrate accepts `()` transitionally); sonnet greps for sites,
   classifies each (value-position vs type-position parens), edits.

After sweep 1b, workspace returns to 0-failed; atomic commit with
sweep 1a happens.

## Sweep order (substrate-as-teacher § "stdlib first")

The substrate's bundled stdlib loads on every wat invocation. If it has unmigrated `:wat::core::unit` sites, every test fires those errors before producing useful output. Sweep order:

1. **`wat/*.wat`** (substrate stdlib) — FIRST
2. **`crates/*/wat/**/*.wat`** (per-crate substrates)
3. **`wat-tests/**/*.wat`** (workspace test wat)
4. **`crates/*/wat-tests/**/*.wat`** (per-crate test wat)
5. **`examples/**/*.wat`** (example programs)
6. **Embedded wat strings in `tests/*.rs`** (Rust integration tests)
7. **Embedded wat strings in `src/*.rs`** (lib tests — the 69 panicking tests)

After step 1, run `cargo test` to confirm stdlib boots clean (BareLegacyUnitName count drops; downstream panics start clearing).

## The two transforms (per site)

### Type-position transform (walker-driven)

Every `:wat::core::unit` at type position → `:wat::core::nil`:

```scheme
;; Before
(:wat::core::define
  (:probe -> :wat::core::unit)
  body)

;; After
(:wat::core::define
  (:probe -> :wat::core::nil)
  body)
```

Same for parametric containment:
```scheme
;; Before
:wat::core::Vector<wat::core::unit>

;; After
:wat::core::Vector<wat::core::nil>
```

The walker fires per-site; the diagnostic names file:line:col + the canonical FQDN. Sonnet edits per error.

### Value-position transform (grep-driven)

Every `()` at value position → `:wat::core::nil`:

```scheme
;; Before — function returning unit value
(:wat::core::define
  (:probe -> :wat::core::nil)
  ())

;; After
(:wat::core::define
  (:probe -> :wat::core::nil)
  :wat::core::nil)
```

**Critical disambiguation:** `()` appears in MANY contexts:
- Value position (e.g., function body returning unit) — TRANSFORM
- Type position parens (e.g., `(:probe -> :wat::core::nil)` — the outer parens) — DO NOT TRANSFORM
- Empty list / vector literal where context expects a Vector — leave alone (those weren't unit anyway)

Sonnet must read each `()` site in context to classify before transforming. When in doubt, run cargo test after edit; if a real list-literal site got accidentally swapped to `:wat::core::nil`, TypeMismatch will fire and sonnet narrows.

## Constraints

- **DO NOT COMMIT.** Working tree stays modified for atomic commit with sweep 1a per recovery doc § 7.
- **NO substrate edits** (`src/*.rs` body). Embedded wat strings INSIDE `src/*.rs` lib tests COUNT (those need migration); but Rust code logic doesn't change.
- **NO `holon-lab-trading/` edits** (separate workspace).
- **STOP at unexpected red.** Distinguish:
  - Expected red: `BareLegacyUnitName` migration error on remaining unmigrated sites (drives your work)
  - Expected red: TypeMismatch from a value-position `()` accidentally swapped where context expects a Vector (drives narrowing)
  - **Unexpected red:** substrate panic, parse error inside check.rs/runtime.rs, runtime crash, TypeMismatch unrelated to nil/unit. Surface as Mode B/C.
- No grinding (>3 iterations on a single site = surface as Mode D).
- Time-box 120 min wall-clock.

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/153-rename-unit-to-nil/DESIGN.md` — read TOP SECTION
2. `docs/arc/2026/05/153-rename-unit-to-nil/BRIEF-SUBSTRATE.md` — what sweep 1a shipped
3. `docs/SUBSTRATE-AS-TEACHER.md` — the four-step recipe + Pattern 3
4. `tests/wat_arc153_nil_rename.rs` — canonical post-rename shape
5. Sample diagnostic: `cargo test --release --workspace 2>&1 | head -100` — internalize the BareLegacyUnitName pattern

## Substrate-as-teacher loop

1. `cargo test --release --workspace 2>&1 | head -100` — read errors
2. For each `BareLegacyUnitName` error: open the file:line:col; replace `:wat::core::unit` with `:wat::core::nil`
3. After each batch (per directory bucket per sweep order above), re-run cargo test; verify failure count drops; new errors are still expected shapes
4. ALSO: `grep -rn '()' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/ tests/ src/` for value-position candidates; classify per site; edit
5. Iterate until `cargo test --release --workspace` shows 0 failed
6. Sample-verify by reading 3-5 transformed sites — read cleanly, no obvious shape errors

## Verification

After convergence:

```bash
cargo test --release --workspace 2>&1 | grep -E "test result:|FAILED" | tail -5
```

Expect: 0 failed across all crates.

```bash
cargo test --release --test wat_arc153_nil_rename 2>&1 | tail
```

Expect: 10/10 pass (sweep 1a's tests; some currently passing under unswept-stdlib precondition; others should now pass under clean stdlib).

```bash
grep -rn ':wat::core::unit' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/ tests/ src/ 2>/dev/null | wc -l
```

Expect: 0 (or near-0; any remaining are intentional historical comments / docstrings explaining the rename).

## Out of scope

- Slice 2 closure paperwork (INSCRIPTION + 058 row + USER-GUIDE + WAT-CHEATSHEET + CONVENTIONS update + retire `:wat::core::unit` typealias)
- Lab consumers (`holon-lab-trading/`) — separate workspace

## Reporting (~300 words)

1. **Pre-flight crawl confirmation:** all referenced files read.

2. **Sweep summary:**
   - Type-position transforms: count per directory bucket
   - Value-position transforms: count per directory bucket
   - Total file count modified

3. **Iteration cycles:** how many cargo test runs to converge; failures-cleared per cycle.

4. **Verification:**
   - `cargo test --release --workspace` → 0 failed
   - `cargo test --release --test wat_arc153_nil_rename` → 10/10
   - `grep ':wat::core::unit' ...` → 0 source spellings remain

5. **Latent bugs:** any sites where the transform surfaced pre-existing issues (e.g., a `()` used where a non-unit value was expected; a `:wat::core::unit` annotation that masked a real type mismatch). Flag for follow-up.

6. **Path:** Mode A clean / Mode B substrate-bug / Mode C unexpected-shape / Mode D grinding.

7. **Honest deltas:** any classification ambiguity around value-position `()` (where context made it tricky); any per-directory pattern of unusual let* / lambda /  tuple shapes; any lib test that panicked unexpectedly.

DO NOT write a SCORE doc — orchestrator scores after the atomic commit lands.

DO NOT COMMIT.

## Time-box

120 minutes wall-clock. ScheduleWakeup at T+120 min.

## Why this matters

User direction 2026-05-06: "let's roll." Sweep 1b ships the consumer migration; arc 153 atomic-commits with sweep 1a; arc 136 slice 2 closure runs after; the do form's return positions become `:wat::core::nil` cleanly.

The triplet `nil / Some / None` reads cleanly across the codebase. Wat-rs becomes a Lisp on Rust with vocabulary that honors both traditions.
