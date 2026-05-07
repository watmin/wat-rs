# Arc 154 — Substrate BRIEF (slice 1a)

**Drafted 2026-05-06 evening.** Slice 1a of arc 154.

User direction: *"new arc - let's do it"*

## Workspace state pre-spawn

- HEAD: `d883209` (arc 136 closure shipped earlier; arc 153 closed
  earlier same session at `0a48f4c`)
- Working tree clean
- Pre-baseline: `cargo test --release --workspace` = 1988 passed / 0 failed

## Goal

Two coordinated substrate changes:

1. **Switch `:wat::core::let` semantics from parallel to sequential.**
   The current `infer_let_star` / `eval_let_star` / tail-call /
   step paths move under the `let` keyword. The current parallel
   `infer_let` / `eval_let` paths retire (zero consumers per pre-arc
   grep — `grep ':wat::core::let[^*]'` returns 0).

2. **Mint `BareLegacyLetStar` walker** on `:wat::core::let*` Path
   detection per substrate-as-teacher Pattern 3. Mirrors arc 109
   slice 1d's `BareLegacyUnitType` and arc 153's
   `BareLegacyUnitName`. The walker fires `BareLegacyLetStar`
   migration error per offending site; sweep 1b uses it as the
   work list.

## Substrate edits

### `src/check.rs`

1. **Move sequential semantics under `let`.** The current
   `infer_let_star(args, env, ...)` becomes `infer_let(args,
   env, ...)`; the current parallel `infer_let` retires. The
   `infer_list` keyword dispatch arm for `:wat::core::let`
   points at the new (sequential) `infer_let`. The arm for
   `:wat::core::let*` either (a) retires entirely (parser-level
   reject post-walker), or (b) stays as a fall-through that
   the walker has already flagged. Slice 1a chooses based on
   what's cleanest; surface in honest delta.

2. **Mint `CheckError::BareLegacyLetStar` variant** with Display:
   ```
   "{span}: ':wat::core::let*' is retired (arc 154); canonical FQDN
    is ':wat::core::let'. Same sequential semantics, single name."
   ```

3. **Walker** — extend body walker to detect
   `:wat::core::let*` keyword in operator position; emit
   `BareLegacyLetStar` per offending site. Mirror arc 153's
   `walk_type_for_legacy_unit_name` shape.

### `src/runtime.rs`

1. **Move sequential eval under `let`.** Current
   `eval_let_star(args, env, ...)`, `eval_let_star_tail`, and
   `step_let_star` become `eval_let`, `eval_let_tail`,
   `step_let` respectively. The current parallel `eval_let`
   retires.

2. Wire `dispatch_keyword_head` and the tail-call dispatch +
   `step_form` to the new sequential implementations.

### `src/special_forms.rs`

Update `:wat::core::let` registration sketch to reflect
sequential semantics. Decide on `:wat::core::let*` registry
entry: retire with retirement comment, OR keep with walker-fires
note. Sonnet's call; surface in report.

### NEW `tests/wat_arc154_kill_let_star.rs`

6-10 unit tests covering:

1. **`:wat::core::let` accepts sequential bindings.**
   `(let* (((a :i64) 5) ((b :i64) (+ a 1))) b)` typechecks +
   evaluates to 6 — using the NEW `let` keyword.
2. **`:wat::core::let*` fires migration error.** Source code
   using `:wat::core::let*` produces `BareLegacyLetStar`
   per-site.
3. **Type-mismatch in let body still surfaces correctly.**
   Sequential let with body type mismatching declared return
   type fires TypeMismatch.
4. **Tail-call optimization preserved.** A let in tail
   position threads through `eval_let_tail` correctly.
5. **Step-evaluator handles let.** Incremental evaluator
   single-steps through let bindings.
6. **Reflection round-trip:** `lookup-form :wat::core::let`
   returns Binding with sequential semantics; `:wat::core::let*`
   either errors or returns a retired-flag.
7. **Nested lets compose:** outer let's binding visible to
   inner let's body.
8. **Lambda body containing let:** sequential semantics
   preserved across lambda boundaries.
9. **Empty bindings list:** `(let () body)` evaluates body
   directly (degenerate but accepted; mirrors current `let*`
   behavior).
10. **Walker narrowness:** other keywords unchanged; only
    `:wat::core::let*` Path triggers the walker.

Use the existing test harness pattern from
`tests/wat_arc153_nil_rename.rs`.

## Constraints

- **Substrate-only edits.** EXACTLY 4 files: `src/check.rs`,
  `src/runtime.rs`, `src/special_forms.rs`, NEW
  `tests/wat_arc154_kill_let_star.rs`. NO consumer wat edits.
  NO other crate.
- **DO NOT COMMIT.** Working tree stays modified for atomic
  commit with sweep 1b per recovery doc § 7
  atomic-commit-across-coordinated-sweeps.
- **The workspace WILL break post-substrate-change** — every
  existing `:wat::core::let*` site fires `BareLegacyLetStar`.
  EXPECTED. Sweep 1b clears them.
- **STOP at unexpected red.** Distinguish:
  - **Expected:** `BareLegacyLetStar` on existing `:wat::core::let*`
    sites (drives sweep 1b)
  - **Unexpected:** substrate panic, parse error inside
    check.rs/runtime.rs/special_forms.rs, runtime crash,
    TypeMismatch unrelated to let/let*
- No grinding.
- Time-box 60 min wall-clock.

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/154-kill-let-star/DESIGN.md` — full read
2. `docs/arc/2026/05/153-rename-unit-to-nil/BRIEF-SUBSTRATE.md`
   — closest precedent; same recipe
3. `docs/arc/2026/05/153-rename-unit-to-nil/INSCRIPTION.md` —
   note the orchestrator-rewrite version (commit `0a48f4c`),
   not the sonnet-original at `969b847`
4. `docs/SUBSTRATE-AS-TEACHER.md` § "The four-step recipe"
5. `src/check.rs::infer_let_star` + `infer_let` — your edit
   targets (current locations and shapes)
6. `src/check.rs::BareLegacyUnitName` + `walk_type_for_legacy_unit_name`
   — your pattern reference for the new walker (post-retirement
   the walker body is gone but the variant + Display remain)
7. `src/runtime.rs::eval_let_star` + `eval_let_star_tail` +
   `step_let_star` — your runtime edit targets
8. `tests/wat_arc153_nil_rename.rs` — test harness shape

## Pre-flight verification

```bash
cargo test --release --workspace 2>&1 | grep -cE "FAILED"
```

Must be 0.

```bash
grep -rh ':wat::core::let[^*]' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/ tests/ src/ 2>/dev/null | wc -l
```

Should be 0 (no parallel-let consumers; the rename is purely
cosmetic at consumer surface).

## Verification (after edits)

```bash
cargo test --release --test wat_arc154_kill_let_star 2>&1 | tail -10
```

Expect: all 6-10 new tests pass.

```bash
cargo test --release --workspace 2>&1 | head -50
```

Expect: many `BareLegacyLetStar` migration errors on existing
`:wat::core::let*` sites; NO unexpected substrate red.

## Reporting (~250 words)

1. Pre-flight crawl confirmation: DESIGN, arc 153 BRIEF +
   INSCRIPTION (orchestrator-rewrite version), pattern
   references, harness shape all read.
2. Edit summary per file (LOC delta; arms moved; walker added).
3. Verification:
   - new tests pass count
   - workspace failure shape (BareLegacyLetStar count; zero
     unexpected reds)
4. Path: Mode A clean / Mode B substrate-bug / Mode C
   unexpected interaction.
5. Honest deltas: sonnet's call on `:wat::core::let*` registry
   entry (retire entirely vs walker-fires-note); any subtleties
   around the parallel-let retirement (probably none — zero
   consumers); any tail-call / step path edge case.

DO NOT write a SCORE doc — orchestrator scores after sweep 1b
ships and atomic commit lands.

## Time-box

60 minutes wall-clock (predicted upper-bound 30-45 min; 2× cap
matches arc 153 slice 1a profile).

## Why this matters

User direction 2026-05-06 evening: *"new arc - let's do it."*
Slice 1a substrate ships the rename + walker. Sweep 1b migrates
~827 consumer sites. Slice 2 retires the walker body and ships
closure paperwork.

The Lisp on Rust gains a single-letform vocabulary matching
Clojure's user-facing surface. Two foundation marks already
landed today (`nil`, `do`); arc 154 lands the third. Foundation
work for what's being built towards.
