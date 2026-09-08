## CONFERRE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> Status for every row lives in `FINDINGS.md`, and nowhere else.

## SCOPE

Swept all 25 target files per the cast (20 Rust files, 5 wat files; confirmed line counts match: 21,523 Rust + 2,363 wat = 23,886).

Commands run:
- `grep -rn 'Mirrors ' <20 .rs files>` — found **7 distinct matches**, not "6 + 1 continuation" as the cast described. My 7: `eval_test.rs:12`, `compiled_cond.rs:136`, `compiled_cond.rs:1399`, `validate/mod.rs:708`, `validate/mod.rs:757`, `vocabulary.rs:1291`, and **`expr_ir/eval.rs:1191`** — this last one is NOT among the 6 the cast quoted. I checked each of the 6 quoted sites for a nearby second "Mirrors" line (a true continuation) and found none — each is a single isolated occurrence. So the cast's list and mine diverge by one: I have a genuine 7th claim they didn't name.
- `grep -n "mirrors\|Mirrors\|matches exactly\|same as the\|identical to"` over the 5 wat files — found one more: `wat/rete/compile.wat:1076`.
- `grep -rn "rune:conferre"` over all 25 files — **zero hits**.
- `grep -rln` for cross-references to the 5 wat filenames inside the 20 Rust files, then read the clause.rs one (a foundational claim about `compile-condition` topology) and verified it against `wat/rete/compile.wat`.

All bodies below were read this session; each verdict is grounded in both coordinates.

## The 7 (+1) `Mirrors` claims, adjudicated

1. **`src/rete/eval_test.rs:12`** (`eval_rhs_expr` mirrors `build_test_env`, `eval_test.rs:41`) — TRUE, trivially: `eval_rhs_expr` literally *calls* `build_test_env(bindings, &Environment::new())` at `eval_test.rs:44`. Not a parallel reimplementation, so no drift is possible.

2. **`src/rete/compiled_cond.rs:136`** (`Op::Not` mirrors `eval_clause`'s `Not`, `src/rete/matcher.rs:764-773`) — TRUE. `matcher.rs` `Not` arm evaluates the sub-clause against a **clone**, discards it, and returns the original `bindings` regardless of outcome. `compiled_cond.rs:1372-1375` clones `slots`, runs `exec_ops` against the clone, returns `!result`, discards the clone. Both discard the negated branch's binds identically.

3. **`src/rete/compiled_cond.rs:1399`** (`eval_cmp` mirrors `eval_clause`'s `Constraint` arm, `matcher.rs:711-736`) — TRUE at the behavioral level. `matcher.rs` uses `compare_values(&a,&b)?` inside the match (a `None` short-circuits `eval_clause` itself to `None`); `compiled_cond.rs`'s `eval_cmp` uses `matches!(compare_values(a,b), Some(...))` (a `None` becomes `false`). Different Rust mechanism, but for `Lt/Gt/Le/Ge` both paths produce "clause fails" on an incompatible-type comparison — no observable divergence.

4. **`src/rete/validate/mod.rs:708`** (`rhs_operand_can_never_resolve` mirrors `resolve_operand`'s accepted set, `matcher.rs:931-966`, "minus the `?var` case") — TRUE for the runtime path it actually gates: `resolve_rhs_value` (`eval_insert.rs:285-291`) calls `resolve_operand(arg, &[], &[], bindings, None)` — `sym` hardcoded to `None` — so `resolve_operand`'s `Keyword` arm's `or_else(|| sym.map(keyword_value))` fallback is dead at this call site and a Keyword really can never resolve, matching the claim. Confirmed correct, including the separately-documented Arc 278 Stone B widening for `List` (explicitly flagged as a deviation from plain `resolve_operand`, not silently glossed — the honest shape).
   - **Minor L2**: the prose "a literal resolves" (line 709) overstates the code's own set. `resolve_operand`'s literal path (`ast_literal_value`, `matcher.rs:977-985`) only accepts `IntLit/FloatLit/BoolLit/StringLit` — `RationalLit`/`BigIntLit`/`NilLit` fall through to `None`. `rhs_operand_can_never_resolve`'s own `matches!` list (line 720-725) correctly omits Rational/BigInt/Nil, so the **code** is right and matches `resolve_operand` exactly; only the doc's generic word "literal" glosses over this. The sibling user-facing error text at `validate/mod.rs:748` ("an integer / float / boolean / string literal") is precise where the doc comment above the function is not. No behavioral bug — a reader trusting the loose doc phrasing could be surprised that a Rational literal in `:then` gets flagged, but the validator's actual behavior is correct.

5. **`src/rete/validate/mod.rs:757`** (`walk_nested_constructors` mirrors `dispatch_keyword_head_value`/`eval_kwargs_construct`, `src/runtime.rs:5360`/`18872`) — TRUE on spot-check. The `match`-arm sub-claim ("mirrors `purity.rs`'s match arm... exactly, including its one indirection through `resolve_core_name`") checked byte-for-byte against `purity.rs:1313-1334`: both walk `scrutinee = items[1]`, arms = `items[2..]`, skip element 0 of each arm-List, check `1..` — identical shape, identical `resolve_core_name` indirection. The one difference (validate's walker silently skips a malformed non-List arm rather than raising) is explicitly documented at the deviation ("shape is not this walker's diagnostic"). Traced `eval_kwargs_construct` far enough to confirm nested field values are plain unevaluated `WatAST` handed to `construct_aggregate`/`eval_inner`, consistent with "walks every argument... just one `eval_inner` deeper." This function carries its own extensive self-audit history (D5/D10/D11, strike-nested-wall) — no new gap found.

6. **`src/rete/vocabulary.rs:1291`** (f64 fallback quartet "mirrors the i64 fallback quartet's shape exactly, but the mechanism... is DIFFERENT") — TRUE, and self-qualified correctly. Read both quartets (`vocabulary.rs:296-345` for i64, `1290-1330` for f64): identical `OpClass::Fallback`, identical `params`/`ret` shape with `I64`→`F64` swapped, identical `meta`. The claim's own caveat (i64 fails by raising; f64 fails via NaN/±Inf) is accurate per the surrounding comment and matches `purity.rs`'s stated `total: false` reasoning for the f64 core ops. No divergence.

7. **`src/rete/expr_ir/eval.rs:1191`** (TupleNew mirrors `eval_tuple_ctor`, `src/runtime.rs:12016`) — **not in the cast's list of 6** — TRUE. Both raise the identical `MalformedForm` with the identical reason string ("tuple must have at least one element; the 0-tuple is :() (Unit)") on zero args, and both build `Value::Tuple` from 1+ args. Exact match.

8. **Bonus, wat-side: `wat/rete/compile.wat:1076`** (`compile-rule`'s RHS fence "mirrors how `where`/accumulate are fenced inline during the LHS fold") — checked against `wat/rete/compile.wat:364-650` (`compile-condition`, which mints `TestNode`/`NegationNode`/`ExistsNode`/`AccumulateNode`). The claim is honest about the difference in its own next clause ("this is the RHS's own... pass") — RHS fencing is a separate up-front `foldl` over `rhs` before the LHS fold runs, not literally the same inline mechanism; the "mirrors" word applies to purpose/timing (freeze-time-only), which the sentence itself clarifies. Not a divergence.

Also spot-checked the adjacent (lowercase, ungrepped) `mirrors` claim at `compiled_cond.rs:132` (`Op::Or` vs `eval_clause`'s `Or`, `matcher.rs:747-762`) since I already had both bodies open: TRUE — both discard the branch's own bindings and return the pre-`or` state on success.

And `src/rete/clause.rs:25` — its claim that `compile-condition` (`wat/rete/compile.wat`) consumes top-level `Not`/`Exists`/`Where`/`Accumulate` into `NegationNode`/`ExistsNode`/`TestNode`/`AccumulateNode` before alpha-match — confirmed against `compile.wat:364-650`, which mints exactly those four node types for exactly those four wrapper shapes. TRUE.

## Other categories

- **Spec clause with no implementation / implementation with no spec**: none found in the areas read. No `rune:conferre` markers exist anywhere in the 25-file target (grep returned zero), so there is nothing pre-adjudicated to score.
- **Two implementations disagreeing**: the interpreted/compiled pairs checked (Not, Or, Constraint/eval_cmp, TupleNew) all agree.

## CLOSE

**FINDINGS** — 1.

- **L2** (structural mumble, borderline): `src/rete/validate/mod.rs:708-698` — the doc comment's phrase "a literal resolves" is imprecise against its own function body and against `resolve_operand` (`src/rete/matcher.rs:931-985`): only `IntLit/FloatLit/BoolLit/StringLit` resolve; `RationalLit`/`BigIntLit`/`NilLit` do not. The code itself (`matches!` list at `validate/mod.rs:720-725`) and the sibling user-facing error text (`validate/mod.rs:748`, "an integer / float / boolean / string literal") are both already precise — this is a doc-comment wording gap, not a behavioral bug.

7 (or 8, counting the wat-side and the bonus one) `Mirrors`-class claims were read in both bodies and hold true. No L1s found on this target.
