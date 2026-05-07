# Arc 155 — Substrate BRIEF (slice 1a)

**Drafted 2026-05-06 evening.** Slice 1a of arc 155.

User direction: *"hold... /everything/ needs a namespace..
:wat::core::Fn to align /with everthing/ else"*

## Workspace state pre-spawn

- HEAD: `d7991d5` (arc 154 closure shipped)
- Working tree clean
- Pre-baseline: `cargo test --release --workspace` = 1998 / 0

## Goal

Two coordinated substrate changes for two parallel renames bundled
in one arc:

### Type-position rename: `:fn(...)` → `:wat::core::Fn(...)`

Closes arc 109 slice 1e's last ungrabbed parametric type head
(slice 1e FQDN'd `Option`, `Result`, `HashMap`, `HashSet`; arc
155 closes the fifth, `Fn`).

- Mint `:wat::core::Fn` as the canonical FQDN parametric type
  for function types
- Walker `walk_for_legacy_lowercase_fn` detects bare `:fn` at
  type position; emits `BareLegacyLowercaseFn` per site
- Bare `:fn(...)` parses cleanly during migration window
  (transitional alias) so consumer tests keep working until
  sweep 1b clears them

### Operator-position rename: `:wat::core::lambda` → `:wat::core::fn`

Mirror arc 154's `:wat::core::let*` → `:wat::core::let` recipe
exactly.

- `infer_lambda` body moves under `:wat::core::fn` keyword;
  retire `infer_lambda` arm
- `eval_lambda` body moves under `:wat::core::fn` keyword;
  retire `eval_lambda` arm
- `:wat::core::lambda` dispatch arms keep functional fall-through
  to `eval_fn` (transitional runtime scaffolding; mirrors arc 154's
  let* fall-through pattern)
- Walker `walk_for_legacy_lambda` detects `:wat::core::lambda`
  keyword; emits `BareLegacyLambda` per site
- Special-forms registry: `:wat::core::fn` minted with sketch
  matching lambda's current shape; `:wat::core::lambda` registry
  entry retained per spawn-family precedent (arc 114 Pattern 2
  poison)

## Substrate edits

### `src/check.rs`

1. **`BareLegacyLambda` variant** (mirror arc 154's
   `BareLegacyLetStar` shape exactly): variant + Display referencing
   arc 155 + canonical `:wat::core::fn`.

2. **`BareLegacyLowercaseFn` variant**: variant + Display referencing
   arc 155 + canonical `:wat::core::Fn`.

3. **Type-position walker** `walk_for_legacy_lowercase_fn`: detects
   `TypeExpr::Parametric` with head `:fn` (lowercase, bare); emits
   `BareLegacyLowercaseFn` per site. Mirror arc 109 slice 1e's
   walker structure for retired bare parametric heads.

4. **Operator-position walker** `walk_for_legacy_lambda`: detects
   `WatAST::Keyword(":wat::core::lambda")` in operator position;
   emits `BareLegacyLambda` per site. Mirror arc 154's
   `validate_legacy_let_star` shape.

5. **Inference dispatch**: `:wat::core::fn` arm routes to renamed
   `infer_fn` (formerly `infer_lambda`); `:wat::core::lambda` arm
   retained as fall-through (mirrors arc 154's let* fall-through).

6. **Type registry**: `:wat::core::Fn` registered as canonical
   parametric type head; bare `:fn` retained transitionally (sweep
   1b clears it).

### `src/runtime.rs`

1. **Eval dispatch**: `:wat::core::fn` arm routes to renamed
   `eval_fn` (formerly `eval_lambda`); `:wat::core::lambda` arm
   retained as fall-through.

2. **Tail-call paths**: same pattern (`:wat::core::fn` → eval_fn_tail;
   `:wat::core::lambda` → fall-through).

3. **Step paths**: same pattern.

### `src/special_forms.rs`

1. Mint `:wat::core::fn` registry entry with sketch matching
   lambda's current `(<params>+ <body>+)` shape.

2. Retain `:wat::core::lambda` registry entry per spawn-family
   precedent (slot value: `<retired-use-fn>`).

### NEW `tests/wat_arc155_fn_rename.rs`

8-12 tests covering both renames:

1. **Type-position canonical:** `(:wat::core::define (:probe (f (:wat::core::Fn(:i64)->:i64)) -> :i64) (f 5))` type-checks
2. **Type-position retired:** bare `:fn(...)` fires `BareLegacyLowercaseFn`
3. **Operator-position canonical:** `(:wat::core::fn ((x :i64) -> :i64) (:+ x 1))` works
4. **Operator-position retired:** `:wat::core::lambda` fires `BareLegacyLambda`
5. **Mixed (canonical both):** function-typed parameter receives a fn-formed lambda
6. **Walker narrowness — type:** `:wat::core::fn` (lowercase, FQDN'd) is NOT detected as legacy (only bare `:fn`)
7. **Walker narrowness — operator:** `:wat::core::fn` operator works; `:wat::core::lambda` flagged
8. **Reflection round-trip:** `lookup-form :wat::core::fn` returns the canonical Binding
9. **Tail-call sanity:** fn body in tail position threads through eval_fn_tail
10. **Pre-existing test compat:** an existing arc-NNN test that uses the new spelling works (sonnet picks one; verifies the migration path is open)

Use the existing test harness pattern from
`tests/wat_arc154_kill_let_star.rs`.

## Constraints

- **Substrate-only edits.** EXACTLY 4 files: `src/check.rs`,
  `src/runtime.rs`, `src/special_forms.rs`, NEW
  `tests/wat_arc155_fn_rename.rs`. NO consumer wat edits. NO
  other crate.
- **DO NOT COMMIT.** Working tree stays modified for atomic
  commit with sweep 1b per recovery doc § 7
  atomic-commit-across-coordinated-sweeps.
- **The workspace WILL break post-substrate-change** — every
  existing `:fn(...)` type-position site fires
  `BareLegacyLowercaseFn`; every `:wat::core::lambda` site fires
  `BareLegacyLambda`. EXPECTED. Sweep 1b clears them.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** `BareLegacyLowercaseFn` on bare `:fn(...)` type sites + `BareLegacyLambda` on `:wat::core::lambda` operator sites
  - **Unexpected:** anything else (substrate panic, parse error,
    unrelated TypeMismatch)
- No grinding.
- Time-box 75 min wall-clock (1.5x arc 154's slice 1a — bundled
  two-rename complexity).

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/155-fn-rename/DESIGN.md` — full read
2. `docs/arc/2026/05/154-kill-let-star/BRIEF-SUBSTRATE.md` —
   closest precedent for operator-position rename
3. `docs/arc/2026/05/154-kill-let-star/INSCRIPTION.md` (orchestrator
   rewrite at d7991d5) — close-to-end shape of the let* recipe
4. **arc 109 slice 1e** INVENTORY notes if any — closest
   precedent for FQDN'ing a parametric type head
5. `docs/SUBSTRATE-AS-TEACHER.md` four-step recipe + Pattern 3
6. `src/check.rs::infer_lambda` + parametric type registration —
   your edit targets
7. `src/runtime.rs::eval_lambda` + tail/step paths — your runtime
   edit targets
8. `tests/wat_arc154_kill_let_star.rs` — test harness shape

## Pre-flight verification

```bash
cargo test --release --workspace 2>&1 | grep -cE "FAILED"
```

Must be 0.

## Verification (after edits)

```bash
cargo test --release --test wat_arc155_fn_rename 2>&1 | tail -10
```

Expect: most new tests pass; some positive-case tests may be
blocked by stdlib pre-sweep state (mirrors arc 154 slice 1a
pattern — 7 of 10 tests blocked there).

```bash
cargo test --release --workspace 2>&1 | head -50
```

Expect: many `BareLegacyLambda` + `BareLegacyLowercaseFn` errors
firing on existing sites; NO unexpected substrate red.

## Reporting (~250 words)

Per BRIEF: pre-flight crawl confirmation; edit summary per file;
LOC delta; verification (new tests pass count + workspace
failure shape — both walker variants firing as expected); path
classification (Mode A/B/C); honest deltas (especially around
`:fn` vs `:wat::core::Fn` parser interaction at type position).

DO NOT write a SCORE doc — orchestrator scores after sweep 1b
ships and atomic commit lands.

## Time-box

75 minutes wall-clock (2x predicted upper-bound).

## Why this matters

User direction 2026-05-06: *"we're moving closer to clojure"* +
*"everything needs a namespace."* Arc 155 ships two coordinated
renames — Cap'd FQDN'd `Fn` for function types; lowercase FQDN'd
`fn` for function values. Closes arc 109 slice 1e's last
ungrabbed parametric type head. Three foundation marks landed
today (nil + do + let sequential); arc 155 lands the fourth.
