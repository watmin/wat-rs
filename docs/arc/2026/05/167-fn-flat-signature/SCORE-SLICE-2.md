# Arc 167 slice 2 — SCORE

Opus shipped the substrate consumer + walker + defn macro on the
`arc-167-slice-2-fn-sig-consumer` branch in ~75 min. Mode A clean.
Branch holds 2 WIP commits (`e30a97f` + `b557b77`); main untouched
at `20eee0b`.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — `parse_fn_signature` consumes Vector body | `eval_fn` accepts new shape; tests 1, 4, 7 pass | ✓ |
| B — `parse_fn_signature_for_check` parallel update | type-check infers correctly; tests 1, 7 pass | ✓ |
| C — `BareLegacyFnSignature` variant + Display | git diff confirms variant + Display impl with verbatim migration text from BRIEF | ✓ |
| D — Walker fires on legacy nested-sig | tests 5, 6 pass | ✓ |
| E — Walker wired into pipeline | freeze.rs:599-616 (user-source pre-pass per honest delta A) | ✓ accepted |
| F — Defn macro shape updated with rest-binder | git diff confirms new variadic shape | ✓ |
| G — Tests 1-4, 7-9 pass | 7/9 success cases pass | ✓ |
| H — Tests 5, 6 pass (walker firing) | walker-firing assertions pass | ✓ |
| I — `cargo build --release --workspace` green | substrate compiles cleanly | ✓ |
| J — `cargo test --release --test wat_arc167_fn_flat_signature` | 9/9 pass | ✓ |
| K — Workspace failure count reported | 152 failed across 18 test targets; lib unit tests stay 793 passed / 0 failed | ✓ (slice 3 input) |
| L — Slice branch on remote | `arc-167-slice-2-fn-sig-consumer` ff-merged with 2 opus commits | ✓ |
| M — Main untouched | `origin/main` unchanged at `20eee0b` throughout | ✓ |

## Honest deltas (accepted with slice 4 scope expansion)

The four questions ran on each delta; all favored acceptance.
Decision committed: slice 4 scope expands to retire BOTH walker
scaffolding AND the transitional legacy parser arms.

### Delta A — Walker placement: user-source pre-pass (NOT check_program)

The BRIEF said "wire into `check_program`." Doing so would have
fired the walker on 30+ stdlib legacy fn-sigs in `wat/core.wat`,
`wat/stream.wat`, etc. — making every `startup_from_source` call
fail before tests could exercise the new shape.

Opus pivoted: walker placed in `freeze.rs:599-616` pre-pass
alongside `validate_bare_legacy_primitives`, scoped to user forms
only. Mirrors arc 163 slice 3g phase A precedent (the same fix
shape for the same problem). The freeze.rs comment explicitly
states "stdlib forms are substrate-authored and audited via in-repo
discipline" — this is the established pattern.

Implication for slice 3: sweep covers user-test-files driven by
cargo test failures + grep-driven sweep of `wat/*.wat` +
`wat-tests/*.wat` (stdlib silently migrates without walker firing).

### Delta B — Transitional legacy parser arms kept active

The BRIEF said "If `args[0]` is a `WatAST::List` (legacy
nested-sig), DO NOT parse it." Opus kept both shapes parseable:
- `parse_fn_signature` (new flat-shape canonical path)
- `parse_legacy_fn_signature` (transitional fallback)
- Same on the check side via `parse_fn_signature_for_check` /
  `parse_legacy_fn_signature_for_check`

Reason: if the legacy parser dies at slice 2, stdlib breaks before
slice 3 can fix it. This is the substrate-as-teacher's
architectural-precedent variant: the dispatch arm survives
during the migration window; firing retires after sweep clears.

Implication for slice 4: scope expands to also DELETE the legacy
parser arms (both runtime + check). Without this, slice 4 wouldn't
actually retire the legacy syntax — it'd retire only the walker.
Slice 4 must close both holes.

### Delta C — Slice 1 follow-on lexer bug

Slice 1 added `Token::LBracket / RBracket` but `lex_keyword` didn't
break on `]`. Slice 2 surfaced this because slice 2 is the first
arc to put a keyword adjacent to `]` (in `[x <- :wat::core::i64]`
the keyword would absorb the `]`). Two-line equivalence fix in
`src/lexer.rs:411-440`.

This is a slice 1 leftover slice 2 caught + fixed. Honest.

### Delta D — eval_fn arity reshape supports both shapes

Same architectural decision as delta B at the dispatch layer.
`eval_fn` accepts `args.len() == 4` (canonical) or `args.len() == 2`
(legacy). Same Function value produced either way.

### Delta E — `startup_err` test helper joins Display + Debug

Per arc 154/166 precedent, the helper used `Debug` only. Opus
extended to `format!("{}\n---\n{:?}", e, e)` so tests can assert
against migration-message-text (Display) OR variant-name (Debug).
Backward-compatible.

### Delta F — Walker scoping success

No false-positives. Walker descends past `(:wat::core::fn LIST ...)`
for siblings (skipping legacy-sig contents) and recurses through
both `WatAST::List` and `WatAST::Vector` children for nested cases.

### Delta G — Lib unit tests (793) stay green

Predicted to fail; don't, because of deltas A + B (walker scoped
to user forms; legacy parser keeps lib code running).

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 60-90 min upper-band; opus tier | ~75 min | A clean |

Failure count for slice 3 calibration: 152 (within EXPECTATIONS
K-row "50-200 → ~60-120 min sonnet" band).

## Discipline check

- ✓ Branch isolation worked: main untouched during opus's run
- ✓ Walker migration message verbatim from BRIEF (slice 3's
  mechanical translation depends on it)
- ✓ Slice 4 scope adjusted post-deltas — captured here for the
  drafter
- ✓ The four questions ran on each delta before acceptance

## What's next

Slice 3 — bundled sweep (test-driven + grep-driven) per the
substrate-as-teacher discipline. Predicted sonnet-tier ~60-120
min based on the 152 failure count + ~30 stdlib grep hits. BRIEF
and EXPECTATIONS draft to main; sonnet works on the same branch.

The slice 2 branch stays open through slice 3; atomic merge to
main happens after slice 3 ships green.
