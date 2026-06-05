# SCORE — Arc 249 Stone 249.1 — threading desugar `->` / `->>`

Scored against an **independent orchestrator re-run on disk**, not the agent's self-report. The
agent reported clean; this score is what the disk says when I re-ran it.

## Verdict: REMARKABLE — one-shot, no R2.

## Gates (re-run by the orchestrator)

| # | Gate | Result |
|---|---|---|
| G1 | `cargo build --release --tests` | clean (warnings pre-existing) |
| G2 | `cargo test --release --test probe_arc249_threading` | **6 passed; 0 failed; 0 ignored** ✓ |
| G3 | `cargo test --release --lib -p wat` | **895 passed; 0 failed; 1 ignored** (baseline unchanged) ✓ |
| G4 | `git diff --stat` | `src/macros.rs` (+97) + `tests/probe_arc249_threading.rs` (−5 `#[ignore]`) — **only** these; no check/runtime/special_forms/lexer/parser/`wat/*` ✓ |

## Scorecard

| Row | Claim | Verified |
|---|---|---|
| 1 | thread-last single-step → `[2 3 4]` | ✓ `mint_thread_last_single_step` green |
| 2 | thread-last pipeline → `[3 4]` | ✓ `mint_thread_last_pipeline` green |
| 3 | thread-first first-inject `(-> 5 (i64::- 3))` → 2 | ✓ `mint_thread_first_injects_first` |
| 4 | thread-last last-inject → -2 (≠ first) | ✓ `mint_thread_last_injects_last` |
| 5 | bare-symbol step `(-> 3 :my::inc)` → 4 | ✓ `mint_bare_symbol_step` |
| 6 | regression (fn-first map, no threading) intact | ✓ `regression_fn_first_map_no_threading` |
| 7 | disambiguation — `->` return-arrow + thread head coexist | ✓ (row 3's form has both; G3 confirms all `-> :Ret` sigs still check) |
| 8 | scope honesty — `macros.rs` only | ✓ G4 |

## The implementation (read, not trusted)

`src/macros.rs`: a recognition arm in `expand_form` (between the `keyword/of` built-in and the
registered-macro dispatch) matching bare `WatAST::Symbol("->")`/`("->>")` heads, dispatching to a
new `thread_desugar` left-fold helper. First/last injection via `insert(1, acc)` / `push(acc)`;
empty form → `MacroErrorKind::ArityMismatch`; single element → identity. Carries the outer
`list_span` onto every constructed node (call-site span convention). The desugar returns ordinary
nested `WatAST::List` calls — the checker/runtime never see `->`/`->>`, exactly as the DESIGN
specified. **Zero check/runtime changes**, confirming the desugar-not-special-form thesis.

The agent additionally guarded a degenerate empty-list step `()` (wrap as `(acc)`) — out of the
probe's scope but a harmless, correct totality choice; left as-is.

## Why one-shot (calibration note)

Predicted 8–20 min Mode A; actual ~4 min (240s), well under band. The work was front-loaded into
the orchestrator crawl: the `keyword/of` built-in was a near-exact template, the cost was correctly
identified as a desugar (not a special form), every lexing/disambiguation assumption was grounded
before the BRIEF, and the FM-2-bis probe pinned the contract executably. Mechanical work + precise
scope + a disconfirming probe = first-try landing. The pattern holds: the crawl is the work.

## Disposition

Stone 249.1 verified green. Committed with this SCORE. **Next: 249.N INSCRIPTION** (FM-11 clean —
the arc ships both forms, no deferral) → gate advances **249 ✓ → 235 → rejoin 232.**
