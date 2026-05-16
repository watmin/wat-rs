# Arc 201 Slice 3 BRIEF — `signature-of-fn` primitive

**Phase:** Third slice of arc 201. Slices 1+2 (commits `0706949`, `c9445a4`) shipped structured type-AST emission + general-purpose accessors. Slice 3 mints `signature-of-fn` — the inline-fn reflection primitive D2's `run-threads` macro needs.

**Originating signal:** D2's call form is:
```scheme
(:wat::kernel::run-threads
  (:wat::core::fn
    [logger   <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>
     counter  <- :wat::kernel::ThreadPeer<wat::core::i64,wat::core::i64>
     reporter <- :wat::kernel::ThreadPeer<wat::core::String,wat::core::String>]
    -> :wat::core::String
    ...body...)
  (:app::logger-worker)
  (:app::counter-worker)
  (:app::reporter-worker))
```

The macro receives the coordinator as an INLINE FN AST (not a name). Existing `signature-of` does symbol-table lookup by NAME — wrong shape. New primitive needed: take a fn AST, return its structured signature HolonAST.

## Goal

Mint `:wat::runtime::signature-of-fn fn-ast -> :HolonAST` that:

1. Accepts a WatAST::List headed by `:wat::core::fn`
2. Extracts the param binders + return type from the AST
3. Returns a structured signature HolonAST in the SAME SHAPE that `signature-of-defn` (the renamed existing `signature-of`, post-slice-4) returns
4. Uses slice 1's structured type-AST emission rules (parametric types as Bundle, paths as Atom)

**Output shape (matches function_to_signature_ast's existing output, lifted to HolonAST):**
```
Bundle [
  Atom(":anonymous"),                              ;; or ":fn"; pick consistent with function_to_signature_ast
  Bundle [Symbol(param0-name), <type0-AST>],
  Bundle [Symbol(param1-name), <type1-AST>],
  ...
  Symbol("->"),
  <ret-type-AST>
]
```

Where `<typeN-AST>` is Bundle for parametric/Tuple/Fn types, Atom for path/var (per slice 1 emission rules).

## Required path (NO new substrate types/structs/special-forms)

This slice adds:
- 1 new substrate VERB at `:wat::runtime::signature-of-fn`
- Eval handler that walks the fn-AST and reuses slice 1's `type_expr_to_ast` (or equivalent) for type slots
- Type scheme registration

Reuse what slice 1 + 2 shipped:
- slice 1's `type_expr_to_ast` if the fn-AST's type slots need re-emission via TypeExpr (they shouldn't — the parser produces WatAST type forms directly; signature-of-fn just lifts them via watast_to_holon)
- existing `watast_to_holon` for the WatAST → HolonAST conversion
- existing `function_to_signature_ast` shape as the output structure model

## Implementation hint (sonnet verifies + adjusts)

The fn-AST coming from the parser looks like:
```
WatAST::List [
  WatAST::Keyword(":wat::core::fn"),
  WatAST::Vector [  // param binders
    WatAST::Symbol("logger"),
    WatAST::Keyword("<-"),
    WatAST::List/Keyword(<type-form>),  // structured List for parametric, Keyword for path
    WatAST::Symbol("counter"),
    WatAST::Keyword("<-"),
    WatAST::List/Keyword(<type-form>),
    ...
  ],
  WatAST::Symbol("->"),
  WatAST::List/Keyword(<ret-type-form>),
  ...body forms...
]
```

(Sonnet verifies via parser inspection; the exact shape may differ on details.)

The implementation:
1. Pattern-match the fn-AST head + extract binders Vector + ret-type
2. Walk binders → pair-tuples of (name, type-AST)
3. Build signature WatAST in `function_to_signature_ast`'s output shape (use the existing helper if you can call it; otherwise mirror it)
4. `watast_to_holon` → wrap in `Value::Option(Some(Value::holon__HolonAST(...)))` matching `signature-of`'s return type

Per `feedback_no_new_types`: this is ONE new verb. No new types. No new structs.

## Tests

`tests/wat_arc201_signature_of_fn.rs` (or sonnet picks name):

- `signature_of_fn_extracts_parametric_arg_types` — inline fn with `:ThreadPeer<I,O>` arg; verify Bundle structure in returned signature
- `signature_of_fn_extracts_monomorphic_arg_types` — fn with `:String` and `:i64` args; verify Atom shape
- `signature_of_fn_emits_anonymous_head` — head is `:anonymous` (or whatever existing convention picks)
- `signature_of_fn_extracts_return_type` — Bundle for parametric ret, Atom for path ret
- `signature_of_fn_handles_variadic` — fn with `& (rest :Vector<T>)` binder; verify the variadic-rest binder appears in signature
- `signature_of_fn_errors_on_non_fn_input` — passing a non-fn AST (e.g., a List headed by something else) → TypeMismatch
- `signature_of_fn_composes_with_extract_arg_names` — signature-of-fn output walks correctly with extract-arg-names + Bundle/children + Bundle/first

## Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release -p wat --test wat_arc201_signature_of_fn  # (or chosen name)
cargo test --release -p wat --test wat_arc201_structured_signature_types  # slice 1
cargo test --release -p wat --test wat_arc201_holon_ast_accessors  # slice 2
cargo test --release --workspace --no-fail-fast
```

Workspace baseline: failures ≤ 4 stable + lifeline flake (commit `10ad4dc`).

## STOP triggers (true emergencies — surface, do not paper over)

1. **The fn-AST shape differs from the BRIEF's implementation hint** — surface what you found; the BRIEF's hint is approximate (sonnet verifies)
2. **`function_to_signature_ast` is reusable via direct call** — surface and reuse (don't duplicate logic); if NOT reusable cleanly, mirror its output shape exactly
3. **Variadic handling is non-trivial** — surface; may warrant a follow-up if variadic-rest signature emission has its own quirks (D2's coordinator is non-variadic so this can ship without variadic if blocking)
4. **Naming-related ambiguity** — `signature-of-fn` is the working name; if cleaner alternative surfaces, capture in SCORE
5. **Workspace baseline regresses** — STOP, surface
6. **Any urge to mint a new substrate TYPE / STRUCT / SPECIAL FORM** — STOP. New verb only.
7. **arc 057's existing surface might already serve this** — `:wat::runtime::signature-of` exists for named defns; check if there's anything that ALREADY handles fn-AST input (you'll find: no — but per arc 199 rejection + slice 2 STOP-trigger discipline, ALWAYS check first)

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may launch into `.claude/worktrees/agent-<id>/` — ignore it; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT mint new types/structs/special-forms. New VERB only.
- DO NOT touch slice 1 or slice 2 work (they're done; just reuse).
- DO NOT touch `signature-of` (existing primitive) — slice 4 renames it; this slice ADDS a sibling.
- DO NOT touch arc 117/133 sibling-binding walker.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / past STONE BRIEFs/EXPECTATIONS/SCOREs.
- DO NOT modify INTERSTITIAL-REALIZATIONS.md or arc 201 DESIGN.md (orchestrator owns).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

**Macro dialect (Clojure-style):**
- `~` = unquote
- `~@` = unquote-splicing
- `,` = whitespace literal

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::runtime::signature-of-fn` minted (eval + dispatch + type scheme) | grep + unit test calls the verb |
| B | Output shape matches `signature-of-defn`'s structure (head + arg-pairs + arrow + ret) | composition test passes — extract-arg-names + Bundle/children walk the output correctly |
| C | Parametric type slots emit as Bundle; path slots emit as Atom (slice 1 rules) | unit test asserts Bundle for `:ThreadPeer<I,O>` arg, Atom for `:String` arg |
| D | Errors cleanly on non-fn input | error-case test passes |
| E | Workspace test failure count ≤ baseline (4) | full workspace cargo test failures ≤ baseline + flake variance |

## Honest deltas to capture in SCORE

- Did you reuse `function_to_signature_ast` directly, or mirror its shape?
- Final name picked (vs working `signature-of-fn`)
- Anonymous head choice (`:anonymous` vs `:fn` vs other)
- Variadic-rest handling — shipped or deferred?
- Per arc 199/arc 200 lessons: did you check arc 057/arc 143 surface FIRST? Surface findings (even if "nothing relevant" — confirms the check happened)
- Any naming-related `/gaze` exchanges

## Time-box

45-75 min predicted (smaller than slice 1; reuses machinery). Hard stop 90 min.

## Workspace baseline (commit `10ad4dc`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 stable failures + lifeline flake variance

Post-slice-3 target:
- ≥ baseline + 5-7 new passes (signature-of-fn tests)
- ≤ baseline failures (purely additive)

## On completion

1. Write `docs/arc/2026/05/201-reflection-structured-type-ast/SCORE-SLICE-3.md` per § SCORE methodology + § Honest deltas.
2. Return final summary to orchestrator: rows passed/failed + workspace delta + path to SCORE + reuse decisions + naming choices.

You are launching now. T-minus 0.
