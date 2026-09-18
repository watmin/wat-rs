# DESIGN — Stone P4: the delta engine (smart activation), decomposed

P4 is the closing capability of the Rust fire kernel: **incremental delta propagation = Clara's "smart
activation."** Today the native side has only `fire-once'` (a single non-cascading pass; P2/P3). The wat oracle
has the full cascade (`fire-rules` = `fire-fixpoint` over `fire-once`, re-run-from-scratch). P4 brings the
cascade to the native kernel and then makes it *incremental* (delta, not re-run).

P4-as-one-stone fails **Simple** (it is hazard #1 — Clara's delta logic). Decomposed, mirroring the P2→P3 shape
(*correct first, then a behavior-preserving perf transform behind the differential net*):

- **P4a — native `fire-rules'` (re-run fixpoint).** A native cascade verb that loops `fire-once'`, merges
  derived facts, terminates on no-new-fact — a 1:1 port of the wat `fire-fixpoint` + `fire-rules` (`wat/rete.wat:981-1018`).
  Differential: `query(fire-rules' s, T) == query(fire-rules s, T)` for every T, on a single rule AND a
  multi-rule cascade. NOT yet smart activation (still re-run each round); it establishes the native cascade verb
  + the fixpoint differential harness. **THIS STONE.**
- **P4b — delta-incremental `fire-rules'` (the smart activation).** Converts P4a's verb IN PLACE: memories
  persist + accumulate across rounds; each round propagates only the NEW facts (the delta) — alpha-match only
  the delta, join new elements against existing beta memories via the P3 keyed index, fire only new tokens — no
  full re-scan. Behavior-preserving (P4a's differential stays green); the join-scaling bench shows the
  re-run→delta bend (the P3 pattern again). **NEXT.**
- **P4c — delta retract + TM cascade.** The support store + the token `matches` chain (CLARA-REF §3): retract
  removes exactly the tokens/derived facts a fact supported, without a full re-run. Differential vs wat
  `retract` + `fire-rules`. **AFTER P4b.**
- **P5 — wire public `(:wat::rete::fire)` + bench vs baseline + Clara → ARC 278 CLOSES.**

---

## This stone: P4a — native `fire-rules'` (re-run fixpoint)

### What it adds
A new native primitive `(:wat::rete::fire-rules' <session>) -> :wat::rete::Session` — the native cascade. It is
to `fire-once'` what the wat `fire-rules` is to the wat `fire-once`: a fixpoint that lets derived facts re-enter
the network until no new fact is produced, then returns a Session with `facts = input only` (derived live in
`production-memory`), exactly matching the wat `fire-rules` contract.

### The algorithm (1:1 port of wat `fire-fixpoint` + `fire-rules`, `wat/rete.wat:981-1018`)
Re-run-from-scratch each round (this stone; P4b makes it delta):
1. `input = session.facts` (the retractable base — restored at the end).
2. **Fixpoint loop** (mirrors `fire-fixpoint`):
   - `fired = fire-once'(session)` — the existing native single pass (alpha → root-join → hash-join →
     production), which recomputes all memories from `session.facts`.
   - `derived = collect-derived(fired.production-memory)` — flatten the per-node `PV<Record>` into one
     `Vec<Value>` of derived facts (mirror `collect-derived`, `wat/rete.wat:940-955`).
   - `new-facts = merge-facts(session.facts, derived)` — conj only facts not already present (structural
     value-equality dedup; the termination guard, mirror `merge-facts`, `wat/rete.wat:960-972`).
   - If `len(new-facts) == len(session.facts)` → no new fact → **return `fired`** (the fixpoint).
   - Else recurse with `session' = fired` but `facts = new-facts` (so the next round's `fire-once'` matches
     input ∪ derived). Mirror the Session reconstruction at `wat/rete.wat:990-998`.
3. **Restore `facts = input`** on the returned Session (mirror `fire-rules`, `wat/rete.wat:1006-1018`): the
   returned `facts` holds ONLY the asserted/input facts; the derived closure lives in `production-memory`. This
   is the fact-model that makes 4c TM-via-replay correct.

### Reuse (what already exists)
- `eval_fire_once_native` / `fire_once` internals (`src/rete/kernel/fire/mod.rs`) — the single pass. P4a calls the pass
  logic in a loop. **Factor the pure pass out of `eval_fire_once_native`** so both the dispatch entry (which
  evals its arg) and the new fixpoint loop call it on an in-hand `Session` value, WITHOUT re-evaluating an AST
  arg each round. I.e. extract `fn fire_once_session(session: &Value, sym: &SymbolTable) -> Result<Value,
  EvalBreak>` (to_transient → clear → 4 passes → to_persistent); `eval_fire_once_native` = eval arg →
  `fire_once_session`. The fixpoint calls `fire_once_session` directly.
- `Value` equality (`==`) for the dedup guard — the same structural equality the wat `contains?` uses.
- The Session struct_form positions (network 0, rules 1, alpha 2, beta 3, production 4, facts 5, next-id 6).

### Reimplement (native, mirroring the wat helpers)
- `collect_derived(production_pm: &Value) -> Vec<Value>` — flatten production-memory's values.
- `merge_facts(facts: &Value (PV), derived: &[Value]) -> Value (PV)` — conj non-present (value-eq dedup).
- the fixpoint loop itself (a `loop`/recursion in Rust; termination = no-new-fact, same monotone-finite
  guarantee as the oracle — NO arbitrary round cap, matching the oracle).
- `eval_fire_rules_native` — the dispatch entry: eval the session arg, run the fixpoint, restore facts=input,
  return.

### The contract decision (pinned)
`fire-rules'` is **observationally equivalent** to the wat `fire-rules`: `query(fire-rules' s, T) == query(fire-rules s, T)`
for every type T. NOT bit-identical Session (P4b will restructure memories by design; the contract is observable
from day one, same as P2). The wat oracle is the reference; `fire-rules'` conforms to it.

### Files touched
- `src/rete/kernel/fire/` — extract `fire_once_session`; add `collect_derived`, `merge_facts`, the fixpoint, and
  `eval_fire_rules_native`. NO change to the four passes themselves, the keyed join, or `WorkingMemory`'s shape.
- `src/runtime.rs` — one dispatch arm: `:wat::rete::fire-rules'`.
- `src/check.rs` — one TypeScheme: `fire-rules'` (Session → Session, mirror `fire-once'`).
- `tests/probe_arc278_P4a_native_fire_rules.rs` — the differential probe (NEW; RED at HEAD).

### Verify
- the new differential probe → native `fire-rules'` == wat `fire-rules` (single rule + cascade chain).
- the P2 `fire-once'` differential stays 4/4 (the extraction of `fire_once_session` didn't change `fire-once'`).
- oracle byte-unchanged; lib floor 935/36; build clean.

## Out of scope = REJECTED (this stone)
- **Delta / incremental / persistent memories** — P4b. P4a is re-run-from-scratch (loops `fire-once'`).
- **Retract / TM cascade / support store** — P4c.
- **Public `fire` / Clara bench** — P5.
- **No change to the four passes, the keyed join, the oracle, or the `WorkingMemory` shape.**
