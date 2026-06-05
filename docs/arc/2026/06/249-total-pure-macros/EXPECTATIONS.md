# EXPECTATIONS — Arc 249 Stone 249.1 — threading desugar `->` / `->>`

Scored against an **independent orchestrator re-run on disk**, not the agent's self-report.

## Gates (raw commands)

| # | Command | Pass condition |
|---|---|---|
| G1 | `cargo build --release --tests` | compiles clean |
| G2 | `cargo test --release --test probe_arc249_threading` | **6 passed; 0 ignored; 0 failed** |
| G3 | `cargo test --release --lib -p wat` | **895 passed; 0 failed; 1 ignored** (baseline unchanged) |
| G4 | `git diff --stat` (orchestrator) | only `src/macros.rs` + `tests/probe_arc249_threading.rs` (the un-ignore) touched — **no** check.rs/runtime.rs/special_forms.rs/lexer.rs/parser.rs/`wat/*` |

## Scorecard rows (each verified by re-running, not trusting the report)

1. **thread-last single-step** — `mint_thread_last_single_step` green; `(->> [1 2 3] (map F))` → `[2 3 4]`.
2. **thread-last pipeline** — `mint_thread_last_pipeline` green; `[3 4]` (the 2-step compose — the 247 raison d'être).
3. **thread-first first-inject** — `mint_thread_first_injects_first` green; `(-> 5 (i64::- 3))` → `2`.
4. **thread-last last-inject ≠ first** — `mint_thread_last_injects_last` green; `-2` (proves first≠last).
5. **bare-symbol step** — `mint_bare_symbol_step` green; `(-> 3 :my::inc)` → `4`.
6. **regression intact** — `regression_fn_first_map_no_threading` still green (harness + fn-first map untouched).
7. **disambiguation** — implied by row 3 (the `->` return-arrow and the thread-first head coexist in one form) AND G3 (every existing `-> :Ret` signature in the lib still checks).
8. **scope honesty** — G4: the diff is `src/macros.rs` only (+ the probe un-ignore). Any other touched file is an automatic Mode-B (the design says zero check/runtime changes).

## Independent prediction (runtime band)

**8–20 min, Mode A.** One `expand_form` arm + one fold helper in `src/macros.rs`; the
`keyword/of` precedent is a near-exact template; the cascade is **zero** (nothing else in the
tree references `->`/`->>` as a head). The risk is entirely local: the fold's first-vs-last
injection and the empty-form arity guard. No substrate-wide ripple.

**2× wakeup cap: 40 min.** If the agent exceeds it, `TaskStop` + score Mode-B-time-violation.

## Failure-profile expectations

- **If the agent edits check.rs/runtime.rs:** Mode B — it misread the desugar-not-special-form
  design. The forms must vanish before type-check.
- **If a mint stays red:** most likely the first/last injection is swapped, or the bare-symbol
  (non-list) step isn't wrapped into `(f acc)`. Re-read the rewrite table.
- **If G3 regresses:** the new arm is matching `->` too broadly (e.g. firing on the infix arrow,
  or recursing wrongly) — it must fire **only** when `->`/`->>` is `expanded_children.first()`.
