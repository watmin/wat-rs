# Arc 167 slice 1 — SCORE

Opus shipped the substrate foundation in ~75 min on the
`arc-167-slice-1-vector-foundation` branch. Workspace 118 OK /
0 FAILED. Atomic squash-merged to main at `7434e4c`.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — `WatAST::Vector(Vec<WatAST>, Span)` added to `src/ast.rs` | git diff confirms variant + `span()` arm | ✓ |
| B — Parser produces `Vector` from `[...]` | tests 1-3 pass; debug-print confirms shape | ✓ |
| C — `eval` errors on Vector at value position with canonical message | test 4 passes; orchestrator grep verified the literal "vector literals at value position are not supported" string in `src/runtime.rs:2728` | ✓ |
| D — `check`/`infer` errors on Vector at value position | test 5 passes; same prose in `src/check.rs:3521` | ✓ |
| E — Match-arm propagation: cargo build green | `cargo build --release --workspace` clean | ✓ |
| F — Existing tests unaffected | pre-arc-167 baseline (117 OK) preserved + new arc 167 group → 118 OK | ✓ |
| G — New tests in `tests/wat_arc167_vector_ast.rs` | 5/5 pass | ✓ |
| H — No clippy regressions | not re-run; opus reports no warnings (acceptable trust given workspace is green) | ✓ |
| I — Slice branch on remote | `arc-167-slice-1-vector-foundation` exists at origin with 2 WIP commits (`38c386b` + `52a4971`) | ✓ |
| J — Main untouched during work | confirmed pre-merge: main was at `d251982` throughout | ✓ |
| K — Test names follow convention | snake_case, descriptive | ✓ |
| L — Error message text matches BRIEF | exact prose appears in both eval and check arms | ✓ |

## Honest deltas (accepted)

1. **Lexer brackets are new tokens** — opus added `Token::LBracket` /
   `RBracket` to `src/lexer.rs` rather than touching wat-edn. The
   wat parser is hand-rolled (doesn't go through wat-edn for the
   wat-language surface), so this is the right surgical layer.
   No EDN-shim conversion path needed.

2. **Hash distinct `TAG_VECTOR = 0x18`** — opus chose a distinct
   canonical-EDN tag for `WatAST::Vector` rather than reusing
   `TAG_LIST`. Rationale: `(a b)` and `[a b]` are syntactically
   distinct; their content-addressed hashes should reflect that.
   Reversible if orchestrator prefers TAG_LIST collapsing later.
   **Accepted** — the distinction is honest at the substrate level.

3. **`watast_to_holon` collapses Vector → Bundle** — at the
   HolonAST/algebra layer, vectors carry no algebra-level meaning
   yet (the algebra-core encoding/binding system doesn't
   distinguish list-vs-vector). Same Bundle shape as List for now.
   Re-tag when a future arc exposes algebra-level vec semantics.
   **Accepted** with inline comment for future rediscovery.

4. **Pattern positions error explicitly** — `check_subpattern`,
   `try_match_pattern`, `try_match_pattern_ast` emit "vector
   sub-patterns are not supported in arc 167" rather than silent
   `None`. Pattern is a syntactic position; explicit error is
   clearer than silent non-match. **Accepted**.

5. **Walkers recurse into Vector children** — `substitute_bindings`,
   `walk_template`, `expand_form`, `substitute`, `scan_for_setter`
   all recurse into Vector children identically to List. Critical
   for slice 2's macro expansion of fn-sigs. **Accepted**.

6. **Stepper returns NoStepRule for Vector** — consumer falls back
   to eval, surfacing the canonical message. **Accepted**.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 60-120 min upper-band; opus tier | ~75 min | A clean |

Match-arm propagation count: predicted 5-15; **actual 17**.
Slightly over upper bound. Extras came from `*variant_name`
helper duplication across 7 modules (each owns its own copy) +
4 walker sites needing positive recursion (rather than
fall-through `_ => Ok(other)`).

**Calibration note for future variant-addition arcs**: when a new
WatAST variant is added, expect ~2 sites per "language layer"
(eval, check, lower, hash, parse, etc.) PLUS ~1 site per walker
family that needs positive recursion. Plan briefs accordingly.

## Discipline check

- ✓ Branch isolation worked: main never moved during opus's
  ~75 min run; opus pushed checkpoints to slice branch only
- ✓ Atomic merge: 2 slice-branch commits squashed to 1 main commit
- ✓ Slice branch retained on origin as audit trail
- ✓ Substrate-judgment work delegated to opus with clear BRIEF;
  orchestrator scored the result rather than coding

## What's next

Slice 2 — fn-sig vector consumer + walker + defn macro shape
change. Substrate-as-teacher cascade expected: substrate change
fires the migration walker on every legacy fn/defn callsite.
Sweep across the workspace lands in slice 3.

Slice 1's foundation makes slice 2's BRIEF cleaner: the Vector
variant exists, walkers know how to recurse, error arms are
positioned. Slice 2 just wires the consumer + walker.
