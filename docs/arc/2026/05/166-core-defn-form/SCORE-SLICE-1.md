# Arc 166 slice 1 — SCORE

Sonnet sweep + orchestrator-side substrate fixes shipped 2026-05-08.
Mode A clean: 10/10 arc 166 tests green; workspace 117 OK / 0 FAILED.

Commit: `4bfbab9` — atomic shipment of macro + tests + substrate fixes.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — `wat/core.wat` has the new defmacro section | git diff confirms section comment + defmacro form | ✓ |
| B — Macro expansion shape | post-expansion AST = `(:wat::core::def name (:wat::core::fn sig body))`; verified via test 10 reflection | ✓ |
| C — Test 1 simple defn add(2,3)=5 | PASS | ✓ |
| D — Test 2 recursive defn fact(5)=120 | PASS (after Gap A fix) | ✓ |
| E — Test 3 defn at file-root | PASS | ✓ |
| F — Test 4 defn inside `(:wat::core::do ...)` | PASS | ✓ |
| G — Test 5 defn inside top-level `let` body | PASS | ✓ |
| H — Test 6 defn inside `if` branch is REJECTED | startup_err matches `DefNotTopLevel`; PASS | ✓ |
| I — Test 7 zero-arg defn | PASS (no-arg sig shape `(-> :T)` works first attempt — honest delta D from sonnet) | ✓ |
| J — Test 8 body type-mismatch surfaces | PASS (TypeMismatch / ReturnTypeMismatch surfaces) | ✓ |
| K — Test 9 redef same name forbidden | PASS (after collision-policy refinement — pre-register only if name new) | ✓ |
| L — Test 10 reflection lookup-define resolves | PASS (after Gap B fix) | ✓ |
| M — `cargo test --release --workspace --no-fail-fast` clean | 117 OK / 0 FAILED | ✓ |
| N — Pre-existing tests unchanged | no regressions across the workspace | ✓ |

## Honest deltas (shipped + closed)

### Gap A — Recursive defn (substrate fix, in-scope)

Sonnet's diagnostic: `infer_def` infers RHS BEFORE writing to
`defined_values`, so `(:wat::core::defn :fact ...)`'s body lookup of
`:user::fact` failed `UnresolvedReferences`. `define` avoids this via
`register_defines` pre-registering ALL function names into
`sym.functions` before `check_program` runs.

Orchestrator fix: extended `register_defines` to ALSO recognize
`(:wat::core::def :name (:wat::core::fn sig body))` shape and
pre-register the fn into `sym.functions`. Form stays in `rest` so
`register_runtime_defs` still evaluates the def at freeze time.

New helper: `try_parse_fn_shape_def(form: &WatAST) -> Option<(String,
Arc<Function>)>` in `src/runtime.rs`. Parses the fn signature via the
existing `parse_fn_signature` helper; builds a Function with
`name: Some(name)` and `closed_env: None` (top-level / splice-eligible
defs don't close over local bindings).

**Collision policy refinement.** First implementation pre-registered
unconditionally and emitted `DuplicateDefine` on collision, which
fired a wrong error type for the second `(defn :user::f ...)` case
in test 9 (test expected `DefRedefForbidden` from `infer_def`).
Refined: pre-register ONLY if name is new in `sym.functions`; on
collision, skip silently and let `infer_def`'s redef discipline
(arc 157 slice 1a-ii) own the decision. Type-check-side
`DefRedefForbidden` remains authoritative.

### Gap B — Reflection on def-bound names (substrate fix, in-scope)

Sonnet's diagnostic: `eval_lookup_define` always evaluates the
argument. After Gap A fix, evaluating `:user::add` returns the
`runtime_def_values` entry — a Value::wat__core__fn with
`name: None` (because `eval_fn` doesn't know the def's name).
`name_from_keyword_or_fn` returns None → TypeMismatch fires.

Orchestrator fix: special-cased `eval_lookup_define` to use the
keyword string DIRECTLY when the argument is a literal keyword AST,
bypassing `eval`. Reflection on a literal keyword should resolve to
the keyword's name without depending on the runtime value's `name`
field. Eval path remains the fallback for non-literal callers
(symbols holding fn-values from sym.functions where `name` is set).

### Sonnet's BRIEF deviation (Delta C from sonnet's report)

The BRIEF named `:wat::runtime::lookup-form` for test 10. That's the
Rust-level `pub fn lookup_form` — not a registered wat callable. The
wat-level reflection primitive is `:wat::runtime::lookup-define`.
Sonnet adapted; test names use `lookup_define`. Calibration note for
future BRIEFs: when the BRIEF needs a wat-callable, grep the wat-side
namespace, not the Rust function name.

### Honest delta D from sonnet — zero-arg sig `(-> :T)` works

Sonnet's report flagged that the BRIEF guessed `(-> :T)` for no-args
sig and expected possible failure; the syntax compiled cleanly first
try. PASS on case 7, no fallback needed. Calibration: arc 155's fn
form accepts pure `(-> :T)` no-args sig as expected.

### Honest delta E from sonnet — position-rule propagates correctly

The macro expands BEFORE `check_program` runs (in `expand_all`); the
post-expansion def inside an if-branch fires `DefNotTopLevel`. No
substrate gap. Inheritance through macro expansion works.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 30-60 min upper-band, 120-min hard cap | sonnet ~18 min + orchestrator-side gap fixes ~40 min = ~60 min total | A clean (at upper bound; substrate gaps required closure in same arc) |

The slice 1 prediction assumed a tight macro shipment without
substrate fix needs. Sonnet's 8/10 first-pass + clean diagnosis of
the two gaps was excellent execution. The orchestrator-side substrate
fixes added ~40 min that wasn't in the original BRIEF prediction —
honest scope expansion per FM 11 (don't INSCRIBE arcs with known
unfixed defects). The total fits the predicted upper band.

**Calibration note**: when a slice ships a macro composing primitives,
the BRIEF should anticipate substrate gaps where the primitives
weren't designed for the composition (here: def's redef-pre-eval
ordering and lookup_define's eval-vs-literal arg semantics). Cost
those at ~30 min orchestrator-side per gap.

## Substrate-as-teacher dynamic

This slice mirrored the substrate-as-teacher discipline: sonnet
shipped what was buildable, the failing tests named the gaps, the
gaps clarified the substrate's actual contracts (def's two-phase
register-after-infer ordering; eval_lookup_define's eager-eval
default). The BRIEF's "STOP at first red" let sonnet surface the
gaps cleanly without bridging. Orchestrator-side closure kept the
substrate-as-teacher loop tight: failures became the design
specification.
