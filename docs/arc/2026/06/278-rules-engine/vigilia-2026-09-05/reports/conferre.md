# ward `conferre` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

## conferre — wat-rs @ `21530efab` (`grok-rete`)

Read-only. Everything below was read from this working tree this session; both sides of every pairing are cited.

---

## L1

### L1-1 · `harvest-support` walks HAMT order, the native walks ascending ids — and the oracle's own doc asserts they match

**Spec side.** `wat/rete/oracle/explain.wat:53` — *"First-producer-wins, matching the native index."* The body it describes folds over `(:wat::core::PersistentMap/keys network)` at `wat/rete/oracle/explain.wat:49`.

**Code side.** `src/rete/kernel/fire/pass/production.rs:115-123` records support via `idx.entry(derived).or_insert_with(...)`, inside the walk `for node_id in &arm.kind_ids.prod` (`production.rs:37`). `arm.kind_ids.prod` is built by `kind_id_lists` from `node_ids` (`src/rete/kernel/arm.rs:529-548`), and `node_ids` comes from `sorted_node_ids`, which ends `ids.sort_unstable()` (`src/rete/kernel/node.rs:193-204`).

**That the two orders differ is a fact of this tree, not an inference.** `PMap` has two arms: `Array` (≤ 8 entries, insertion-ordered) and `Trie` (above it, HAMT), `src/value/pmap.rs:93,98-104`; `keys()` is just `self.iter()` (`src/value/pmap.rs:295-297`). Any network past 8 nodes is a Trie. The oracle already knows this and says so, in the sibling file: `wat/rete/oracle/fire.wat:151-160` — *"`PersistentMap/keys` is HAMT order — not that … Native sorts (`sorted_node_ids`); the spec must too"* — and applies `(:wat::core::sort …)` there. `harvest-support` never got that sort.

There is a **second, independent** order divergence at the same site. Native's "first producer" is gated by `if seen_insert(...)` (`production.rs:110`), i.e. *the first round of the incremental fixpoint* that derives the fact. The oracle's is *the first ProductionNode in one full replay over the already-closed fact set* (`explain.wat:78-101` builds a fresh session over `closed` and replays `fire-once$oracle`). Where two rules derive the same fact in different rounds, native credits the earlier round's rule regardless of node id; the oracle cannot see rounds at all.

**Which half I believe is wrong: the oracle.** The precedent is in the same file family, was written down, and was not applied. Native's traversal is deterministic and topological; the oracle's is neither.

**Consequence.** `fire-rules-explain$oracle` can attribute a derived fact to a different rule than `fire-rules-explain` whenever two rules derive one fact — and the differential cannot see it: `tests/rete/probe_arc278_P12a_explain_substrate.rs:49-54` compares only `support-index-length` (cardinality), and its fixture has two rules deriving two *different* facts (`…_substrate.wat:9,16`). The `Support/rule` field is never compared by any gate.

**Same root, second site.** `node-parents` (`wat/rete/oracle/pass.wat:378-395`) folds the same `(:wat::core::PersistentMap/keys network)`; native's `parents_of` is filled walking sorted `node_ids` (`src/rete/kernel/arm.rs:589-607`), so it is ascending. `node-parents` feeds `tokens-from-parents`, which feeds TestNode/ExistsNode filtering, `fire-production`, and `collect-query-memory` — so for any rule with condition `:or` (N arm terminals) the token order, and therefore the **query answer row order**, differs between the two engines.

---

### L1-2 · `retract` removes every equal fact; `insert` stages one — and this repo forbids exactly that collapse, in writing

**Code.** `wat/rete/oracle/insert.wat:100-113` — the foldl keeps `f` iff `(not (= f fact))`, i.e. it drops **all** copies. Doc at `:95` calls it *"Symmetric with insert"*, at `:97` *"value-precise"*. There is **no native dual**: `grep -rn retract src/ --include=*.rs` returns four unrelated prose hits and zero handlers, so this wat body is the entire behaviour.

**The contradicting spec, in this repo.** `wat/rete/oracle/fire.wat:235-238`, the ⛔ on `retain-supported`: *"`insert$oracle` never dedups, so a caller that stages the same fact twice genuinely holds it twice and its alpha memory carries two elements. Collapsing here would silently retract a duplicate the INPUT contains — a retraction with no cause."* That is a verbatim description of what `retract` does.

**Which half I believe is wrong: the code.** Multiplicity in `facts` is load-bearing by explicit design (it is what `retain-supported`'s length test is proved on, same paragraph), and `retract` is documented as insert's inverse. A one-add inverse that removes N is not the inverse.

**Consequence.** Insert `f` twice, retract once → both gone, alpha loses both elements, and every `acc::count` / `:exists` / join multiplicity over that type answers as if `f` had never been inserted. Silent wrong answer, no diagnostic. Live callers: `wat-scripts/perf/grid/where-exists.wat:91-92`, `where-not-fact.wat:48,55`, `where-not-or.wat:80`, `tests/rete/probe_arc278_4c_retraction.wat:83,92,101`, `probe_arc278_P4c_native_retraction.wat:47`.

---

## L2

### L2-1 · A leading accumulate re-seeds every round into a cumulative beta; leading `:not`/`:exists` is guarded against exactly that

`src/rete/kernel/fire/pass/accumulate.rs:134-142` — for an accumulate with no parents, `new_tokens = vec![Token{empty, empty}]` is rebuilt **every round**, unguarded, and `record_token` (`fire/pass/mod.rs:60-73`) pushes it into the cumulative `wm.beta` whenever the node is a `beta_reader` (any node with a HashJoin or Query child, `arm.rs:608-618`).

Its sibling pass has the guard and names the harm: `src/rete/kernel/fire/pass/filter.rs:85-90` seeds leading `:not`, and `filter.rs:250-260` refuses to re-pass it — *"Without this it re-seeded and re-passed one empty token on EVERY round into a cumulative beta."* Leading `:exists` gets a round gate at `filter.rs:100-108`.

The oracle has no such exposure: `wat/rete/oracle/accum-pass.wat:224` calls `tokens-or-empty-seed` (`pass.wat:631-643`), but `fire-once$oracle` starts every replay from `(:wat::core::PersistentMap)` for all three memories (`wat/rete/oracle/fire.wat:162-168`), so the seed happens once per replay into a fresh beta.

**Partly mitigated, and I found the mitigation before writing this:** `src/rete/kernel/fire/rules.rs:558-579` re-harvests query memory from the closed fact set whenever a query's upstream closure reaches an `Accumulate` or `Negation`, which throws the accumulated beta away for `fire-rules`. So the *query* observable is covered. What that repair does not touch is `left_idx` — *"all left tokens seen so far for J"*, `src/rete/kernel/fire/delta.rs:320-324` — which is cumulative across rounds and would take one duplicate copy of the leading accumulate's token per round.

**Blunt:** the divergence in the seed is grounded and read; the downstream join consequence is inferred and **not driven**. Remedy either way is the cheap one: give the leading-accumulate seed the same `leading_emitted`/`d_alpha` gate its sibling has, keyed on the aggregate value so a genuine recompute still emits.

### L2-2 · `insert-all`'s hardcoded `OP` defeats the stated reason its checker is parameterised

`src/rete/kernel/insert.rs:142-145`: *"Takes `op` rather than hardcoding one so the same check serves `insert` and `insert-all` and reports the verb the user actually wrote."*

`insert_facts_on_session` (`insert.rs:213`) hardcodes `const OP: &str = ":wat::rete::insert-all"` at `:219` and passes it to both `require_session_agg` and `require_record_fact` — and it is reached from **two** entry points: `eval_insert_all_native` (`:262`, correct) and `eval_insert_public`'s 3+-arity arm (`insert.rs:96-106`), where the user wrote `:wat::rete::insert` (the wat surface being the `defclause` 3+ arity at `wat/rete/oracle/insert.wat:81-93`). So `(:wat::rete::insert s f1 f2 <not-a-record>)` reports a TypeMismatch on a verb the author never typed. The code is the wrong half; thread `op` from the entry.

### L2-3 · Stratify: the native has a `+1` for `:exists` / accumulate-`:from` over a derived type; the oracle does not, and claims lockstep

`src/rete/kernel/stratify.rs:216-228` raises `required` to `stratum[b] + 1` for every `b` in `view.exists_and_from_types` that the rule set derives — *"exists / acc :from of a type THIS SET derives: +1 (closed bag)"* — fed by `rule_bag_consumes` / `bag_types` (`stratify.rs:145-164`).

The oracle has no bag partition. `rule-consumes` (`wat/rete/oracle/stratify.wat:160-200`) folds `:exists` inner and acc `:from` into the *same* `consumed` list as ordinary positive reads, and `stratify-sweep` applies them with `req-pos` at **+0**, commented *"NOT +1"* (`wat/rete/oracle/stratify.wat:233-235`). `rule-consumes`' own header at `:158` asserts *"lockstep with native `rule_consumes`"* — true of the function it names, false of the stratification the two compute.

**Which half:** native's rule is the sounder one — an `:exists`/`:from` reads a *bag*, which is only correct once the bag is closed. The oracle survives at +0 only because it re-runs the whole match to a grow-then-shrink fixpoint (`wat/rete/oracle/fire.wat:376-381`), which is not a property the numbering can claim. The two now assign different strata to the same rule set; nothing compares strata.

---

## L3

- **The differential's blind spot is a shape, not a gap.** Every oracle/native comparison I opened compares a **set or a count** (`:derived` deduped, `support-index-length`, `[rows sum]`). The D2 commit message says this itself — *"`:derived` IS blind — confirmed — but A CHAIN-MIRRORING QUERY IS NOT"*. Both L1s above live in the same place: **ordering and multiplicity of an ordered observable**. `Support/rule`, query row order, and `production-memory`'s per-node shape are all reachable from wat and all uncompared.
- **`fire-once$oracle`'s sort comment is the best artefact in the oracle and it was applied once.** It states the general law ("HAMT order is not topological; native sorts; the spec must too") and then fixes exactly one of the five `PersistentMap/keys network` folds in the oracle (`explain.wat:49`, `pass.wat:177,224,395`, `fire.wat:124,161` are the others). Two of those five — `alpha-feeding` and `alpha-id-for-cond` — are order-insensitive because their match is unique; `node-parents` and `harvest-support` are not, and are L1-1.
- **The oracle is code, and it is the half that was wrong in two of the three closed precedents cited in this tree** (`probe_arc278_oracle_accumulate_supersedes.rs:3` — *"THE ORACLE WAS THE ONE THAT WAS WRONG"*). Worth carrying into the next cast: "spec side" here does not mean "presumed right".

---

## What I could not check, and why

- **`wat/rete/compile.wat` (1163 lines) against the native compiler.** I read only `exists-uses-alpha-probe?` (:87-96), `mint-leaf-alphas` (:117-127) and `sort-lhs` (:1002-1100). Alpha minting/dedup, `wire-parents`, node-id assignment, and the `:or` arm-terminal wiring are **unaudited** — and that is where L1-1's consequence actually bites, so the ordering finding is grounded but its blast radius is not bounded.
- **`wat/rete/syntax.wat` (368 lines) against `src/rete/clause.rs` / `where_tree.rs` / `expr_ir/`.** Not opened. This is the D5 class (a form legal in one position, refused in another) and I have no coverage of it.
- **Export/import ABI** (`src/rete/export.rs`, 1500+ lines) against `wat/rete.wat`'s `Export` bounds. Not opened. The native is documented as the only engine that consumes an Export, so there is no oracle to compare against — a conferre pairing that structurally cannot exist, which is itself worth someone's attention.
- **L2-1's downstream consequence is not driven.** I am read-only and did not build, fire, or run the floor. The seed asymmetry is read from both files; the duplicate-`left_idx` consequence is a reading of `delta.rs:320-324`, not a measurement. Do not treat it as a defect until someone fires a leading accumulate feeding a HashJoin across ≥2 rounds and counts the rows.
- **No mutation proof for any of this.** Every finding here is a divergence between two texts I read; none of it is a red I produced. The two L1s are falsifiable by a ten-line probe each (two rules deriving one fact, then compare `Support/rule` across engines; insert-twice-retract-once, then count alpha), and neither probe exists in `tests/rete/`.
