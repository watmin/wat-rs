## SOLVERE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> HTML entities (`&lt;` `&gt;` `&amp;`) are artifacts of the agent's own output encoding.

## SCOPE

**Files swept:** all 25 files named in the cast (20 Rust, 5 wat).

**Depth of read, by file:** Full or near-full — `clause.rs` (309 lines incl. `ReteClauseShape`/`CmpKind`/`ConstraintSpelling`/`classify_constraint_head`), `mod.rs`, `eval_insert.rs`, `matcher.rs` (eval_clause/eval_clauses/CmpKind arm/value_to_ast_literal/rune, 643-1025), `compiled_cond.rs` (header, compile_one dispatch, exec_op, eval_cmp/eval_cmp_operand), `compiled_rhs.rs` (module header + `RhsOp`/`CompiledRhs`), `export.rs` (full function inventory + full read of `pack_pat`/`unpack_pat`/`pack_expr`/`unpack_expr`), `where_tree.rs` (range_holds/constraint_kind/is_eq_op/range_kind_of/flip_range), `step_payload.rs`, `wat/rete/compile.wat` (inventory + full read of `compile-rule`/`compile-query`/`compile`/`compile-all`). Targeted — `validate/mod.rs`, `validate/typing.rs`, `purity.rs` (`classify_expr` dispatch, ~170 lines). Grep-only inventory — `alpha_tree.rs`, `collect.rs`, `eval_test.rs`, `vocabulary.rs`, `reachability.rs`, `expr_ir/{eval,mod}.rs`, `validate/error.rs`, `wat/rete.wat`, `wat/rete/{syntax,acc,factbag}.wat`.

**How I enumerated the walkers over the grammar:** `grep -rn "classify_rete_clause" src/rete/` to find every consumer of the ONE-DOOR clause classifier (matcher.rs, compiled_cond.rs, alpha_tree.rs, step_payload.rs, validate/{mod,typing}.rs, kernel/{stratify,arm}.rs) — confirming **that axis is already unified**. Then, per the cast's hint, I grepped `CmpKind::(Eq|Lt|Gt|Le|Ge|NotEq)` across every in-scope file to find every hand-written "what does this comparison mean" table, and `grep -n "impl CmpKind|fn holds|fn eval_cmp|fn cmp_holds"` to check whether a shared evaluator already exists (**it does not**).

## Finding 1 — `CmpKind`'s "does this comparison hold" table is hand-written twice, with no shared function

- **Sites:** `src/rete/matcher.rs:733-741` (`eval_clause`'s `Constraint` arm — interpreted path); `src/rete/compiled_cond.rs:1403-1412` (`eval_cmp` — compiled path).
- **The braid:** both independently encode the same six-way mapping — `Eq→a==b`, `NotEq→a!=b`, `Lt/Gt/Le/Ge→compare_values(...)` against an `Ordering` — as a hand-rolled `match CmpKind`. `clause.rs:119-126` defines `CmpKind` but implements no method on it; there is no `CmpKind::holds` or free `cmp_holds`. I grepped `impl CmpKind|fn holds|fn eval_cmp|fn cmp_holds` across the tree: only `compiled_cond.rs`'s private `eval_cmp` exists — matcher.rs's copy has no name at all, it is inline in the match arm.
- **This is exactly the shape the cast asked me to hunt.** `compiled_cond.rs:1399-1402`'s doc already reasons about drift for *ordering* (*"`compare_values` is REUSED from `matcher.rs`… so an ordering definition can never drift"*) but stops one level too shallow: it protects the `Ordering` computation, not the `CmpKind → bool` dispatch table sitting on top of it, which is still duplicated verbatim — the same six arms, written with `?`-propagation in one file and `matches!` in the other.
- **Where each concern should live:** one `pub(crate) fn cmp_holds(op: CmpKind, a: &Value, b: &Value) -> Option<bool>` (or a method on `CmpKind`) in `clause.rs`, beside the type it dispatches on.
- **Judgment:** **structural, but low blast radius** — `CmpKind` is a closed six-variant enum overseen by `every_constraint_head_is_a_real_rete_row`, so drift risk is small in practice. Still precisely the duplicated-encoding shape the cast named, previously unflagged, and a one-function extraction closes it.

(I also checked `where_tree.rs:320-329`'s `range_holds` and `:392-405`'s `constraint_kind`/`is_eq_op`/`range_kind_of` — a *different* question, "which `Ordering` satisfies this operator given an already-computed `Ordering`," used only for discrimination-tree traversal, and already the *product* of a prior consolidation: their own doc at `:367-391` documents a FIFTH hand-match this one replaced by routing through `classify_constraint_head`. Not a new finding.)

## Finding 2 — `export.rs`'s `pack_X`/`unpack_X` pairs: the encoder's exhaustiveness is compiler-enforced, the decoder's is not

- **Sites:** `export.rs:755-829` (`pack_expr`) vs `:846-1108` (`unpack_expr`); same at `:661-684`/`:689-746` (`pack_pat`/`unpack_pat`). The pattern recurs through the file's inventory — `pack_cmp`/`unpack_cmp`, `pack_prog`/`unpack_prog`, `pack_cond_op`/`unpack_cond_op`, `pack_compiled_cond`/`unpack_compiled_cond`, `pack_driver`/`unpack_driver`, `pack_fold`/`unpack_fold`, `pack_rhs_op`/`unpack_rhs_op`, `pack_rhs`/`unpack_rhs` — ten pairs. I read the `Pat`/`Expr` pair in full and confirmed the same tag-then-fields shape from the signatures of the rest.
- **The braid:** `pack_expr` is a bare `match e: &Expr` with no catch-all — the compiler forces a new arm the moment `Expr` grows a variant (the file says so at `:751-754`: *"the one whose exhaustiveness the compiler enforces for you… Its inverse cannot get that guarantee"*). `unpack_expr` matches on a **string tag** read at runtime, with `other => Err(malformed(…))` as its catch-all (`:1106`). A new `Expr` variant therefore compiles cleanly on the decode side even if nobody writes its arm — it degrades to "unknown expr" at runtime instead of a build failure. The module doc (`:834-839`) already names this precisely and names the mitigant as a **runtime test corpus**, not a structural guarantee.
- **Where each concern should live:** the ward's own "duplicated encoding" case — a single declarative shape (a per-variant table, or a macro emitting both match arms from one declaration) so a missed `unpack` arm is a build failure rather than a corpus gap.
- **Recommendation:** shared per-variant table or macro; short of that, a `rune:solvere(irreducible-tangle)` at each `unpack_X` acknowledging the asymmetric-enforcement risk — **none of the ten pairs currently carries one.**
- **Judgment:** **structural** — it recurs across essentially every serialized type in the file, the module doc already diagnoses it accurately, and it is currently open (mitigated by corpus, not closed by construction).

## Finding 3 — `compile-rule` / `compile-query` duplicate the same fold-and-wire pipeline

- **Sites:** `wat/rete/compile.wat:1079-1100` (`compile-rule`) and `:1103-1124` (`compile-query`).
- **The braid:** both perform the identical sequence — `sort-lhs`, build a `CondFoldAcc`, `foldl :wat::rete::compile-condition`, destructure `state2`/`pids`, destructure `network2`/`next-id2`, build a terminal node, `assoc` it into the network, `wire-parents`, construct the next `CompileState` with `next-id` incremented — differing only in (a) `compile-rule`'s extra `_rhs-fence` step and (b) the terminal-node type (`ProductionNode` vs `QueryNode`). The comment directly above `compile-query` (`:1102`) even names it: *"compile-query — same LHS fold as compile-rule; terminal is a QueryNode"* — **self-diagnosed, never extracted.**
- **Where each concern should live:** one helper, e.g. `compile-terminal [state lhs terminal-builder]`, called by both.
- **Judgment:** **structural** — the same "one grammar rule, two hand-rolled copies" shape, found on the wat-spec side. It will recur the day a third terminal kind is added.

## Runes encountered

- `src/rete/matcher.rs:998-999` — `rune:solvere(load-bearing-coupling)` on `value_to_ast_literal`, reason: *"encode includes keyword/unit/enum-unit; decode for operands must not, or field refs become values."* **Verdict: valid.** I read the function (`:1000-1025`) and its sibling `ast_literal_value` (`:980-987`) — the asymmetry is real and load-bearing for the reason stated (a keyword in operand position must stay a field-reference, never round-trip as a value); the reason is non-empty and specific.
- This was the **only** `rune:solvere` in the entire 25-file target (`grep -rn "rune:solvere" src/rete/ wat/rete.wat wat/rete/*.wat`) — notably, none of the ten `pack_X`/`unpack_X` pairs carries one despite the module doc reasoning about the same load-bearing-asymmetry shape this rune category exists to name.

## Close

**FINDINGS** — 3 rows (2 in the Rust compile side, 1 in the wat spec half), plus 1 rune reviewed and passed.
