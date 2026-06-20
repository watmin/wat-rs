# Arc 278 — explicit work queue (real names, tractability-ordered)

The single source for "what's left and in what order." Real names, not codenames. Each entry: what it actually
is · how tractable · status · old codename (so prior notes still map). Supersedes the scattered "banked `8-x`"
notes across DESIGN-STONE-* — if it's not here, it's not queued.

**THE ORDER IS FIXED — not reorderable:**

> ## ①  FEATURE PARITY  →  ②  FEATURE PERFORMANCE SUPERIORITY  →  ③  GRIMOIRE

1. **FEATURE PARITY** = implement *every feature we've chosen to ship* (our chosen set, complete — NOT
   full-Clara-parity; we deliberately cut things). This is the pending queue below (§1).
2. **FEATURE PERFORMANCE SUPERIORITY** = only once the feature set is *complete*, measure it and beat Clara
   across it (§2). **Perf is NOT next** — you cannot measure or optimize a feature set that is still growing.
3. **GRIMOIRE** = the `kernel.rs` vigilia ward, **last**, on *final* code (§3) — you cannot ward code that is
   still changing.

278 closes at the end of ③. Within Phase ① work in tractability order. §4 is the payoff after close; "Deferred
for real" and "Horizon" are NOT in any phase.

---

## ①  FEATURE PARITY — the features we've chosen to implement (the pending queue)
*The accumulator-completion cluster — all ride stone 8's machinery. Worked in tractability order: capability
first, ergonomics last. **This entire phase must complete before ② Perf begins.***

**Coherence prerequisites (surfaced 2026-06-20 while de-risking returns-the-fact):**
- ✅ **seq/collection container drift — DONE** (`75356ecc`). The checker false-rejected `first`/`second`/
  `third`/`rest` on PersistentVector + WatAST and `conj` on List (one-sided changes across arcs 220/249/
  278-0b — runtime built, `check.rs` half skipped under the misread megafile guard). All false-REJECT, fixed
  checker-side; `tests/probe_seq_container_parity.rs` pins checker≡runtime. This was the likely cause of prior
  unexplained sonnet thrash (the checker's error message lied about the accepted set).
- **QUEUED follow-on (not deferred-vaguely) — structural single-source-of-truth for the container set.** Route
  the positional/rest/conj checker arms through one shared element-extractor (extend `extract_seq_elem`,
  collection/infer.rs:500, today only `{Vector, PersistentVector}`; Tuple handled explicitly) so a new container
  repr added once reaches all consumers and a one-sided arm becomes UNREPRESENTABLE (top of the extirpare
  ladder). Genuine design (per-op-family container subsets differ; Tuple is heterogeneous) → its own DESIGN +
  strike. The drift probe guards the class in the interim. See `DESIGN-STONE-seq-container-drift.md` "Out of scope".

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

## ②  FEATURE PERFORMANCE SUPERIORITY — only after ① is COMPLETE

*You cannot measure or optimize a feature set that is still growing. ① must be done first.*

5. **Rule-count / shared-prefix perf measurement vs Clara** — the last unmeasured Clara cell, **plus**
   re-confirm we beat Clara across the now-complete feature set (we added where/:not/:exists/accumulators
   since the last measurement). Beta/join-prefix sharing is already BUILT (`rete.wat:481`); this MEASURES it
   holds at high rule count + probes the syntactic-vs-semantic-sharing nuance. DESIGN:
   `DESIGN-STONE-P7b-rulecount-sharing.md`. Harness = sonnet; the grid run = orchestrator-only. Any cell where
   we don't yet win → an iterate-stone (still inside this phase, until we're superior). *(codename `②`)*

## ③  GRIMOIRE — last, on FINAL code

*You cannot ward code that is still changing. ① and ② must be done first.*

6. **`kernel.rs` grimoire / vigilia — the final-code ward.** The "beat the shit out of it" pass. Includes the
   dead-code purge below. *(codename `⑤`)*
   - **6a. Remove the dead `QueryNode`** — the `QueryNode` record + `:QueryNode` enum variant are defined but
     never minted or executed (query reads production-memory directly). The vigilia's purgare ward catches it.

→ **278 CLOSES** here (features complete → measured-superior → code warded).

---

## Deferred for real — NOT in any phase (NOT cheap — honest reasons, not near-term)

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

## The payoff — after 278 closes (NOT in any phase)
- **The reborn linter — lint rules as rete rules.** The engine's first serious app and the *why* of arc 278:
  rewrite the linter's if/cond rules as maintainable rete rules. The capability (where/:not/:exists/
  accumulators) is already enough; this is the application build.

---

## Horizon (NOT this arc — see `NOTE-overlay-read-path-and-distributed-horizon.md`)
The persistent-WM service (⑥), the LMDB/durable backend, snapshot+journal replication, the metered-fire +
`total?` resource governor, the rules-as-a-service / eBPF-for-application-rules thesis, the distributed
deployment + its 5 edge-hardening items. All captured, all horizon, none queued here.
