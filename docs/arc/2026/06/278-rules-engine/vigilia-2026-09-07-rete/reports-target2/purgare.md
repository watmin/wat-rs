## PURGARE — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.
> HTML entities (`&lt;` `&gt;` `&amp;`) are artifacts of the agent's own output encoding.

## SCOPE

Target: 25 files, 23,886 lines, all read in full this session (split across 5 parallel sweeps, groups A–E, then independently re-verified).

Liveness method used throughout: grep every candidate's Rust name across `src/` (incl. `src/runtime.rs` dispatch tables, `src/check.rs` type registration), then grep its wat-facing name across `wat/`, `wat-scripts/`, `wat-tests/`, `tests/` (tests count as a live consumer; `.wat` calls into Rust primitives and vice versa both count). `reachability.rs`'s disconfirming-probe header and `purity.rs`/`kernel/stratify.rs` duplication were read and treated as already-adjudicated per the cast's prior art — not re-flagged.

I independently re-verified every finding below by re-reading the cited lines and re-running the sweep agents' grep commands myself (not merely trusting their summaries).

Groups C (`purity.rs`, `reachability.rs`, `step_payload.rs`) and D (`vocabulary.rs`, `where_tree.rs`, `expr_ir/{eval,mod}.rs`) came back **CLEAN**.

## FINDINGS

**1. `src/rete/clause.rs:68-71` — stale/false rune on `ReteClauseShape::Accumulate.var`**
```rust
// rune:purgare(trait-contract) — classifier names the full accumulate shape
// (`?var <- acc-form :from inner`); current consumers only walk `from`.
#[allow(dead_code)]
var: &'a str,
```
Evidence: `var` IS read — `src/rete/validate/mod.rs:561-562`: `ReteClauseShape::Accumulate { var, from, .. } => { out.push(var); collect_all_declarations(from, out); }`, feeding the freeze-time "trapped bind" wall (`validate_wrapper_binds`). The rune's stated reason is factually false today; the `#[allow(dead_code)]` is unneeded. Severity L2. Recommendation: delete the stale rune/attribute (field itself must stay — it's live). Cost: leaf.

**2. `src/rete/clause.rs:72-76` — `ReteClauseShape::Accumulate.acc_form` field is genuinely dead; rune category mislabeled**
Evidence: `grep -rn "Accumulate { acc_form\|Accumulate{ acc_form" src/` → 0 hits; every destructuring site (`matcher.rs:806`, `compiled_cond.rs:598`, `kernel/stratify.rs:173,196`, `validate/typing.rs:515`, `validate/mod.rs:293,460,561,616`) discards it via `..`. The rune's *reason* is plausible (fire reads acc-form off the compiled `AccumulateNode` instead), but the category `trait-contract` doesn't fit — this is a plain enum-variant field, not a trait-bound-mandated item. Severity L2. Recommendation: recategorize rune to `future-fixture`. Cost: leaf.

**3. `src/rete/matcher.rs:484-568,843` — `sym: Option<&SymbolTable>` is never `None` anywhere**
Threaded through `alpha_match_inner`/`_local`/`_seeded`/`_opts` → `eval_clauses`/`eval_clause` → `eval_computed_operand` (does `let sym = sym?;`) → `resolve_operand`. Verified all 8 call sites repo-wide (`matcher.rs:325,327,410`; `compiled_cond.rs:1471`; `kernel/tests/alpha_discrimination.rs:97,275,418,516`) pass `Some(...)`; zero `None` call sites found. The function's own doc at ~841 concedes "`None` when there is no `SymbolTable` (no caller lacks one today)" — an honest admission, but not phrased as a `rune:purgare` exemption. Severity L2 (Honest? violation — the type advertises an unreached branch; no wrong value is ever produced). Recommendation: either add `rune:purgare(safety-margin)` or collapse to `sym: &SymbolTable`. Cost: cascade (4 signatures + `eval_computed_operand` + `resolve_operand`'s `.or_else` arm, plus 8 call sites).

**4. `src/rete/export.rs:2286-2300` — write-only `agg` binding in `import_export`**
```rust
let agg = match export { Value::Aggregate(a) if ... => a, other => { return Err(...); } };
let _ = agg;
```
`agg` is bound, immediately discarded via `let _ = agg;`, and never read — the type/class refusal is fully done inside the match arms. Severity L2 (reader tax, no behavior effect). Recommendation: rewrite as a `matches!`-guarded early return without binding. Cost: leaf.

**5. `src/rete/eval_test.rs:72-73` — rune category mislabeled on `eval_test_core`'s `env` parameter**
Verified all 5 call sites (`eval_test.rs:150`; `kernel/tests/node_share_cost.rs:142,196,374,395`) pass a freshly built `Environment::new()` — a genuine constant-parameter pattern, correctly identified and correctly placed. But `eval_test_core` is a plain function, not a trait impl — `trait-contract` doesn't fit the doctrine's own taxonomy. Severity L2 (informational). Recommendation: recategorize to `future-fixture`. Cost: leaf.

**6. `wat/rete/factbag.wat:103-115` — `:wat::rete::factbag::count-of` is dead**
Evidence: `grep -rn "count-of" --include=*.wat --include=*.rs .` — only hits are the definition and its own doc-comment listing (`factbag.wat:13`). The other `count-of` hits repo-wide belong to an unrelated `:sq::count-of`/`:t118b::count-of` namespace (scratch-pad probes, type-system tests) — confirmed by full-name match, not substring. No Rust dispatch match on `count-of`/`count_of` in `vocabulary.rs`/`runtime.rs`/`check.rs`. Severity L2. Recommendation: delete, or add `;; rune:purgare(future-fixture)` if held for the file's documented "rung-3 seal." Cost: leaf (plus trimming the doc-comment line).

**7. `wat/rete/compile.wat:263-266` — `AxisViolation`'s `axis` and `span` fields are populated on every construction but never read**
Verified construction at `src/rete/purity.rs:2055-2070` builds real, non-placeholder `axis`/`span` values on every call. Verified consumption: repo-wide, only `AxisViolation/head` is ever accessed (`compile.wat:332,339,346,359`, inside `axis-violation-message`); zero `AxisViolation/axis` or `AxisViolation/span` accessors anywhere in wat/wat-scripts/tests/src, and no generic native field-reader touches "axis"/"span" on this record. (Distinct from the unrelated Rust-native `ReteDefnAxisViolation` error variant in `src/value/signal.rs` — confirmed a different type.) Severity L2 — the record's own doc promises a caller can "report as well as the substrate can" using axis+span, but no caller does; the diagnostic today only ever uses `head`. Recommendation: wire `axis-violation-message`'s 4 arms to use `span` (and `axis`), or drop the two fields, or mark with `rune:purgare(future-fixture)`. Cost: leaf to drop; cascade (4 arms) to wire up.

## No findings in

`alpha_tree.rs`, `collect.rs`, `eval_insert.rs`, `mod.rs`, `purity.rs`, `reachability.rs`, `step_payload.rs`, `vocabulary.rs` (including the 108-row `RETE_OPS` table), `where_tree.rs`, `expr_ir/eval.rs`, `expr_ir/mod.rs`, `validate/error.rs`, `validate/mod.rs`, `validate/typing.rs`, `wat/rete.wat`, `wat/rete/syntax.wat`, `wat/rete/acc.wat` — every checked struct/fn/field/enum variant/def traced to a live consumer (production, wat-corpus, or test).

## FINDINGS: 7

(2 stale/mislabeled runes on live fields, 1 genuinely dead struct field with a mislabeled-but-plausible rune, 1 always-`Some` `Option` parameter family, 1 write-only local binding, 1 mislabeled rune on a genuinely constant parameter, 1 dead wat function, 1 wat record with two write-only fields.) All L2 — no L1 correctness lies found in this target.
