# Arc 278 — explicit work queue (real names, tractability-ordered)

## ⛔ NOT THE BREADCRUMB — historical, 2026-06-21 (superseded 2026-08-25)

> **STOP. This section is NOT the current state, and it used to claim it was.** It was headed
> "CURRENT STATE — read first (breadcrumb)" until 2026-08-25, which made TWO files in this arc
> each announce themselves as the one live breadcrumb — the exact "breadcrumb forks" failure
> `curare` names, where the next self reconstructs the present from a pile and trusts the wrong
> stratum. This one is over two months stale: it says 278 is PARKED, and 278 has been the active
> arc through 55+ commits since.
>
> **The single live breadcrumb is
> `docs/arc/2026/06/278-rules-engine/CURRENT-STATE-annihilate-interpretation.md`.** Read that.
> Everything below is kept as history because the collection-campaign detail is still accurate
> ABOUT ITS OWN PERIOD — but nothing below is a statement about now.

**⛔ [HISTORICAL 2026-06-21] 278 IS PARKED — PIVOTED TO ARC 255 (builtin registry). See `docs/arc/2026/06/255-builtin-registry/DESIGN.md`.**
Building the collection campaign surfaced a **catastrophic checker-soundness hole**: the resolver
blanket-accepts ANY `:wat::*` head (`is_reserved_prefix → true`) and the checker punts via a permissive
`Infer` fallback — so a typo'd/retired/nonexistent builtin (`:wat::core::nonexistent-xyz?`) type-checks
clean and only dies at runtime. Builder verdict: annihilation. **255 closes it** (builtins become
first-class `sym`-registered, reflectable, with a forced minimum-baseline + adjacent per-def-kind record),
**and is the vehicle to carve `runtime.rs`'s ~483 dispatch arms into namespaced homes.** 255 UNLOCKS 278's
continuity — the `List?`→`ast-list?`/`list?` split, retirement-loud-at-resolve, the container-predicate
family, and the collection HOF fills (1c WatAstList / 1d HashSet) all resume on the sound substrate.

**COLLECTION CAMPAIGN (parked mid-flight — resume after 255):** lookup/size done both families; seq HOFs:
1a (cap split `5ac9abdb`) + 1b (List `751d131d`) DONE; **remaining 1c (WatAstList HOFs+conj), 1d (HashSet
map/filter/fold), then map-iteration, index-assoc, set algebra.** Spec/grid: `docs/COLLECTION-CAPABILITIES.md`.
Ethos: the substrate forces our hand — no deferral, satisfy forcing-signals by USE not `#[allow]`/`pub`
(memory `feedback_substrate_forces_idealized_state`).

**✅ LOOKUP/SIZE COMPLETE — both families** (tree clean + pushed; `git log --oneline -8` for the collection
commits; floor lib 953/36/1, warnings 26).
`assoc` + `get`/`contains?`/`length`/`empty?` route through MapContainer (HashMap/PersistentMap/Record) AND
SeqContainer (Vector/PV/List/Tuple/WatAstList/HashSet) via genuine `if c.CAP()` gates — every cell `done` or
grounded-`N/A` (only Tuple/get+assoc, HashSet/concat are N/A). Shipped as: map waist `f4beda7d` + A2 `361788a1`
(Record) + seq-1a `76ebd62c` (route) + seq-1b `7550310f` (fill). Two silence-the-signal cheats (a pub-leak +
debug_assert-shadows) were caught in the weigh and fixed to genuine gates.

**Build order — remaining (then resume rete):**
1. **seq HOF fills (NEXT)** — flip `mappable` for List/WatAstList/HashSet; build/route map+filter+foldl+foldr for
   List, WatAstList, HashSet(set→set); reverse/take/drop/concat for List + WatAstList. (Tuple HOFs = ∅N/A.)
2. **map/filter/fold over maps → Vec** — eager entry-iteration (HashMap/PersistentMap/Record); no lazy-seq needed.
3. **index-assoc** — `assoc`-by-index on Vector/PV/WatAstList (homogeneous, bounds-checked); Tuple/List = N/A.
4. **set algebra** — new verbs `union`/`intersection`/`difference` on HashSet.
Then collections sane → resume the accumulator cluster (§1) = custom-accumulators → returns-the-fact →
field-sugar → acc/-alias. Pattern for each: mirror the lookup/size fills (flip cap → build/route → checker →
black-box probe → ProbeDummy recon → floor held → **sync the grid cell in the same weigh** → commit+push).
Grid: `docs/COLLECTION-CAPABILITIES.md`.

> ⚠️ **WEIGH WARNING (proven 3× this campaign):** sonnets keep trying to silence the forcing-signal — making a
> registry `pub`, `debug_assert!(m.cap())` instead of the genuine `if m.cap()` gate, `_ => unreachable!()`
> catch-alls. AND the harness's live diagnostics LAG (stale `dead_code`/`ProbeDummy` after a strike finishes).
> Trust neither the SCORE nor the diagnostics — VERIFY THE DISK every weigh: `grep` the gate is `if m.CAP()`,
> `grep` helper call-sites are wired, `cargo build` warning count, re-run the floor. See memory
> `feedback_substrate_forces_idealized_state`.

> ⛔ **You are a NEW instance.** You did not live the above; it's a cache. Run **recolligere** against the disk
> before acting: `git status` + `git log --oneline -15` + read `docs/COLLECTION-CAPABILITIES.md` (the grid is the
> truth) before any move. The campaign is mid-flight — orient, don't assume.

---

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

> ⛔ **STANDING ORDER (2026-06-20): COLLECTIONS BLOCK RETE.** Building rete exposed how unsound the collection
> surface is. The accumulator cluster below (custom accumulators, returns-the-fact, field-sugar, acc/-alias) is
> **BLOCKED** until the collection capability grid is all `done`/`N/A`. Authoritative spec + queue:
> **`docs/COLLECTION-CAPABILITIES.md`**. Order: (1) map waist complete, (2) seq waist complete, (3) index ops,
> (4) set algebra (union/intersection/difference), (5) HashSet/get. Then resume the accumulator cluster. No
> deferral — every cell ships `done` or grounded-`N/A`; lazy-seq is the one bounded future-type, not a gap.

**Coherence prerequisites (surfaced 2026-06-20 while de-risking returns-the-fact):**
- ✅ **seq/collection container drift — DONE** (`75356ecc`). The checker false-rejected `first`/`second`/
  `third`/`rest` on PersistentVector + WatAST and `conj` on List (one-sided changes across arcs 220/249/
  278-0b — runtime built, `check.rs` half skipped under the misread megafile guard). All false-REJECT, fixed
  checker-side; `tests/probe_seq_container_parity.rs` pins checker≡runtime. This was the likely cause of prior
  unexplained sonnet thrash (the checker's error message lied about the accepted set).
- ✅ **seq-container registry strikes 1-3 — DONE.** The narrow-waist home `src/collection/seq_container.rs`:
  `enum SeqContainer {Vector,List,PersistentVector,Tuple,WatAstList,HashSet}` + `of_type`/`of_value` classifiers
  + capability methods (`indexable`/`has_tail`/`has_append`/`mappable`). Strike 1 routed positional accessors
  (`5da88139`); strike 2 `rest`+`conj` (`21543cef`); strike 3 the HOF family (`534171ea`). `first`/`rest`/
  `conj`/`map`/`filter`/`fold`/… now derive their accepted set from one capability table, both sides.

- **THE NARROW WAIST IS NOT DONE — the guarantee is incomplete.** The goal (R14 / builder): *the next primitive
  we introduce isn't allowed a partial/wrong impl* — drift **unrepresentable**, not merely caught. Two holes,
  both grounded 2026-06-20, mean a new container can still be added half-wrong with NO compile error:
  - **Coverage hole.** `get`/`contains?`/`length`/`empty?` do NOT route through the waist, and there is **no
    MapContainer registry at all**. A new seq container is silently absent from these four (no compile error;
    `length`/`empty?` are `∀T` at the checker so they even type-check); a new MAP type (the persistent-map that
    started this) has zero structural forcing.
  - **Depth hole.** Even the *routed* ops use a unit enum + an inner `match &Value { … _ => unreachable!() }`
    (`runtime.rs:11012, 11015, 12454`). Adding a variant forces classification + capability declaration but
    **NOT the inner behavior arm** — a `has_append`-true variant with no inner arm compiles clean and panics at
    runtime. `conj` itself can be given a partial impl today. The waist forces "is-it / what-can-it-do," not
    "do-it"; the last mile is a runtime `unreachable!`, not a compile error.

  **Decomposition (dependency order; the guarantee holds only when all three land):**
  - [x] **Strike 4 — depth fix — DONE** (`c70b2733`, 2026-06-20). Inner dispatch now `match container` over the
    closed `SeqContainer` enum, exhaustive, no `_` (Form 1: explicit named-helper arms, decided over Form 2 after
    an architecture audit — Pattern A confirmed, traits/`defprotocol` rejected for these type-projective
    intrinsics). 11 dispatch sites retrofitted (runtime.rs ×2, eval.rs ×2, transform.rs ×7). Proven: adding a
    throwaway enum variant now errors at all 11 dispatch sites + 4 capability methods (was: 4 only). Floor held
    941/36, warnings 26, behavior byte-identical. Strikes 5+6 inherit this pattern.
  - [ ] **Strike 5 — MapContainer registry (NEXT).** Mint the sibling keyed registry for
    `{HashMap, PersistentMap, Record}` mirroring `SeqContainer` (exhaustive enum + `of_type`/`of_value`
    classifiers + capability methods + strike-4 exhaustive `match container` dispatch). Route the map-only op
    `assoc` through it. Standalone, independently green — the missing dependency the mixed-op routing needs.
  - [ ] **Strike 6 — route the mixed ops through BOTH waists.** `get`/`contains?`/`length`/`empty?` route their
    seq arms through `SeqContainer` and their map arms through `MapContainer` in ONE pass — symmetric, exhaustive
    on both sides, each op touched once. Closes the coverage hole; the guarantee then holds for seq AND map.
  - *(Reordered 2026-06-20, was: seq-half-then-MapContainer. Four-questions: building MapContainer first lets the
    mixed ops route both halves in one pass — no asymmetric seq-routed/map-hand-rolled interim, each mixed op
    edited once not twice. The mixed ops have no live drift today, so the interim order is a free choice.)*
  - **End state:** adding any container (seq or map) lights up compile errors at classification + capability +
    every op's exhaustive arm → a partial impl cannot be written. The `tests/probe_seq_container_parity.rs` drift
    probe guards the class in the interim. Supersedes the old "extend `extract_seq_elem`" framing (too narrow —
    it addressed only the checker element-type, not the depth hole or the map family).

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
