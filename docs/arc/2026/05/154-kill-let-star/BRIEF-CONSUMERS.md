# Arc 154 — Consumer Sweep BRIEF (slice 1b)

**Drafted 2026-05-06 evening.** Sweep 1b of arc 154.

User direction: *"new arc - let's do it"*

## Workspace state pre-spawn

- HEAD: `bd27820` (arc 154 DESIGN + slice 1a BRIEF + EXPECTATIONS)
- Working tree DIRTY with sweep 1a substrate edits (4 files):
  - `src/check.rs` (BareLegacyLetStar variant + walker; sequential let)
  - `src/runtime.rs` (eval_let / eval_let_tail / step_let — sequential)
  - `src/special_forms.rs` (let registration sequential; let* registry retained-with-retirement-slot)
  - NEW `tests/wat_arc154_kill_let_star.rs` (10 tests; 3 pass, 7 blocked by stdlib pre-sweep state)
- Pre-baseline (post-substrate): ~1260 `BareLegacyLetStar` migration errors firing on `:wat::core::let*` sites + ~72 downstream panics in lib tests where `assert!(check(...).is_ok())` fails because stdlib check is dirty. EXPECTED per atomic-commit-across-coordinated-sweeps.

## Goal

Mechanical 1:1 transform: every `:wat::core::let*` site → `:wat::core::let` across the entire codebase. Workspace returns to 0 failed when the sweep is structurally complete.

## The transform

```scheme
;; Before
(:wat::core::let*
  (((a :i64) 5)
   ((b :i64) (:wat::core::i64::+ a 1)))
  (:wat::core::i64::+ a b))

;; After
(:wat::core::let
  (((a :i64) 5)
   ((b :i64) (:wat::core::i64::+ a 1)))
  (:wat::core::i64::+ a b))
```

Identical body shape — only the keyword changes. Sequential semantics preserved (always was sequential under let*; now is sequential under let too).

## Sweep order (per substrate-as-teacher § "stdlib first")

1. **`wat/*.wat`** stdlib (binary loads on every wat invocation; ~141 sites)
2. **`crates/*/wat/**/*.wat`** per-crate substrates
3. **`wat-tests/**/*.wat`** workspace test wat
4. **`crates/*/wat-tests/**/*.wat`** per-crate test wat
5. **`examples/**/*.wat`**
6. **Embedded wat in `tests/*.rs`** (~391 sites combined with src/)
7. **Embedded wat in `src/*.rs`** lib tests

After step 1, re-run cargo test to confirm stdlib boots clean (BareLegacyLetStar count drops dramatically; downstream panics start clearing).

## Constraints

- **DO COMMIT + PUSH** when workspace = 0-failed (atomic with slice 1a).
- **NO substrate edits** (`src/*.rs` Rust code body — but embedded wat strings inside `src/*.rs` lib tests COUNT and need migration).
- **NO `holon-lab-trading/` edits** (separate workspace).
- **STOP at unexpected red.** Distinguish:
  - Expected: `BareLegacyLetStar` walker fire on remaining unmigrated sites
  - Expected: pre-existing intentional thread-panic tests (e.g., `assertion-failure` tests; `expect-fail` tests)
  - Unexpected: substrate panic, parse error inside check.rs/runtime.rs, runtime crash, TypeMismatch unrelated to let/let* migration
- No grinding (>3 reads/edits per site = surface as Mode D).
- Time-box 120 min wall-clock.

## Pre-flight crawl (mandatory)

1. `docs/arc/2026/05/154-kill-let-star/DESIGN.md`
2. `docs/arc/2026/05/154-kill-let-star/BRIEF-SUBSTRATE.md` (slice 1a's contract)
3. `docs/arc/2026/05/153-rename-unit-to-nil/BRIEF-CONSUMERS.md` — closest precedent; same recipe; ~75 min wall-clock
4. `docs/SUBSTRATE-AS-TEACHER.md` four-step recipe + Pattern 3
5. `tests/wat_arc154_kill_let_star.rs` — canonical post-rename shape

## Sweep strategy

1. `grep -rln ':wat::core::let\*' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/ tests/ src/` to scope
2. Per file: replace every `:wat::core::let*` keyword occurrence with `:wat::core::let`
3. Batch by directory; run cargo test between major batches
4. Identical-shape transform — no semantic concerns, no per-site classification
5. The walker fires per-site; the diagnostic stream IS the work list

## Verification

- `cargo test --release --workspace`: 0 failed (atomic with sweep 1a; 1988+10 = ~1998 tests passing post-sweep)
- `cargo test --release --test wat_arc154_kill_let_star`: 10/10 pass (the 7 currently-blocked positive tests unblock when stdlib is clean)
- `grep -rln ':wat::core::let\*' wat/ wat-tests/ crates/*/wat/ crates/*/wat-tests/ examples/`: 0 source spellings (only intentional fixtures in `tests/wat_arc154_kill_let_star.rs` may remain — those are negative-test sources)

## Reporting (~250 words)

1. Pre-flight crawl confirmation: all referenced files read
2. Sweep summary per directory bucket: file count + transform count
3. Iteration cycles: cargo test runs to convergence + wall-clock per cycle
4. Verification: workspace 0-failed; arc154 tests 10/10; grep count 0 source spellings
5. Path classification (Mode A/B/C/D)
6. Honest deltas: any sites where transform was non-trivial; any class of files heavier/lighter than expected; any latent bugs surfaced

DO NOT write a SCORE doc — orchestrator scores after atomic commit lands.

DO NOT COMMIT individually — orchestrator atomically commits sweep 1a + sweep 1b together when workspace = 0-failed (per recovery doc § 7).

## Time-box

120 minutes wall-clock. ScheduleWakeup at T+120 min.

## Why this matters

User direction 2026-05-06 evening: *"new arc - let's do it."* Sweep 1b is the consumer migration completing arc 154. After atomic commit, slice 2 (substrate retirement + closure paperwork orchestrator-side) ships next.

The Lisp on Rust gains its single-letform vocabulary. Three foundation marks in one session: `nil` (arc 153), `do` (arc 136), `let` (arc 154). The substrate's user-facing surface keeps consolidating.
