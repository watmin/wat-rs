# BRIEF — Arc 210 Slice 1: `:restricted-to` keyword tag on `def-restricted` + `defn-restricted`

**Predecessors:** Arc 210 DESIGN inscribed; substrate pre-flight greps complete (see DESIGN § "Substrate touchpoints" + this BRIEF's "Substrate facts verified" section).

**This is a SINGLE-SLICE ARC.** One atomic substrate-primitive update + sugar update + test sweep. Pure-additive parser extension; no semantic change to whitelist enforcement.

## Goal

Migrate both `:wat::core::def-restricted` (substrate primitive) and `:wat::core::defn-restricted` (defmacro sugar) from positional `[prefix-vec]` to keyword-tagged `:restricted-to [prefix-vec]`.

**Before:**
```scheme
(:wat::core::def-restricted :name [:wat::kernel::] <expr>)
(:wat::core::defn-restricted :name [:wat::kernel::] [p <- :T] -> :Ret <body>)
```

**After:**
```scheme
(:wat::core::def-restricted :name :restricted-to [:wat::kernel::] <expr>)
(:wat::core::defn-restricted :name :restricted-to [:wat::kernel::] [p <- :T] -> :Ret <body>)
```

## Why this slice exists

Per arc 210 DESIGN § "The crack arc 210 closes": every other substrate keyword-tagged form (defservice's `:state`/`:admin`/`:user`; defmacro typed-AST params) names sections by keyword. `def-restricted`'s positional encoding was the lone holdout. Arc 209 surface convergence (collapsed shape) made the inconsistency visible.

Per arc 210 DESIGN § "Precursor sequencing": arc 209 Stone A drafts after arc 210 closes, so defservice's generated `defn-restricted` forms emit the new shape from day 1.

## Substrate facts verified (pre-flight; cite file:line)

| Touchpoint | File:line | Current shape | After |
|---|---|---|---|
| Parser (load-bearing) | `src/check.rs:7490` `infer_def_restricted` | `if args.len() != 3` — expects (name, prefix-vec, expr) | `if args.len() != 4` — expects (name, `:restricted-to` literal, prefix-vec, expr); args[1] keyword-validated |
| Runtime dispatch | `src/runtime.rs:2287` (eval handler arm) | Mirrors check.rs's 3-arg shape | Mirrors check.rs's 4-arg shape |
| Defmacro pattern | `wat/core.wat:221-227` `:wat::core::defn-restricted` | `(name) (prefixes) & rest` | `(name) (restricted-to-keyword) (prefixes) & rest` — middle binder must be literal `:restricted-to` keyword at expand time |
| Head-keyword matches | `src/check.rs:4657`, `src/check.rs:7699`, `src/check.rs:7826`, `src/freeze.rs:1419`, `src/freeze.rs:1457`, `src/special_forms.rs:128`, `src/runtime.rs:2854`, `src/runtime.rs:4132` | Match `:wat::core::def-restricted` head only | NO CHANGE — these just check the head; don't touch the args |
| Wat-side consumers | `wat/runtime.wat`, `wat/test.wat`, `wat/kernel/*.wat`, `wat/services/*.wat` (grep `defn-restricted\|def-restricted`) | Zero hits outside `wat/core.wat` | NO CHANGE — verified by pre-flight grep |
| Test sites | `tests/wat_arc198_def_restricted.rs` | ~10-12 sites use `def-restricted [prefix-vec]` or `defn-restricted [prefix-vec]` positionally | Migrate each to `:restricted-to [prefix-vec]` form |

## Implementation tasks (atomic; single commit)

### Task 1 — `src/check.rs:7490` `infer_def_restricted` parser update

Current:
```rust
if args.len() != 3 { /* error: "expected (def-restricted :name [prefix-vec] expr); got N args" */ }
let name = match &args[0] { WatAST::Keyword(k, _) => k.clone(), ... };
match &args[1] { WatAST::Vector(prefix_items, _) => { ... validate keywords ... } ... }
let expr_ty = infer(&args[2], ...);
```

After:
```rust
if args.len() != 4 { /* error: "expected (def-restricted :name :restricted-to [prefix-vec] expr); got N args" */ }
let name = match &args[0] { WatAST::Keyword(k, _) => k.clone(), ... };
// NEW: args[1] must be literal :restricted-to keyword
match &args[1] {
    WatAST::Keyword(k, _) if k == ":restricted-to" => {}
    other => { /* error: "second arg must be `:restricted-to` keyword tag; got {other}" */ }
}
match &args[2] { WatAST::Vector(prefix_items, _) => { ... validate keywords ... } ... }
let expr_ty = infer(&args[3], ...);
```

Same diagnostic messages updated to mention `:restricted-to`. Same redef gating logic; same MalformedForm error variants.

### Task 2 — `src/runtime.rs:2287` runtime dispatch arm update

Mirror the same shape change at the runtime arm. Expected to be a small parallel update (same arg-index shift; same `:restricted-to` literal validation).

### Task 3 — `wat/core.wat:221-227` defmacro pattern update

Current:
```scheme
(:wat::core::defmacro
  (:wat::core::defn-restricted
    (name :AST<wat::core::nil>)
    (prefixes :AST<wat::core::nil>)
    & (rest :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def-restricted ~name ~prefixes (:wat::core::fn ~@rest)))
```

After:
```scheme
(:wat::core::defmacro
  (:wat::core::defn-restricted
    (name :AST<wat::core::nil>)
    (restricted-to-keyword :AST<wat::core::nil>)    ;; must be literal :restricted-to at call site
    (prefixes :AST<wat::core::nil>)
    & (rest :AST<wat::core::Vector<wat::WatAST>>)
    -> :AST<wat::core::nil>)
  `(:wat::core::def-restricted ~name ~restricted-to-keyword ~prefixes (:wat::core::fn ~@rest)))
```

The defmacro doesn't need to VALIDATE the keyword literal — the substrate primitive's parser does that. defmacro just splices the keyword through. The expansion produces the correct 4-arg substrate form.

Update the comment at lines 208-220 to reflect the new shape.

### Task 4 — `tests/wat_arc198_def_restricted.rs` test migration

~10-12 sites. Mechanical sweep: each `[prefix-vec]` positional arg becomes `:restricted-to [prefix-vec]`. No semantic changes to tests; same expected behavior; same error message assertions (updated for the new diagnostic mentioning `:restricted-to`).

Specifically search for:
- `(:wat::core::def-restricted :name [` → add `:restricted-to ` before `[`
- `(:wat::core::defn-restricted :name [` → same pattern
- Error message assertions: if any test asserts on the parser's error message, update the assertion to expect the new diagnostic phrasing (mentioning `:restricted-to`).

### Task 5 — workspace `cargo test --release` green

After all 4 tasks, the workspace test suite must be green. Pre-existing failures (if any from the existing baseline) acceptable; new failures introduced by this slice must be ZERO.

## HARD constraints

- DO NOT change whitelist enforcement semantics. Same matching rules (namespace-prefix-with-trailing-`::` vs exact-FQDN-without). Same walker.
- DO NOT add a backwards-compat path that accepts both old (3-arg) and new (4-arg) shapes. Break-and-fix: old form rejected with diagnostic pointing at new form.
- DO NOT touch any consumer outside `tests/wat_arc198_def_restricted.rs` — pre-flight grep confirmed zero other consumers. If sonnet finds others, STOP and surface.
- DO NOT touch defmacro signature parsing in substrate — defmacro accepts positional + variadic per existing capability; no new pattern-matching needed.
- DO NOT extend keyword-tag support to OTHER substrate forms. Arc 210 is `def-restricted` + `defn-restricted` ONLY.
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/`.
- DO NOT commit. Orchestrator commits the slice atomically after SCORE.

## STOP triggers

1. **Pre-flight grep finds wat-side consumers** (other than `wat/core.wat:221-227`). DESIGN claimed zero; if any exist, surface immediately + orchestrator decides scope expansion.
2. **`src/runtime.rs:2287` runtime dispatch arm needs deeper changes** beyond the parallel 4-arg parsing (e.g., new mutation tracking). Substrate-internal complexity unexpected → surface.
3. **Defmacro can't splice a keyword positional binder** for some reason. (Should work per arc 198 + arc 150 variadic patterns, but verify.) If blocked, surface + orchestrator decides whether to use `& rest` + manual destructure pattern.
4. **Test assertion update reveals deeper diagnostic shape** that affects more than just the test file. E.g., if `MalformedForm` is rendered differently elsewhere.
5. **More than 15 test sites** in `tests/wat_arc198_def_restricted.rs` — DESIGN estimated ~10-12; if substantially more, surface but proceed (mechanical scope expansion is acceptable).

## SCORE methodology

3 rows; atomic YES/NO:

| Row | Evidence |
|---|---|
| A — Substrate parser + runtime updated to accept 4-arg shape with `:restricted-to` keyword tag | `src/check.rs:7490` infer_def_restricted reflects new shape; `src/runtime.rs:2287` mirrors it; new arity-error + new keyword-tag-error diagnostics in place |
| B — Defmacro `defn-restricted` updated to splice `:restricted-to` keyword through | `wat/core.wat:221-227` reflects new shape; expansion produces correct 4-arg substrate form |
| C — Test sweep complete; workspace cargo test green | All ~10-12 sites in `tests/wat_arc198_def_restricted.rs` migrated; workspace test count = same as baseline ± ~0 (no new failures) |

## Time-box

Predicted 45-75 min. Hard stop 90 min. Pure-additive parser extension; bounded scope; verified pre-flight.

## On completion

Return summary: how many test sites migrated; any STOP triggers fired; any honest deltas vs DESIGN. Workspace cargo-test result. Orchestrator scores + atomically commits + drafts arc 210 slice 2 (closure paperwork) next.

You are launching now. T-minus 0.
