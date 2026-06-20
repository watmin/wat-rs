# Arc 278 — explicit work queue (real names, tractability-ordered)

The single source for "what's left and in what order." Real names, not codenames. Each entry: what it actually
is · how tractable · status · old codename (so prior notes still map). Supersedes the scattered "banked `8-x`"
notes across DESIGN-STONE-* — if it's not here, it's not queued.

**Worked top-to-bottom in tractability order. Capability before measurement before the final ward.**
278's *close* needs only the **Perf measurement + the grimoire ward** (§2). Everything in §1 is cheap,
genuinely wanted capability-completion (the anomaly domain) that we're choosing to finish first so the ward
sees final code. §3 is honestly-hard and stays deferred. §4 is the payoff after close.

---

## §1 — Cheap capability completion (the accumulator cluster — all ride stone 8's machinery)
*Build these first, in this order: capability, then ergonomics.*

1. **Custom accumulators** — let the accumulator slot take *any* user-supplied `pure ∧ deterministic ∧ total`
   fold fn over the gathered set, not just the 8 built-ins. **This is the percentiles / stddev / top-k
   unlock** — the thing we argued we need for the anomaly domain. One strike: generalize the accumulate
   dispatch (known head → built-in fast-path; else eval the user fn over the gather, reusing eval-test);
   compile-time fence rejects impure/non-total. *(codename `8-custom`)* — **one strike, high value, NEXT.**

2. **Accumulator returns-the-fact** — `min`/`max` (and friends) return the *fact* that won, not just the
   value (argmin / argmax). The gather already carries the facts (`element_fact_bindings`; `all`/`group-by`
   already return facts), so it's a fold variant that tracks the winning element. *(codename
   `8-returns-fact`)* — **small.**

3. **Accumulator field shorthand** — `(acc/sum :size)` instead of `(?v <- :size) … (acc/sum ?v)`. A
   defrule-macro expansion over the positional `?var` form (kwargs-is-a-macro; the arc-249 macro engine
   exists). *(codename `8-field-sugar`)* — **small (a macro), ergonomics only.**

4. **Accumulator `acc/` alias** — `acc/` shorthand for the `:wat::rete::acc::` FQDN. Pure naming sugar.
   *(codename `8-sugar`)* — **trivial, ergonomics only.**

---

## §2 — The 278 close path (the only items the close actually requires)

5. **Rule-count / shared-prefix perf measurement vs Clara** — the last unmeasured Clara cell. (Beta/join-
   prefix sharing is already BUILT — `rete.wat:481`; this MEASURES that it holds at high rule count + probes
   the syntactic-vs-semantic-sharing nuance. DESIGN: `DESIGN-STONE-P7b-rulecount-sharing.md`.) Harness =
   sonnet; the grid run = orchestrator-only. *(codename `②`)* — measurement, not a build.

6. **`kernel.rs` grimoire / vigilia — the final-code ward.** Cast LAST, on final code (so it must follow §1).
   Includes the dead-code purge below. *(codename `⑤`)* — the "beat the shit out of it" pass.
   - **6a. Remove the dead `QueryNode`** — the `QueryNode` record + `:QueryNode` enum variant are defined but
     never minted or executed (query reads production-memory directly). The vigilia's purgare ward catches it.

→ **278 CLOSES** here (perf done + measured, capability complete, code warded).

---

## §3 — Deferred for real (NOT cheap — honest reasons, not near-term)

- **Negation / exists over *derived* facts (stratified)** — our `:not`/`:exists` are complete for *base*
  facts; negating a *derived* fact needs **stratification** (only evaluate the negation once its negated input
  is stable). Real semantics work, not a one-strike. *(codename `7-strat`)*
- **Incremental accumulation (retract-fn)** — update an aggregate in O(delta) on support loss instead of
  re-folding. Perf only, **and replay dissolves the need** until a hot persistent working memory exists.
  *(codename `8-perf`)*
- **Inline-constraint join fusion (ExpressionJoinNode)** — fuse a cross-condition predicate into the join
  instead of a separate TestNode. Perf optimization; the TestNode already works correctly. *(codename
  `6b-perf`)*
- **Native Rust accumulator folds** — move the wat folds to Rust for speed; measured follow-on. *(codename
  `8-perf-folds`)*
- **Leading negation** (`:not` as the first condition, no left token) — needs a synthetic root token. Small
  but a real edge case, not free.

---

## §4 — The payoff (after 278 closes)
- **The reborn linter — lint rules as rete rules.** The engine's first serious app and the *why* of arc 278:
  rewrite the linter's if/cond rules as maintainable rete rules. The capability (where/:not/:exists/
  accumulators) is already enough; this is the application build.

---

## Horizon (NOT this arc — see `NOTE-overlay-read-path-and-distributed-horizon.md`)
The persistent-WM service (⑥), the LMDB/durable backend, snapshot+journal replication, the metered-fire +
`total?` resource governor, the rules-as-a-service / eBPF-for-application-rules thesis, the distributed
deployment + its 5 edge-hardening items. All captured, all horizon, none queued here.
