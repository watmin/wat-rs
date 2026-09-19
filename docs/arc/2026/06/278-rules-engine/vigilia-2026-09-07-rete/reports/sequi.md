# Sequi Ward — Report

> Written verbatim as returned. `&lt;`/`&gt;`/`&amp;` are HTML-entity artifacts of the agent's output.

**Verdict: CONVERGED.** Chains followed to their ends; no hidden domain-state break found in the fire path. One unruned ambient-state read is worth naming, and one existing rune's category label doesn't hold up cleanly against the ward's own definition — both adjudicated below, neither is a red.

## Chains followed

**1. The fire round chain — the ward's primary target**
`fire_fixpoint_delta_armed` (`src/rete/kernel/fire/delta.rs:271-833`), the loop body at `delta.rs:490-744`, calling in order: `alpha_seed`/`alpha_delta` (`delta.rs:515-554`) → `root_join_delta` (`delta.rs:560-566`) → `hash_join_delta` (`delta.rs:568-586`) → `accumulate_pass` (`delta.rs:588-605`) → `filter_pass` (`delta.rs:607-625`) → `join_after_filter` (`delta.rs:627-635`) → `filter_after_join` (`delta.rs:637-657`) → `production_delta` (`delta.rs:661-668`).

Each call constructs a fresh `RoundScratch { d_alpha, packed_full, bind_only, cond_key_ids, cand_scratch, match_scratch, seen, leaf_aids, pre_dispatched }` (struct at `fire/pass/mod.rs:148-163`) and threads `wm: &amp;mut FireSession`, `d_beta`, `left_idx`, `right_idx`, `gather_cache`, `leading_emitted`, and `support: &amp;mut Option&lt;&amp;mut ExplainSupport&gt;` explicitly through every call's signature. `production_delta` (`fire/pass/production.rs:27-151`) returns `next_delta: Vec&lt;u32&gt;` rather than writing an out-parameter, and the loop epilogue (`delta.rs:692-701`) binds it visibly. **This is the honest shape sequi defends — obvious, simple, honest, rewireable.** No break found.

**2. The oracle mirror — confirms the reference is itself clean**
`wat/rete/oracle/fire.wat:18-96`: `walk-alpha-ids`, `walk-beta-ids`, `walk-filter-ids`, `walk-prod-ids` each declare `acc &lt;- :wat::rete::*Memory` and `-&gt; :wat::rete::*Memory`, threading the accumulator through the recursive fold by value, textbook state-monad shape. No `set!`, no atom, no global found anywhere in `wat/rete/oracle/*.wat` (grepped for `set!`, `Atom`, `RefCell`, `thread-local`, `global` — zero hits).

**3. The two "duplicate copy" locals (`delta.rs:387-408`)** — `bind_only`/`cond_key_ids` are cloned into `wm.bind_only`/`wm.cond_key_ids` (`delta.rs:407-408`) for a borrow-split. The comment records this was already flagged by a prior sequi pass (2026-08-25) and resolved by argument, not by structural elimination: both copies are written from the same source exactly once, at the top of this same function, never mutated afterward, so they cannot diverge. Read and agree; a documented case rather than a live finding.

**4. `RoundScratch.pre_dispatched`** (`fire/pass/mod.rs:157-162`) — per the briefing, a sibling ward already flagged this holds its invariant by call order, not by type. Confirmed the site (declared in the same struct as every other explicitly-threaded field, so at least it's *visible* state, just not compiler-enforced ordering). Agreement noted; not re-derived further.

## Runes encountered — adjudicated

| Site | Category | Verdict |
|---|---|---|
| `census.rs:104,126,240,285,316,358,433,506,618,673,732,834` (11 marks) | `performance-counter` | Per given CONTEXT, `census.rs` is `#[cfg(test)]`-gated thread-local instrumentation, the ward's own ambient-context/dedicated-pipeline exemption. Not re-derived. |
| `arm.rs:727` `ARM_BUILDS` `AtomicUsize` | `performance-counter` | `#[cfg(test)]`-only intern-miss counter, carries no domain data, compiles out in release. **Sound.** |
| `arm.rs:717` `rune:circumspicere(accepted-by-design)` | (not sequi's) | Different ward's rune; read for context only. |
| `arm.rs:705` `ARM_TABLE` thread-local `RefCell&lt;FxHashMap&lt;u64, InternEntry&gt;&gt;` | `ambient-context` | **Label tension, not a violation.** The comment itself states the table "holds DOMAIN state (the armed network + its lease count)" (`arm.rs:710`) — but the ward's own definition of `ambient-context` is explicitly "a true cross-cutting concern... that doesn't carry domain state". Those two statements don't reconcile at face value. Read the actual chain, though: the fire path never touches `ARM_TABLE` mid-round — `rete_arm_get_or_build`/`rete_arm_lookup` run once, above the loop (`delta.rs:325-328`), and everything downstream receives the result as an explicit `arm: &amp;InternedNetwork` parameter threaded through every pass call listed in chain 1. `ARM_TABLE` is a cross-fire *build cache* with lease-counted eviction, sitting upstream of the state-threading chain the ward is scoped to, and constrained by an explicit architectural rule (`DESIGN-STONE-intern-zero-mutex`). So: the *category name* is arguably a stretch against the ward's stated definition, but the *chain in scope* is not the thing being hidden here. Recommend re-labelling rather than `ambient-context`; a vocabulary-precision note, not a live break. |

## One finding: an unruned, load-bearing global read

`session_ceiling_breach` (`src/rete/kernel/session.rs:1778-1788`) reads `crate::alloc_counter::session_bytes(origin)` — a global, thread-wide allocator byte-counter outside `src/rete/kernel/` — and its result directly decides a hard `Err` return inside the fire loop (`fire/delta.rs:715-727`, called with `origin_key` computed once at `delta.rs:481`) and inside both insert doors (`insert.rs:172-174`, `insert.rs:213-215`, via `check_insert_ceiling`). The `origin` **key** is threaded explicitly and visibly — that part is honest. What is *not* visible in any `FireSession`/`RoundScratch` field is the byte-count value itself: it's read from a global side-channel, and no `rune:sequi` marks the site, despite ~30 lines of prose at `session.rs:1759-1777` and `delta.rs:470-480` that read exactly like a rune's reason field.

Adjudication: I judge the underlying design as defensible under sequi's own carve-outs — allocated-byte count is a resource-governance signal, not RETE domain data the chain operates on, and threading a byte-budget parameter through every allocation site is precisely the "would bloat every signature" case `ambient-context` exists for. But it is **unmarked**, where the codebase's own convention (as practiced at `arm.rs:705`, `census.rs`) is to tag exactly this shape. Recommendation: add `// rune:sequi(ambient-context) — &lt;cite the existing session.rs:1759-1777 rationale&gt;`, rather than leaving a load-bearing hidden-state read unlabelled while sibling global reads in the same crate are.

## Clean by inspection (no findings)

- `outcome.rs` (all 251 lines) — pure conversions, no state.
- `node.rs` (all 230 lines) — pure `Value`-reading helpers, single-step, out of sequi's chain scope.
- `insert.rs` (all 258 lines) — explicit `session: Value → staged_session: Value` threading throughout.
- `fire/rules.rs`, `fire/acc.rs` — grepped for `static|thread_local|OnceLock|RefCell|Cell&lt;`: no hits.
- `session.rs:1430-1628` OnceLock caches and `fire/mod.rs:1671-1677` `#[cfg(test)]` `EMPTY` OnceLocks — memoized referentially-transparent constants derived from static wat source; never mutated, never diverge, don't participate in the fire chain's state thread.
- Full-scope grep for `RefCell|Cell::new|Cell&lt;` outside `census.rs`/`arm.rs`: zero hits. No other interior mutability exists in the 28-file target.
