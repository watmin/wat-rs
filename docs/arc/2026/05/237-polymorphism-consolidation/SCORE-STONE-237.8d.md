# SCORE — Stone 237.8d — equality reclassified as a relational intrinsic; grid residue cut; partition inscribed

Scored against an **independent orchestrator re-run + a HARD READ of the diff**, not the agent's self-report.

## Gates (independent re-run)

| Gate | Expected | Observed | ✓ |
|---|---|---|---|
| `probe_arc237_8d_equality_intrinsic` | 10 / 0 / 0 | **10 passed / 0 failed / 0 ignored** | ✓ |
| `probe_arc237_8c_equality_grid` | green (regression_* kept, mint_f64_* gone) | **5 passed / 0 failed / 0 ignored** | ✓ |
| `cargo test --lib -p wat` | 895 / 0 / 1 | **895 passed / 0 failed / 1 ignored** | ✓ |
| `cargo build --release --tests --workspace` | clean | clean | ✓ |
| clippy (flat files — NOT gated) | no NEW warnings | 274 pre-existing baseline; my changes added none (empty-line warns at check.rs:12509, unrelated) | ✓ |

## HARD READ — scope guard held (the load-bearing check)

From `git diff src/check.rs src/runtime.rs`:

- **`infer_equality` body — UNTOUCHED.** The only edit is a one-line comment inserted *above* `fn infer_equality` (the relational-flavor marker citing `docs/DISPATCH.md`). The equality engine (`infer_equality` / `eval_eq` / `eval_not_eq` / `values_equal`) is byte-identical in behavior. ✓
- **Surgical deletions.** `:i64::=` / `:i64::not=` removed from `register_builtins`' i64 list; the `:f64::=`/`:f64::not=` registration block removed; the four runtime dispatch arms removed. The **ordering** ops (`<`/`>`/`<=`/`>=` for i64 + f64) and the canonical `:wat::core::=`/`not=` (runtime 5652/5653, check 4622) are all **preserved**. ✓
- **Inscription correct.** Both PARTITION markers (check.rs `infer_list`, runtime.rs `dispatch_keyword_head_value`) rewritten to the **two-flavor** rule (projective + relational) citing `docs/DISPATCH.md`; the agent also corrected a stale "define-dispatch" note in passing. The markers now genuinely teach the rule. ✓

## The reject-arms judgment (examinare — why this is NOT a shim)

The agent added explicit reject arms at `check.rs:4636-4643` for the four aliases, emitting `UnknownCallee`. Independently verified the necessity: the `None =>` unregistered-scheme fallback in `infer_list` (~5327) **intentionally does not `UnknownCallee` `:wat::*` forms** (it protects runtime-internal forms like `struct-new`). So removing the registrations alone would leave `(:wat::core::i64::= 1 1)` **check-permissive** (passing the type-checker, failing only at runtime).

Verdict: the reject arms are **legitimate, not a shim.**
- They make the retired form fail at **check time** — failure-engineering-aligned (a removed op should not silently pass the checker; check-time rejection beats a runtime surprise).
- They do **not** make the old form work (the HARD CUT test: a shim keeps a retired form functioning; these make it *fail loudly*).
- They are **idiomatic** with the neighboring special-form arms (`:wat::time::-`, `:wat::form::matches?`) — the same shape, the same `infer_list` match.

## Gate refinement (honest correction to my own EXPECTATIONS)

EXPECTATIONS gate 5 said `grep i64::=\|f64::= → zero matches`. That was **too literal** — written before I knew the permissive `:wat::*` fallback forces explicit check-time rejection. The correct invariant is **"zero LIVE DISPATCH + zero registration,"** which holds: no runtime eval arm routes the aliases, no `env.register` entry exists. The residue is honest: 4 reject-arm keyword names (the cut, made visible at check time) + 3 tombstone comments. Recorded so the next reader doesn't mistake the non-empty grep for an incomplete cut.

## Optional future polish (not blocking)

The reject arms emit bare `UnknownCallee`. A retirement-aware hint ("equality is uniform — use `:wat::core::=`") would be marginally better UX (substrate-as-teacher / remedy). Deferred — `UnknownCallee` is honest (the keyword *is* unknown now), and a hint mechanism is its own concern, out of this stone.

## Verdict

**237.8d PASSES.** Equality is now classified, in code and doctrine, as a relational intrinsic; the grid residue is cut at both runtime and check; the two-flavor partition is inscribed at all three source sites citing `docs/DISPATCH.md`; the equality engine is untouched. The last real work of arc 237 is done.

**NEXT: 237.9 INSCRIPTION** — the arc's victory-story close; flags arc 245 (wat-corpus-warding) unblocked. Then **237 DIES.**
