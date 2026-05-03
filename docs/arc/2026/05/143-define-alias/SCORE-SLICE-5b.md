# Arc 143 Slice 5b — SCORE

**Sweep:** sonnet, agent `a4f5bebec8af56435`
**Wall clock:** ~7.8 min (under the 14-min cap; in band)
**Output verified:** orchestrator re-ran the slice 6 foldl test and
workspace tests.

**Verdict:** **MODE A — clean ship with HONEST SCOPE EXPANSION.** All
6 hard rows pass. Sonnet exercised judgment to extend the scope from
1-line fix to 4 fixes, ALL within `src/runtime.rs` (file scope held),
ALL necessary to achieve the brief's load-bearing test transition.
The scope expansion exposed two latent prior-slice bugs.

## Hard scorecard (6/6 PASS)

| # | Criterion | Result |
|---|---|---|
| 1 | File diff scope (1 Rust file) | ✅ Only `src/runtime.rs` modified. NO other files. |
| 2 | New arm in `value_to_watast` | ✅ `Value::holon__HolonAST(h) => Ok(holon_to_watast(&h))` at runtime.rs:5890 (between wat__WatAST arm and catch-all). |
| 3 | Unit test for new path | ✅ `arc143_slice5b_value_to_watast_accepts_holon_ast` in src/runtime.rs::tests, asserts HolonAST::symbol(":foo") converts to WatAST::Keyword(":foo"). PASSES. |
| 4 | **Slice 6 foldl test transitions** | ✅ `define_alias_foldl_to_user_fold_delegates_correctly` PASSES (was FAILING with Gap 1; now resolves). The load-bearing end-to-end verification. |
| 5 | `cargo test --release --workspace` | ✅ Exit non-zero ONLY because of: 1 slice 6 length test (Gap 2) failure. The pre-existing LRU failure is in `cargo test -p wat-lru` (separate crate not in --workspace). ZERO new regressions. |
| 6 | Honest report | ✅ Detailed report covers the 1-line fix + 3 honest-delta fixes + verbatim test bodies + transition confirmation + test totals. |

## Honest deltas — three additional fixes (slice 1 + 3 latent bugs)

The brief specified ONE 1-line fix. Sonnet shipped THREE additional
fixes in `src/runtime.rs`, each necessary to achieve the brief's
load-bearing test transition. These exposed LATENT BUGS in slice 1
and slice 3 code:

### Delta 1 — `type_scheme_to_signature_ast` Keyword/Symbol mismatch (SLICE 1 BUG)

**At** `src/runtime.rs:~6035`. Slice 1 emitted param names as
`WatAST::Keyword(":_a0", ...)` in the synthesized signature head.
But `:wat::core::define`'s parser requires `WatAST::Symbol` for
param-name positions. Sonnet fixed to
`WatAST::Symbol(Identifier::bare("_a0"), ...)`.

**Why slice 1's tests didn't catch it:** slice 1's tests inspect the
AST shape (Bundle structure, symbol strings) — they don't RE-PARSE
the synthesized AST as a callable define. The bug was latent until
slice 6 quasiquoted the head into a fresh define.

### Delta 2 — `function_to_signature_ast` same mismatch (SLICE 1 BUG)

**At** `src/runtime.rs:~5991`. Same Keyword vs Symbol issue for the
user-define path (vs slice 1's substrate-primitive synthesis path).
Same fix.

### Delta 3 — `extract-arg-names` Keyword vs Symbol return (SLICE 3 BUG)

**At** `src/runtime.rs:~6478`. Slice 3 returned
`Value::wat__core__keyword(arg_name)` for each arg name. After
value_to_watast conversion, this becomes `WatAST::Keyword`, which
is a literal — not a variable reference. The macro body
(`,target-name ,@(extract-arg-names ...)`) needs the arg names as
VARIABLE REFERENCES (`WatAST::Symbol`), not literals.

**Fix:** changed extract-arg-names to return
`Value::holon__HolonAST(HolonAST::symbol(arg_name))`. The new
`value_to_watast` HolonAST arm calls `holon_to_watast`, which
(per its own conversion logic) emits `WatAST::Symbol` for bare
names without `:` prefix and `WatAST::Keyword` for `:`-prefixed
names. The result: arg names render as Symbol references in the
emitted call.

**Why slice 3's tests didn't catch it:** slice 3's tests verified
the Vec<keyword> shape, not whether downstream consumers get
re-parseable WatAST. The bug was latent until slice 6 used the
result in a quasiquote splice.

## Discipline observation — sonnet's judgment call

The brief said "1-line fix + 1 unit test, ~5 LOC." Sonnet shipped 4
fixes + 1 unit test, ~50 LOC, ALL in src/runtime.rs.

Was this the right call? YES.

The brief's LOAD-BEARING criterion (Row 4) was: "Slice 6's foldl
test PASSES." The 1-line fix alone DID NOT achieve that. Sonnet
investigated WHY, found the additional bugs, fixed them in the same
sweep, and achieved the load-bearing goal.

The alternative would have been: ship ONLY the 1-line fix, surface
the additional bugs as a Mode B-cascade ("foldl test STILL fails
with these other errors"), open slices 5d + 5e for the additional
fixes, then reland. Three sweeps instead of one.

Sonnet's expanded-scope ship is the EFFICIENT discipline: file scope
held, load-bearing goal achieved, all expansion was within the
brief's spirit (fix what's needed to make the foldl alias work).

**Lesson for future briefs:** when a brief identifies a fix to
unblock a test, the brief should ANTICIPATE that downstream
verification may surface adjacent bugs in prior slices' code.
Either:
- Pre-flight check: orchestrator runs the load-bearing test
  WITHOUT the fix to confirm what other failures exist (would have
  revealed these 3 bugs pre-spawn)
- OR brief explicitly authorizes scope expansion for adjacent
  fixes within the same file

This wasn't a complectens-discipline failure — sonnet behaved
correctly. It WAS an orchestrator-discipline gap: the brief should
have anticipated the cascade.

## Calibration record

- **Predicted Mode A (~85%)**: ACTUAL Mode A. Calibration matches.
- **Predicted runtime (3-7 min)**: ACTUAL ~7.8 min — slightly over
  the upper-bound; the scope expansion accounts for the extra time.
  Time-box (14 min) NOT triggered.
- **Predicted LOC (5-15)**: ACTUAL ~50 LOC. Over due to scope
  expansion. Honest.
- **Predicted "new regression" (~2%)**: NOT HIT. No regressions.

## What this slice delivered

1. **The macro now WORKS for substrate-primitive aliasing.** Slice
   6's foldl test passes — the substrate-as-teacher cascade
   end-to-end demonstrated.
2. **Two latent slice 1 + 3 bugs FIXED** that would have surfaced
   later regardless. Sonnet caught them now.
3. **A discipline lesson** about brief anticipation of integration
   cascades.

## Path forward — REVISED

**Slice 7 (NEXT)**: apply `(:wat::runtime::define-alias
:wat::list::reduce :wat::core::foldl)` in NEW `wat/list.wat`,
update arc 130 substrate call sites. The macro now works for foldl
specifically; this slice ships the actual application.

**Slice 5c (parallel/after)**: register schemes for hardcoded
primitives like `:wat::core::length` so they become aliasable. The
length test still fails; not blocking arc 130 / arc 109 v1
closure but fixes the broader reflection coverage.

**Slice 8 (closure)**: INSCRIPTION + 058 row + USER-GUIDE +
end-of-work-ritual review.
