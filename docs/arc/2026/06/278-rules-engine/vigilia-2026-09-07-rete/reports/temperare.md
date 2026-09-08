# Temperare — final report

> Written verbatim as returned. `&lt;`/`&gt;`/`&amp;` are HTML-entity artifacts of the agent's output.

**Scope cast:** `src/rete/kernel/**` (excl. `tests/`, 13,451 lines, 22 files) + `wat/rete/oracle/**` (2,320 lines, 6 files). All 28 files read in full, split across four parallel readers whose findings were then verified against the source for the two substantive hits below.

## Findings

**1. `src/rete/kernel/fire/pass/filter_after_join.rs:71` and `:98` — loop-invariant clone (redundant-call)**

Inside `for filter_id in filter_kids.iter().copied()` (one HashJoin/frontier node `hj_id`), both branches independently execute `d_beta.get(&amp;hj_id) … .clone()` (lines 71 and 98), keyed on the *outer* loop variable `hj_id`, not on `filter_id`. Verified: line 59 opens the loop over `filter_kids`; line 70's `HashJoin` arm re-fetches+clones at 71 then `continue`s (line 90); the fallthrough Test/Negation/Exists arm re-fetches+clones the same `d_beta[hj_id]` again at 98. Across multiple children of one join, the parent's delta `Vec&lt;Token&gt;` gets fetched-and-cloned once per child instead of once for the whole loop.
- Dimension that grows: number of Test/Negation/Exists/HashJoin children sharing one join node's delta (fan-out of trailing filter clauses off a single join) × tokens in that delta.
- Contrast confirmed by direct read: the structurally analogous sibling pass `join_after_filter.rs:62-65` fetches `d_beta.get(node_id)` **once**, before its own `for child_id in child_ids` loop — same shape, already hoisted there.
- Estimated savings: one clone of a `Vec&lt;Token&gt;` avoided per extra filter-child beyond the first, per round — order-of-magnitude bound by filter-fan-out (typically single digits per join), not by fact volume.
- Tempered direction: hoist `d_beta.get(&amp;hj_id).cloned()` once above the `for filter_id in filter_kids` loop, matching `join_after_filter.rs`'s existing shape.

**2. `src/rete/kernel/fire/pass/round_census.rs:103` and `:109` — redundant pure call**

`right_idx.per_join_marks()` is called twice building one `RoundCensus`: once for the `right_idx_by_join` field (line 103), again at line 109 (`let marks = right_idx.per_join_marks();`). Same receiver, same (no) arguments, same round — a pure recomputation of the identical structure.
- Dimension that grows: number of HashJoin nodes carrying a right-index mark, once per round census snapshot.
- Scope note: this is `#[cfg(test)]`/`FIRE_CENSUS`-gated instrumentation, not release fire-path cost, but the file's own header stresses these numbers are hand-verified against real cost claims — a duplicated allocation here skews whatever the instrument reports.
- Tempered direction: bind `let marks = right_idx.per_join_marks();` once and reuse it for both fields.

## Runes encountered (5 of 5 — all confirmed by direct read)

| Location | Reason given | Verdict |
|---|---|---|
| `fire/pass/filter.rs:57` | "3.7 still get_node+node_children; 3.6 already walks arm.children_of. n HashJoin×filter descendants is small vs intern hoist." | Justified — concrete cost ceiling cited. |
| `fire/pass/root_join.rs:53` | "kind_of filters mixed children_of; typed child lists at intern would drop the Value-network probe. n children × rounds is small." | Justified — same fan-out-bound ceiling argument. |
| `fire/mod.rs:311` | "And is sequential join of kid extensions; empty short-circuits. A specialized 2-kid path would duplicate the fold." | Justified, though a code-structure argument rather than a runtime-cost trade — accept. |
| `fire/mod.rs:494` | "combinator :not/:exists still PMap::from_pairs; leaf already uses BindView. n tokens with combinator inners is the rare path. Measured 2026-09-07 on floor 2026-09-07T02-03-18Z (5471 passed): re-derives 206 / hoisted 245583 (0.084%)." | Well-supported — dated, cites the specific floor run and an actual measured ratio rather than an estimate. **Model case for how this rune should be written.** |
| `fire/rules.rs:682` | "AST sessions re-walk lhs/rhs to learn max_s; interned arm.rule_deps is the Export door. n rules is small." | Justified — `rules` is authored/schema-scale, not fact-scale. |

All five reasons hold; none is struck.

## Prior-closed items (off-limits, encountered but not re-reported)

All five named closures were located and confirmed already fixed: `join_extend`'s SipHash triple, `key_of_el`/`col_field_of` hoist, `production_delta`'s per-fact `entry`, and `root_join_delta`'s per-element map ops are each named directly in `census.rs`'s own doc comments as "temperare §1"–"§4" fixes. `ensure_gather`'s per-token cache-key re-derivation: the call site at `fire/pass/accumulate.rs:154-164` was read and confirmed to be a caller of the already-cured helper, not a new site.

## Oracle (`wat/rete/oracle/**`)

Read in full. Per the ward's binding exemption for this group, the designed-in whole-set-replay cost (re-deriving `topological-node-ids` per pass entry, re-scanning `network`, re-running `fire-once$oracle` from scratch every round, per-token re-evaluation) was read but correctly excluded as the oracle's contract, not waste. No same-expression/duplicate-pure-call redundancy — the only in-scope shape for this group — was found anywhere in the six files. No `rune:temperare` occurs in this tree. Zero findings.

One non-temperare item noted for awareness: `stratify.wat:178-184` documents the known oracle/native stratum divergence — a correctness/documentation matter, out of this spell's category.

## Scope not covered by findings (converged clean)

`session.rs`, `arm.rs`, `census.rs`, `stratify.rs` (5,297 lines) read in full, no findings: `session.rs` is fire-entry/exit boundary code (once per freeze/thaw, not per fact/token/round); `arm.rs` is the once-per-network compiler explicitly documented as the hoist that avoids per-round re-derivation; `census.rs` is `#[cfg(test)]`-only with no-op release twins; `stratify.rs` runs once per compile, bounded by rule/type count.

`fire/mod.rs`, `fire/rules.rs`, `fire/delta.rs`, `fire/acc.rs` (4,713 lines) read in full; beyond the runes above, several call sites (`col_fields_for`, `dispatch_where_tests`'s `covers(tid)` hoist, `acc.rs`'s bucket-level `packed_operand_field`) were checked and confirmed already correctly hoisted.

## CONVERGED

Examined all 22 `src/rete/kernel/**` files (excl. `tests/`) and all 6 `wat/rete/oracle/**` files, 15,771 lines, in full. 2 findings (both loop-scoped redundant work in the fire-pass tier, both with a small hoistable dimension). 5/5 runes located, read, upheld. 5/5 prior-closed items located and correctly skipped. 0 findings in the oracle, per its binding exemption.
