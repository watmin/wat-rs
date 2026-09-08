## INTUERI — Cast Report (TARGET 2 — the compile side and its spec)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

**SCOPE.** All 25 files read in full: 20 Rust + 5 wat spec files — 23,886 lines, re-derived by `wc -l` (**delta 0** from the handed count). I personally read `mod.rs, collect.rs, eval_test.rs, step_payload.rs, alpha_tree.rs, eval_insert.rs, clause.rs, compiled_rhs.rs, where_tree.rs, matcher.rs` (10 files) directly. Three forks covered the remaining 15 files independently, each applying the same method: read a function's doc header, then its full body, and ask whether the header covers every responsibility/branch the body executes (the "undeclared second job" shape from target 1). I then re-derived all three of the third fork's findings myself against the live file before including them — one fork initially mistook itself for a coordinator and briefly delegated out of scope; I disregarded that side work and used only its own direct, re-verified findings.

`src/rete/kernel/**` was not read (correctly out of scope). **No `docs/*.md` claim was used as authority anywhere.**

### Findings

**1. `src/rete/purity.rs:1811-1818` — Level 2 (mumbles/misnames a mechanism)**
`walk_rete_defn_callees`'s doc: *"Walk a rete definition's callees for **purity**, returning the first **impure** one with its span… purity is a yes/no property of the definition, and one located counterexample is the whole proof."* The function does no purity classification at all — it is a gray/black DFS **cycle detector** (a `gray.contains(head)` back-edge = recursion, verified at lines 1825-1870), called only from `rete_defn_cycle` (#87, recursion). Two adjacent comments in the same file correctly distinguish this from purity: `rete_defn_cycle`'s own doc (line ~1802) says *"`pure?` is unchanged: this walk is a LOAD refusal, not a fifth axis,"* and `apply_rete_defn_contracts`'s inline comment (line ~1782) says *"cycle is a second question (#87 recursion), not a fifth fence axis."* Only this one function's header reaches for purity language, in a file where "pure" is an extremely load-bearing, precisely-scoped term. A reader trusting this doc in isolation would look for the wrong bug class.
Direction: reword to name the recursion cycle explicitly.

**2. `src/rete/export.rs:2274` — Level 1 (stale/checkable numeric claim)**
`import_export`'s doc: *"Its 194 lines are phase COUNT rather than depth."* Re-derived by brace-matching: the function body (lines 2278-2535) is actually **258 lines**. The qualitative claim (9 phases, brace nesting peaks at 3) is accurate and still holds — only the line count is stale.
Direction: drop the number (the qualitative point stands without it) or recompute it.

**3. `src/rete/vocabulary.rs:104-105` and `:1845` — Level 1 (stale/checkable numeric claim, two sites)**
Module header (line 104-105): *"Nine rows total in `NAMING_RULE_EXCEPTIONS`, not six."* Re-derived: the array (line 1775) has **14** entries — confirmed by its own gate `assert_eq!(NAMING_RULE_EXCEPTIONS.len(), 14)` at line 1853, and by direct count (8 equality rows + 3 `first` rows + 3 `Tuple` accessors = 14). The array's own inline comments tracked its growth correctly as it grew; the header prose ~1700 lines above was never revised.
A second, distinct staleness on the same fact: the test's own doc comment immediately above the count-gate (line 1845) says *"exactly the eleven rows the module doc names"* — while the test function itself is named `naming_rule_exceptions_are_exactly_the_documented_fourteen` and asserts `len() == 14`. So the doc comment is stale even against its own test's name and body, two lines away.
Direction: drop the number from both prose sites, or replace with "count enforced by the gate below"; fix the "eleven" in the test's own doc comment.

**4. `src/rete/compiled_cond.rs:1550-1613` — Level 2 (misleading assertion text)**
`lands()`'s doc (1552-1556) states `Op::Eval` is `Bind`'s sibling and lands the same way — both `Lands::Driver` (confirmed at 1557). But the test `every_op_variant_lands_in_core_or_driver`'s `variants` fixture (1568-1590) omits `Op::Eval{..}` entirely, then asserts `driver.len() == 1` with message *"driver must be exactly Bind, got {driver:?}"* (1604-1608) — a claim that directly contradicts the classifier's own doc two lines above it. It's true only because `Eval` was never in the fixture; the message overclaims a coverage the test does not have.
Direction: either add `Op::Eval{..}` to `variants` and assert `driver.len() == 2`, naming both variants, or reword the message.

### Sub-threshold note (not filed as a finding)
`src/rete/expr_ir/mod.rs:428-430` — `lower_hof_callee`'s doc says "It sets `cx.hof_fn_pos`," but the flag is actually set by the caller (`lower_call_args`) and cleared by `lower_expr` before `lower_hof_callee` runs; the function itself never touches the field. Imprecise about mechanism ownership, not a lie about behavior.

### rune:intueri census
**Zero** `rune:intueri(...)` comments exist anywhere in the 25-file target (direct grep). The one instance in the repo (`wat/rete/oracle/fire.wat:54`) is outside this target's file list.

### PRIOR ART — not re-reported
Confirmed still standing at the cited sites and excluded: `validate/mod.rs:709` (2C1), `clause.rs`'s `Accumulate.var` rune (2P1, verified again independently at 69-74), `compiled_cond.rs:245` (2X1), `export.rs`'s module doc (2S2), `wat/rete/compile.wat:1102` (2S3), `purity.rs:168-169` (2F1), and the settled `rune:lint(...)` runes.

FINDINGS — 4 findings (1 Level 2 mechanism-mislabel, 2 Level 1 stale-number, 1 Level 2 misleading-assertion), plus 1 sub-threshold note, 0 `rune:intueri` instances in target.
