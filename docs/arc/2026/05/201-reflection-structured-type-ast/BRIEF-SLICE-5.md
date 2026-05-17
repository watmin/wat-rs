# Arc 201 Slice 5 BRIEF — `extract-arg-types` substrate primitive

**Phase:** Fifth slice of arc 201. Slices 1+2+3+4 (`0706949`, `c9445a4`, `815d597`, `ecc876a`) shipped structured type-AST emission + Bundle accessors + `signature-of-fn` + `signature-of`/`signature-of-defn` rename. Slice 5 mints the type-extraction sibling of `extract-arg-names`.

**Originating signal:** D2's `run-threads` macro (arc 170) needs per-arg type extraction. The composition is:
1. `signature-of-fn coordinator` → structured signature HolonAST (slice 3)
2. `extract-arg-names sig` → Vector<keyword> of param names (arc 143 slice 3)
3. `extract-arg-types sig` → Vector<HolonAST> of structured arg type-ASTs ← THIS SLICE
4. For each arg, `Bundle/children type-ast` → unpack `:ThreadPeer<I,O>` into [Atom(:ThreadPeer), I-ast, O-ast]

`extract-arg-names` exists; its type-direction sibling doesn't. Slice 5 closes the asymmetric pair.

## Goal

Mint `:wat::runtime::extract-arg-types sig -> :wat::core::Vector<wat::holon::HolonAST>` as a substrate primitive that:

1. Accepts a signature HolonAST (the shape `signature-of-defn` and `signature-of-fn` return)
2. Walks the signature head looking for arg-pair Bundles (skipping head + arrow + ret)
3. For each arg-pair, extracts pair[1] (the structured type AST emitted per slice 1 rules)
4. Returns the collected type-ASTs as a Vector

Direct mirror of `eval_extract_arg_names` (`src/runtime.rs:10165`); same walker shape; one-character difference (pair[0] → pair[1]); different return type (Vector<keyword> → Vector<HolonAST>).

## Required path (NO new substrate types/structs/special-forms)

This slice adds:
- 1 new substrate VERB at `:wat::runtime::extract-arg-types`
- Eval handler `eval_extract_arg_types` (mirror of `eval_extract_arg_names`)
- Type-scheme registration (mirror of `extract-arg-names` registration)
- Dispatch arm
- 2-4 unit tests demonstrating the verb on parametric + monomorphic signatures

Reuse what's shipped:
- The signature HolonAST shape from slices 1 + 3 (`Bundle [head, pair1, pair2, ..., :->, ret]`)
- Slice 1's structured type-AST emission rules (parametric → Bundle, path → Atom)
- The walker logic in `eval_extract_arg_names` (head/arrow/ret skipping) — copy verbatim, change pair extraction

Per `feedback_no_new_types`: ONE new verb. No new types. No new structs.

## Implementation hint (sonnet verifies + adjusts)

**Reference precedent:**
- `eval_extract_arg_names` at `src/runtime.rs:10165` — full implementation; mirror this
- Type-scheme registration at `src/check.rs:14385+` — mirror this
- Dispatch arm at `src/runtime.rs:4051` (`:wat::runtime::extract-arg-names` → `eval_extract_arg_names`) — add a sibling arm

The shape:

```rust
fn eval_extract_arg_types(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    const OP: &str = ":wat::runtime::extract-arg-types";
    // ... mirror eval_extract_arg_names arity check + sig extraction ...
    // walker: for each child of sig's Bundle/children:
    //   - skip head (first child)
    //   - skip arrow (Symbol("->"))
    //   - skip ret (after arrow)
    //   - for arg-pair (a Bundle with [name, type]):
    //       extract type from pair[1] (NOT pair[0] like extract-arg-names)
    //       push to result vec
    // return Value::vec(result)
}
```

The walker logic is IDENTICAL to `eval_extract_arg_names`'s walker — only the per-pair extraction differs. Likely the cleanest refactor is to factor out the shared walker and parameterize on which slot to extract; OR keep them as parallel near-identical handlers per `feedback_simple_is_uniform_composition` (uniform near-duplicate IS simple; abstraction over a 2-call pair is premature). Sonnet picks based on which reads cleaner; both are honest.

**Return type:** `Vec<Value>` of `Value::wat__holon__HolonAST(_)` per type — same lifting pattern signature-of-fn uses to wrap each emitted type.

## Tests

`tests/wat_arc201_extract_arg_types.rs` (sonnet picks final name):

1. `extract_arg_types_returns_atoms_for_monomorphic_args` — fn with `:String` and `:i64` args → Vector contains two atomic Symbols (`:wat::core::String`, `:wat::core::i64`)
2. `extract_arg_types_returns_bundles_for_parametric_args` — fn with `:Vector<wat::core::i64>` arg → Vector contains one Bundle `[Atom(:wat::core::Vector), Atom(:wat::core::i64)]`
3. `extract_arg_types_arity_matches_extract_arg_names` — for the same fn signature, `extract-arg-types` and `extract-arg-names` return Vectors of identical length (the per-arg correspondence is structural)
4. `extract_arg_types_composes_with_bundle_children_on_parametric` — for `:ThreadPeer<I,O>` arg, calling `Bundle/children` on the extracted type-AST returns `[Atom(:ThreadPeer), Atom(:I), Atom(:O)]` (proves the D2 algorithm chain works)
5. `extract_arg_types_errors_on_non_signature_input` — passing a non-Bundle (e.g., a bare Atom or wrong-shape HolonAST) → TypeMismatch with OP tag

## Build + test

```bash
cd /home/watmin/work/holon/wat-rs
cargo build --release --workspace --tests
cargo test --release -p wat --test wat_arc201_extract_arg_types  # (or chosen name)
cargo test --release --workspace --no-fail-fast
```

Workspace baseline at commit `c47c601`: 2323 passed / 3 failed (lifeline flake may flap; 3 stable failures: deftest_wat_tests_tmp_totally_bogus + startup_error_bubbles_up_as_exit_3 + t6_spawn_process_factory_with_capture_round_trips). Post-slice target: pass count ≥ 2323 + 4-5 (new tests); fail count ≤ 4.

## STOP triggers (true emergencies — surface, do not paper over)

1. **`eval_extract_arg_names` shape differs from BRIEF's description** — surface what you found. The BRIEF assumes a walker that handles head/arrow/ret skipping; if the actual impl uses a different mechanism (e.g., direct positional indexing), mirror what's actually there.
2. **The pair Bundle's structure has more than 2 children** — if pair-Bundles are `[name, type]` (2 children) as expected, extract pair[1] directly. If the structure differs (e.g., `[name, separator, type]` with 3 children), update extraction logic accordingly. Surface what the actual shape is.
3. **Return-type lifting fails** — wrapping `Vec<HolonAST>` as `Value::vec(Vec<Value::wat__holon__HolonAST>)` may need specific helpers. Check how `Bundle/children` (slice 2) builds its Vector return; mirror that pattern.
4. **The walker mutates or owns the HolonAST in ways that prevent direct extraction of pair[1]** — if extraction would require deep cloning, do it; surface the cost.
5. **Workspace baseline regresses** — fail count > 4 or pass count drops materially. STOP, surface diff.
6. **Any urge to mint a new substrate TYPE / STRUCT / SPECIAL FORM** — STOP. New verb only.
7. **Wat-side composition temptation** — STOP. Q2 was settled substrate-side per DESIGN.md § Slice 5; do not implement this as a wat-level defn.

## HARD constraints

- DO NOT commit. Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. Harness may report `.claude/worktrees/agent-<id>/` paths — ignore; operate on the real repo per `docs/COMPACTION-AMNESIA-RECOVERY.md` § 7-bis.
- DO NOT modify arc 201 DESIGN.md (orchestrator owns).
- DO NOT touch slices 1, 2, 3, or 4 work (they're done; just reuse).
- DO NOT touch arc 143's `extract-arg-names` — slice 5 ADDS a sibling.
- DO NOT touch arc 202's walker work or run-hermetic-driver.
- DO NOT touch historical artifacts (past INSCRIPTIONs / SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / past STONE BRIEFs/EXPECTATIONS/SCOREs).
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks.

## SCORE methodology

5 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | `:wat::runtime::extract-arg-types` minted (eval + dispatch + type-scheme registration) | grep finds new entries; unit tests call the verb successfully |
| B | Monomorphic arg types extracted as atomic Symbols | `extract_arg_types_returns_atoms_for_monomorphic_args` passes |
| C | Parametric arg types extracted as Bundles per slice 1 emission rules | `extract_arg_types_returns_bundles_for_parametric_args` passes |
| D | Composes with `Bundle/children` for D2 algorithm chain | `extract_arg_types_composes_with_bundle_children_on_parametric` passes |
| E | Workspace test failure count ≤ baseline (3 stable + 1 lifeline flake variance) | full workspace cargo test failures ≤ baseline |

## Honest deltas to capture in SCORE

- Did you keep `eval_extract_arg_types` parallel to `eval_extract_arg_names` (near-duplicate), or factor out a shared walker? Why?
- Final return-type lifting pattern (Vec<Value> shape; how each type-AST gets wrapped)
- Per arc 199/200/201 lessons: did you check arc 057/143 surface FIRST? Surface findings (even if "nothing relevant" — confirms the check happened)
- Edge cases the walker handles differently from extract-arg-names (e.g., variadic-rest binders, if `eval_extract_arg_names` handles them — sonnet inspects)
- Naming-related `/gaze` exchanges (working name `extract-arg-types` — confirm or refine)

## Time-box

30-60 min predicted. Hard stop 90 min.

## On completion

1. Write `docs/arc/2026/05/201-reflection-structured-type-ast/SCORE-SLICE-5.md` per § SCORE methodology + § Honest deltas.
2. Return final summary to orchestrator: rows passed/failed + workspace baseline delta + reuse decisions (parallel-handler vs factored walker) + any honest deltas surfaced.

You are launching now. T-minus 0.
