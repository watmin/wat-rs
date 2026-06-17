# BRIEF — Stone 277.1: the `wat-lint` framework + the nested-if-=-ladder rule

## The work (one paragraph)

Build `wat/lint.wat` — the linter framework — plus its first structural rule. A **rule** is
`(form → Vector<Finding>)`; a **Finding** carries `{rule, file, line, col, severity, message, fix?}`
where `fix?` is the optional `(offset, old-len, new-text)` edit (the exact shape `fix.wat`'s
`fix-text-apply` consumes — this is the seam that lets `wat-fix` apply lint findings). `lint-source
(files: Vector<SourceFile>) → Vector<Finding>` runs the form-level rules over every top-level form of
every file (reusing `:wat::deporder::SourceFile` + `:wat::deporder::stdlib-sources` from arc 275).
`lint-stdlib` is the surface: form-level findings over the real stdlib **plus `deporder`'s load-order
folded in as rule-zero** (report-only, no fix). The first form-level rule is the bad form that triggered
the whole arc: **`nested-if-=-ladder`** — `(if (= VAR LIT) true (if (= VAR LIT) true … false))` over ONE
var, all returning `true`: a `HashSet/contains?` membership in disguise — detected, reported, and (if it
lands cleanly) carrying a `fix` that rewrites it to the cleaned-`deporder` shape.

Worked references on disk: `wat/deporder.wat` (SourceFile, `stdlib-sources`, AST-walk, the *cleaned*
`is-def-head?`/`structural?` = the fix's OUTPUT shape), `wat/fix.wat` (the edit shape + `fix-text-apply`
+ how a rule produces `(offset,old-len,new-text)` edits via `ast-span` — `fix-macro-param-types` is the
rule→edits→splice exemplar), `wat-tests/deporder.wat` (the deftest proof shape).

## Read in order (the rooms)

1. **`wat/deporder.wat`** — `SourceFile` record, `stdlib-sources`, the `read-string`→`ast->children`
   top-level-form iteration, `structural?` + the recursive AST walk, and the **cleaned** `is-def-head?`
   (lines ~70-87) and `structural?` (lines ~44-51) — those `HashSet/contains?` forms are *exactly* the
   output your fix should produce. Reuse `:wat::deporder::SourceFile` and `:wat::deporder::stdlib-sources`;
   do NOT redefine them.
2. **`wat/fix.wat`** — the edit engine: `fix-text-apply` (applies `(offset,old-len,new-text)` edits
   right-to-left), `fix-text-offset-of` / `ast-span` usage (how to turn a node's span into an edit
   offset+len), and `fix-macro-param-types` (lines ~432-510) — the canonical rule→edits→splice shape.
   Your `fix?` field is one of these edit tuples; do NOT fork the engine.
3. **`wat/Record.wat`** — `(:wat::Record::def :ns::Name [field <- :Type …])` for the `Finding` record.
4. **`wat-tests/deporder.wat`** — the `deftest` proof shape (pure-form tests, literal fixtures).
5. **`src/stdlib.rs`** — register `wat/lint.wat` as a `WatSource` (near `fix.wat`/`deporder.wat`). It
   defines defmacros/defns (order-free); the `deporder` gate must stay at **0 violations** after.
6. **`tests/probe_arc277_lint_if_ladder.rs`** — the RED gate. Remove its `#[ignore]` once green.

## Implementation sketch (you fill the bodies)

```clojure
;; Finding — a lint result; fix is the optional edit fix-text-apply consumes.
(:wat::Record::def :wat::lint::Finding
  [rule     <- :wat::core::String
   file     <- :wat::core::String
   line     <- :wat::core::i64
   col      <- :wat::core::i64
   severity <- :wat::core::String            ;; "error" | "warn" | "info"  (L1/L2/L3)
   message  <- :wat::core::String
   fix      <- :wat::core::Option<(wat::core::i64,wat::core::i64,wat::core::String)>])  ;; (offset,old-len,new-text)

;; A RULE: (form -> Vector<Finding>). Detect (recursively) + how-to-fix in one fn.
(:wat::core::defn :wat::lint::rule-nested-if-=-ladder
  [form <- :wat::WatAST  file <- :wat::core::String]
  -> :wat::core::Vector<wat::lint::Finding>  …)

;; lint-source: run the form-level rules over every top-level form of every file.
(:wat::core::defn :wat::lint::lint-source
  [files <- :wat::core::Vector<wat::deporder::SourceFile>]
  -> :wat::core::Vector<wat::lint::Finding>  …)

;; lint-stdlib: the surface — form-level findings + deporder load-order folded as rule-zero.
(:wat::core::defn :wat::lint::lint-stdlib [] -> :wat::core::Vector<wat::lint::Finding>
  ;; (lint-source (stdlib-sources)) ++ (deporder violations mapped to report-only Findings)
  …)
```

### The nested-if-=-ladder rule — detection + fix

- **Detect** (recursive walk, mirror `deporder`/`fix.wat`): a List whose head ast-name is
  `:wat::core::if`, child[1] is `(:wat::core::= VAR LIT)` (VAR a symbol/expr, LIT a literal), child[2] is
  the literal `true`, and child[3] (the else) is **either** `false`/another value **or** another matching
  `if` over the **same VAR**. Walk the else-chain collecting `(VAR, LIT)`; it is a ladder when the same
  VAR is compared against **≥ 3** literals all returning `true`. (≥3 matches the probe fixture; tune if a
  real case argues otherwise, and say so.) Recurse into all forms so a nested ladder is found.
- **Report**: emit a `Finding` — `rule "nested-if-=-ladder"`, `severity "warn"`, a message naming the var
  + the literal count + the cure (`HashSet/contains?`), `line`/`col` from `ast-span`.
- **Fix** (the contract-proving part): build the replacement form — the cleaned-`deporder` shape,
  `(:wat::core::contains? (:wat::core::HashSet :T LIT…) VAR)` (or the `let`-bound variant `deporder` uses)
  — `write-forms` it to text, and emit the edit `(ladder-span-offset, ladder-span-len, new-text)` in the
  `fix` field. (The output need not be perfectly formatted — `wat-fmt` is 277.3; comment-faithful
  span-splice is what matters here.) **If constructing this edit cannot land cleanly in this stone, see
  STOP-1.**

## Your proof (wat deftests in `wat-tests/lint.wat` — match `deporder.wat`'s shape)

1. **Detects the ladder.** `lint-source` on a `SourceFile` whose body is a 3-deep `if`-`=`-ladder over
   one var → **≥1 finding**, `rule == "nested-if-=-ladder"`.
2. **No false positive.** A clean file (a `cond`, or a single `if`, or two ifs over *different* vars) →
   **0 findings** from this rule.
3. **The fix applies (end-to-end lint→fix).** Take the finding's `fix` edit, run it through
   `fix.wat`'s `fix-text-apply` on the original source → the result contains `HashSet`/`contains?` and no
   longer trips the rule (re-lint → 0). This proves the lint→fix seam on one rule. *(If STOP-1, this test
   is deferred with the fix to 277.1b — say so.)*
4. **`lint-stdlib` runs + rule-zero present.** `(:wat::lint::lint-stdlib)` evaluates to a `Vector` and
   includes `deporder`'s load-order as rule-zero (currently 0 violations since 275 fixed them — assert it
   runs and returns a Vector, and that a fabricated out-of-order input would surface a rule-zero finding).

## STOP triggers (halt + report; do NOT improvise)

1. **STOP-1** — if building the `fix` edit (replacement AST → `write-forms` → span offset/len) cannot land
   cleanly, ship the rule **report-only** (`fix` = `None`), keep proofs 1/2/4, and STOP to report — we
   split the auto-fix into 277.1b. The framework + detection + findings is the must-have; the fix proves
   the contract and is the stretch.
2. **STOP-2** — if the rule-registry shape won't compose (heterogeneous rule signatures, the `Option`
   tuple field won't type), STOP and report the exact checker error; do not loosen to an untyped map.
3. **STOP-3** — after registering `wat/lint.wat`, the `deporder` gate (`tests/test_stdlib_load_order.rs`)
   must stay **0 violations** and the build must freeze; if either breaks, STOP and report.

## Expectations (scorecard — fill on your own re-run)

| what | command | expected |
|---|---|---|
| RED gate goes green | `cargo test --release -p wat --test probe_arc277_lint_if_ladder` (after un-ignore) | pass |
| lint deftests green | `cargo test --release --test test` (the new `deftest_wat_tests_lint_*`) | all green |
| deporder gate holds | `cargo test --release --test test_stdlib_load_order` | 0 violations |
| no regression | `cargo test --release -p wat --lib` | 929/36 (baseline, zero new) |

## Blast radius

- NEW: `wat/lint.wat`, `wat-tests/lint.wat`.
- EDIT: `src/stdlib.rs` (one `WatSource` entry for `wat/lint.wat`); un-ignore
  `tests/probe_arc277_lint_if_ladder.rs`.
- REUSE (do not fork): `:wat::deporder::SourceFile` / `stdlib-sources`; `fix.wat`'s `fix-text-apply` +
  edit shape. Nothing else.

## Discipline

- **Do NOT spawn sub-agents.** Single executor. Do NOT commit (orchestrator weighs + commits).
- Build green; a broken `lint.wat` fails the stdlib freeze (instant feedback).
- Typed records only (the `Finding`), no loose maps. One opinionated rule — no config, no options.
- Return: what you built, your own test results (commands + counts), whether the fix landed or you hit
  STOP-1 (report-only), the `deporder` gate result, line counts, any STOP hit.
