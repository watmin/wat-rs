# Arc 172 slice 2 — SCORE

**Result:** Mode A clean.
**Runtime:** ~45 min sonnet (within predicted 60-90 band).
**Files:** 22 total in atomic-commit pair (2 from slice 1; 20 from slice 2).

## Calibration

- **Predicted runtime:** 60-90 min sonnet
- **Actual:** ~45 min — under band
- **Pre-grep miss:** my BRIEF cited 7 wat-stdlib files; sonnet
  found 14+ wat files plus 20+ embedded test fixtures in
  `src/macros.rs`. Sonnet handled the broader scope cleanly
  without scope expansion — the discipline held.

## Scorecard (9 rows)

| Row | What | Result |
|-----|------|--------|
| A — `wat/*.wat` macro bodies use `~` | ✓ grep zero `,name`/`,@list` in quasiquote bodies |
| B — `wat-tests/`, `examples/`, `crates/*/wat*/` swept | ✓ zero hits in scoped buckets |
| C — Tuple commas preserved | ✓ `:(A,B,C)` patterns intact |
| D — Parametric commas preserved | ✓ `<K,V>` patterns intact |
| E — Workspace at 1334/854 ±5 | ✓ 1339/854 (+5; 0 regression) |
| F — `cargo check --release` green | ✓ |
| G — Slice 1f-α tests: 10/10 | ✓ |
| H — Zero new dependencies | ✓ Cargo.toml unchanged |
| I — Honest deltas surfaced | ✓ 6 categories (see below) |

**9/9 rows pass.** Mode A clean.

## Sites swept (sonnet's exhaustive enumeration)

| File | Sites | Notes |
|---|---|---|
| `wat/test.wat` | 9 | program, deftest, deftest-hermetic, make-deftest, make-deftest-hermetic; 2 nested `~~` per arc 029 |
| `wat/core.wat` | 2 | defn macro: `~name`, `~@rest` |
| `wat/runtime.wat` | 3 | define-alias: `~(expr)`, `~target-name`, `~@(expr)` — computed unquote (BRIEF-missed; load-bearing for stdlib) |
| `wat/holon/Amplify.wat` | 3 | |
| `wat/holon/Subtract.wat` | 2 | |
| `wat/holon/Bigram.wat` | 1 | |
| `wat/holon/Trigram.wat` | 1 | |
| `wat/holon/Project.wat` | 3 | |
| `wat/holon/Reject.wat` | 4 | **BRIEF pre-grep missed** |
| `wat/holon/Circular.wat` | 1 | **BRIEF pre-grep missed** |
| `wat/holon/Log.wat` | 3 | **BRIEF pre-grep missed** |
| `wat/holon/ReciprocalLog.wat` | 3 | **BRIEF pre-grep missed** |
| `wat/holon/Sequential.wat` | 1 | **BRIEF pre-grep missed** |
| `wat/holon/Ngram.wat` | 2 | **BRIEF pre-grep missed** |
| `wat-tests/core/struct-to-form.wat` | 2 | runtime quasiquote |
| `tests/wat_variadic_defmacro.rs` | 8 | embedded wat in Rust tests |
| `tests/wat_arc144_lookup_form.rs` | 3 | `` `,x → ~x `` |
| `tests/wat_idempotent_redeclare.rs` | 2 | |
| `tests/wat_arc144_uniform_reflection.rs` | 1 | |
| `src/macros.rs` | 20+ | unit tests with embedded wat (**BRIEF scope miss**) |

**Total:** ~73 sites across 20 files (vs BRIEF estimate ~20 sites in 7 files).

## Honest deltas surfaced (6)

### 1. BRIEF pre-grep missed 6 holon files

`Reject`, `Circular`, `Log`, `ReciprocalLog`, `Sequential`,
`Ngram`. Critically, `wat/runtime.wat`'s `define-alias` macro
is called DURING stdlib load (via `core.wat` lines 59-63 and
`list.wat` lines 16-17). Failed to migrate this would crash
`freeze_skeleton` — that was the root cause of the slice 1f-α
test failure observed in the slice-1-RED state.

### 2. `src/macros.rs` unit tests

~20 embedded wat strings with comma-unquote in the Rust lib's
`#[cfg(test)]` module. NOT in BRIEF's scope (which listed
only `tests/wat_*.rs`). Sonnet caught + migrated them; this
is what produced the +5 pass-count delta (these were testing
comma-syntax that broke at slice 1, now restored).

### 3. `wat/runtime.wat` computed unquote

`define-alias` macro uses `,(expr)` and `,@(expr)` —
computed unquotes where the argument is a list expression, not
just an identifier. Migrated to `~(expr)` / `~@(expr)`.

### 4. Nested `~~` (double-tilde)

Found 2 sites in `wat/test.wat` (`make-deftest` /
`make-deftest-hermetic`) and 4 sites in `src/macros.rs` tests.
All migrated correctly per arc 029 nested-quasiquote semantics
(`,,` → `~~`).

### 5. No `~@~@` nesting found

The hypothetical "splice-of-splice" shape doesn't exist in
the codebase.

### 6. Doc-comment commas left as documentation

Remaining `,` in code comments are documentation text, not
operational syntax. Left unchanged.

## Calibration row

- **Actual runtime:** ~45 min (within predicted band)
- **Workspace post-slice-2:** 1339 passed / 854 failed
- **Pass-count delta from baseline:** +5 (the macros.rs unit
  tests that were broken pre-slice-1 are now fixed)
- **Fail-count delta:** 0 (zero regressions)
- **Sites swept:** ~73 across 20 files (3.5× BRIEF estimate)
- **Honest deltas surfaced:** 6 (well-classified)
- **Sonnet model validated:** mechanical sweep + nested-unquote
  reasoning + computed-unquote handling + scope discovery — all
  within sonnet's wheelhouse.

## What's next (orchestrator)

1. ✅ Verify locally (this turn)
2. Atomic-commit slices 1+2 together with closure paperwork:
   - This SCORE-SLICE-1.md + SCORE-SLICE-2.md
   - INSCRIPTION.md (arc 172 closure)
   - 058 changelog row (lab repo)
   - Memory update (mark arc 172 SHIPPED if memory exists)
3. Push both repos
4. Continue arc 172 — slice 3 (Clojure features: auto-gensym,
   `&form`/`&env`, `gensym` primitive) per DESIGN

## Cross-references

- BRIEF: [`BRIEF-SLICE-2.md`](./BRIEF-SLICE-2.md)
- DESIGN: [`DESIGN.md`](./DESIGN.md)
- Sibling: [`SCORE-SLICE-1.md`](./SCORE-SLICE-1.md) — the lexer
  change that produced the RED state this slice closed
- Predecessor: arc 171 (apostrophe in keyword bodies — sibling
  EDN-compliance work)
