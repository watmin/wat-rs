# CLARA-TRANSLATIONS — grounded Clara (clara-rules 0.24.0) forms for the arc-278 grid axes

Reference doc for the fleet building `wat-scripts/perf/grid/<axis>.{wat,clj}` (see
`docs/arc/2026/06/278-rules-engine/DESIGN-clara-grid.md`). For each axis: the wat capability + where it
lives, the canonical Clara `.clj` form (quoted from a repo example or from Clara's real source), whether the
two compute the same final derived set, and any semantic caveat that could make an accuracy/speed
differential misleading.

**Grounding sources used:**
- Repo Clara examples: `wat-scripts/perf/clara/gen-bench.sh`, `wat-scripts/perf/matrix/fanout-clara.clj`,
  `wat-scripts/fixes/rete-truth-maintenance-probes/{neg.clj,chain.clj,README.md}`.
- Clara-rules 0.24.0 **actual source**, extracted from the pinned jar
  (`~/.m2/repository/com/cerner/clara-rules/0.24.0/clara-rules-0.24.0.jar`, the same version
  `gen-bench.sh`/the README pin) — `clara/rules.cljc`, `clara/rules/dsl.clj`, `clara/rules/compiler.clj`,
  `clara/rules/accumulators.cljc`, `clara/rules/engine.cljc`. This is authoritative: it is the real
  library code the benchmarks will link against, not documentation paraphrase.
- `clara-tools/` (at `/home/watmin/work/holon/clara-tools`, a separate sibling git repo) was checked — it is
  Cerner's *session-inspector/browser* tooling (`clara.tools.watch`, a web UI), not an API/accumulator
  reference. It contributed nothing to the axes below and is not cited further.
- wat side: `wat/rete.wat`, `docs/arc/2026/06/278-rules-engine/{DESIGN.md,DESIGN-STONE-8-custom.md}`, and the
  differential-test `.rs` fixtures under `tests/rete/` (these pin the exact wat surface syntax used in the
  Clara mapping).

All axes below are **grounded** (none UNVERIFIED) — Clara's own source shipped everything needed.

---

## A2 — asymmetric-arrival joins (derived⋈input, right-before-left)

**(a) wat capability.** `tests/rete/probe_arc278_P6_delta_asymmetric_join.{wat,rs}`. Three scenarios:
- *chain*: `R1: A(?k)→B(?k)`, `R2: B(?k)⋈A(?k)→C(?k)` — insert `A(1),A(2)` — the **right side of R2's join
  (A) arrives in round 1 while the left side (B) doesn't exist yet**.
- *triple cascade*: adds `R3: C(?k)⋈B(?k)→D(?k)` (derived⋈derived).
- *xyz*: `X(?k)⋈Y(?k)→Z(?k)` with all `X` inserted before all `Y` (left-before-right).

This is a real bug class native to wat's **round-based semi-naive delta kernel**: `join_keys`/`right_idx`
were keyed lazily per round, so if a join's right side populated in an earlier round than the left, the
join was skipped and its index was never back-filled (`probe_arc278_P6_delta_asymmetric_join.rs:1-11`: *"If
the RIGHT side of a join (the alpha memory) arrived in an EARLIER round than the LEFT side (the beta/token
memory), the join node J was skipped for every prior round and right_idx[J] was never populated."*). Fixed
by the P6 catch-up: on first keying, rebuild both indices from ALL cumulative memory. `fire-rules` (native)
is now asserted `== fire-rules-spec` (oracle) on all four cases.

**(b) Clara form.** The identical rule shape already lives in the repo as the R18 reference,
`wat-scripts/fixes/rete-truth-maintenance-probes/chain.clj`:
```clojure
(defrule r1 [A (= ?k k)] => (clara.rules/insert! (->B ?k)))
(defrule r2 [B (= ?k k)] [A (= ?k k)] => (clara.rules/insert! (->C ?k)))
```
inserted as `(insert (->A 1) (->A 2))` then `fire-rules`, in any order.

**(c) Same final derived set — yes, with a load-bearing caveat: Clara has no "arrival order" axis at all.**
This is grounded directly in Clara's join-node protocol, `clara/rules/engine.cljc`. `HashJoinNode`'s
`left-activate` immediately joins new left tokens against `(mem/get-elements memory node join-bindings)` —
the **full accumulated right-side memory**, populated by `right-activate`'s own `(mem/add-elements! memory
node join-bindings elements)` (engine.cljc, `HashJoinNode` `ILeftActivate`/`IRightActivate` impls,
~lines 608–654). Symmetrically, `right-activate` joins new right elements against `(mem/get-tokens memory
node join-bindings)`. Both sides always read the *other* side's complete persistent memory — there is no
per-round index that can be "not yet built" the way wat's delta kernel had. **Clara's rete is a mature
per-fact incremental engine (left/right-activate on every insert); wat's is a round-based semi-naive delta
kernel (the whole reason P6 was a distinct bug class).** So: the Clara `.clj` translation is *not* a
distinguishing form for this axis — any insertion order produces the same result in Clara, trivially, by
construction. The value of running A2 against Clara is using Clara as the **ground-truth oracle** for the
now-fixed wat ordering bug (native == Clara, confirming P6 generalizes), not exercising a Clara-specific
mechanism. **STOP-worthy note for the brief:** don't frame A2's speed comparison as "Clara handles
asymmetric arrival better/worse" — frame it only as an accuracy check; the interesting bug was wat-internal
and is closed.

---

## A3 — negation (`:not`)

**(a) wat capability.** `NegationNode` (`wat/rete.wat:109-112`): "*passes a token iff ZERO elements in the
negated alpha-memory are compatible with the token's bindings*". Compiled from `(:wat::rete::not <fact-form>)`
in the rule LHS (`rule-negates`, `wat/rete.wat:1570-1596`, used by the stratifier). Simple (non-derived)
negation is tested in `tests/rete/probe_arc278_7a_negation_oracle.rs` /
`probe_arc278_7b_negation_native_differential.wat`; negation-over-a-**derived** type additionally requires
the stratified fixpoint (`fire-stratified`, `wat/rete.wat:1767-1805`) — see the caveat below.

**(b) Clara form.** Grounded two ways:
1. The docstring for `defrule` itself, `clara/rules.cljc:374-393`:
   ```clojure
   (defrule hvac-approval
     [WorkOrder (= type :hvac)]
     [:not [ApprovalForm (= formname "27B-6")]]
     => (insert! (->ValidationError :approval "HVAC repairs must include a 27B-6 form.")))
   ```
2. The repo's own R18 reference `wat-scripts/fixes/rete-truth-maintenance-probes/neg.clj` (negation over a
   **derived** fact `Bad`, the harder case):
   ```clojure
   (defrule mark-bad [A (= ?k k) (= ?k 2)] => (clara.rules/insert! (->Bad ?k)))
   (defrule ok [A (= ?k k)] [:not [Bad (= ?k k)]] => (clara.rules/insert! (->Ok ?k)))
   ```
   The DSL grammar for `:not` is `clara/rules/dsl.clj:16` (`(def ops #{'and 'or 'not 'exists ...})`) and
   `clara/rules/compiler.clj` `condition-type` (`is-negation (= :not (first condition))`, ~line 660).

**(c) Same final derived set — yes; different mechanism, and the repo's own history proves why the caveat
matters.** `wat-scripts/fixes/rete-truth-maintenance-probes/README.md` records that the ORIGINAL wat
`fire-fixpoint` had a real bug on exactly the `neg.clj` shape: *"negation over a derived fact +
`fire-fixpoint` → Bad=2, Ok=2 (NO dedup — BUG)"* vs Clara's correct `Bad=1, Ok=1`. That bug is **now fixed**
by stratification (`wat/rete.wat:1513-1805`, arc 300 interstitial): wat computes negation-over-derived-facts
correctly by **partitioning rules into strata and running each stratum to its own fixpoint in order**
(`fire-stratified`), so `Ok`'s negated type `Bad` is always fully derived before `Ok`'s stratum runs — a
static, batch, one-shot-per-stratum answer. Clara instead uses **incremental truth maintenance**: `Bad`'s
insert triggers `:not [Bad ...]`'s negation node to *retract* any already-fired `Ok` token for that `?k`
live, no stratum concept, no restart. **Both now converge on the same final set** (`Bad=1, Ok=1` — confirmed
correct on both sides), but the mechanism is structurally different: wat = static ordering + fixpoint restart
per stratum; Clara = dynamic negation-as-retraction via a listener-driven activation network. A benchmark
comparing A3 speed should note this: Clara's incremental TMS pays a per-fact bookkeeping cost that wat's
batch/stratum approach doesn't, but wat's approach re-derives a whole stratum from scratch each round
(semi-naive) rather than truly incrementally — the two are optimizing different things, so a raw
wall-clock ratio without noting *why* is a plausible source of a misleading "we win/lose" claim.

---

## A5 — accumulate / exists (built-in folds; `:exists`)

**(a) wat capability.** `AccumulateNode` (`wat/rete.wat:128-142`) + the `acc::*` fold library
(`wat/rete.wat:1989-2213`): `count`, `sum`, `min`, `max`, `mean` (=Clara's `average`), `distinct`, `all`,
`group-by` (=Clara's `grouping-by`) — **1:1 with Clara's 8 built-ins**
(`docs/arc/2026/06/278-rules-engine/DESIGN.md`: *"8 accumulators (count/sum/min/max/mean/distinct/all/
group-by — 1:1 with Clara's set)"*). `:exists` is `ExistsNode` (`wat/rete.wat:114-126`), documented as "kept
as sugar" over `:not (:not X)` (DESIGN.md keep/cut table) — additive, own node, not literally compiled
through negation twice. Surface: `tests/rete/probe_arc278_8a_accumulate_oracle.rs:64`:
`"(?n <- (:wat::rete::acc::count) :from (:w::Reading (?loc <- :location)))"`.

**(b) Clara form.** `clara/rules/accumulators.cljc` (the real, shipped source) defines exactly these 8 as
top-level fns: `count` (166-170), `sum` (152-161), `min`/`max` (118-134, via `comparison-based`), `average`
(136-150), `distinct` (185-199), `all` (201-213), `grouping-by` (83-99, aliased in wat as `group-by`). The
LHS accumulate syntax, grounded in `clara/rules/dsl.clj:105-127` (`parse-condition-or-accum`: `(second
condition)` is `'from`/`:from`) — canonical form:
```clojure
[?result <- (acc/count) :from [Type (= ?k k)]]
[?total  <- (acc/sum :value) :from [Reading (= ?loc location)]]
```
`:exists` is grounded in `clara/rules/compiler.clj:677-698` (`extract-exists`): *"Converts `:exists`
operations into an accumulator to detect the presence of a fact and a test to check that count is greater
than zero"* — literally rewritten at compile time into
`{:accumulator '(clara.rules.accumulators/exists) :from (second condition) :result-binding (keyword
(gensym "?__gen__"))}`, and `accumulators.cljc:172-183`'s `exists` fn returns `nil` (not `false`) when the
count is zero specifically so *"the accumulator condition will fail to match"* — i.e. Clara's `:exists` is
literally `acc/exists` (a `count`-based accumulator whose `convert-return-fn` suppresses non-positive counts
to `nil`), matching wat's DESIGN.md framing of `:exists` as sugar over presence-detection almost exactly
(wat: sugar over `:not (:not X)`; Clara: sugar over `acc/count` + nil-suppression — same semantic endpoint,
different desugaring target).

**(c) Same final derived set — yes, no material caveat.** Both produce identical bag semantics per fold
(count/sum/min/max/mean/distinct/all/group-by all have the same mathematical definition over the joined
element set on both sides); `:exists`/`ExistsNode` and Clara's `acc/exists` both fire at most once per
compatible-token regardless of match multiplicity. One minor asymmetry worth noting in the brief but not
disqualifying: wat's accumulate re-gathers and re-folds the whole compatible-element set on every fire
(`AccumulateNode` docstring, `wat/rete.wat:131`: *"Pure replay: re-accumulates on every fire (no retract-fn
needed)"*), whereas Clara's accumulators carry an explicit `:retract-fn` (`accumulators.cljc:19,143-150,
159-161,169-170,193-198,208,213`) used for **incremental** update when a contributing fact is retracted —
same final answer at rest, different amount of recomputation, which is exactly the axis a speed
differential should surface (and is *not* an accuracy risk).

---

## A9 — LEADING `:exists` read through a query, across a multi-round fixpoint

**Why this axis exists at all.** Added 2026-08-24, after a full vigilia found that wat's
LEADING (parentless) `:not`/`:exists` emitted one token **per fixpoint round** into the
cumulative beta. A query over such a rule returned `rounds x locs` rows where `locs` is
correct — exactly, at every chain length measured (2→2, 3→3, 4→4, 6→6).

**It was never a differential failure. Both references were right.**
- **Clara** does not share the flaw. Measured on 0.24.0: a leading `[:exists [Wind (= ?loc loc)]]`
  activates once, with an unrelated cascade running.
- **The wat `$oracle`** does not share it either, and is immune **by construction**:
  `fire-once$oracle` (`wat/rete/oracle/fire.wat:131`) rebuilds `new-amem` and `new-bmem` from an
  **empty** `PersistentMap` on every fire. It has no cumulative memory for a re-emission to
  compound into. The native engine accumulates for speed — and accumulation is exactly what
  turned a per-round re-emission into a duplicate.

So the machinery would have caught this instantly; **the corpus had no case of this shape.**
Three properties must ALL hold or the defect hides again: the filter must be **leading** (a
mid-chain one is fed by its parent's delta and was never wrong), it must be observed **through a
query** (`production_delta` dedups derived facts by value and masks token multiplicity), and the
fixpoint must run **more than one round** (at one round, "once per fire" and "once per round" are
the same number).

**(a) wat form.** `(:wat::rete::defquery ... :when [(:wat::rete::exists (:lx::Wind (?loc <- :loc)))])`
— one token per DISTINCT inner binding, so two `Wind` at one loc yield one `{?loc}`.

**(b) Clara form.** `(defquery q-exists [] [:exists [Wind (= ?loc loc)]])` — verified to bind
`?loc` outward and yield one row per distinct loc (5 Winds over 3 distinct locs → 3 rows,
`{:?loc "A"}`). Clara's `:exists` is implemented over accumulators, so `clara.rules.accumulators`
must be loaded or the run dies with `ClassNotFoundException` rather than a wrong answer.

**(c) Equivalence.** Same semantics, same witness — the sorted distinct locs, comparable
byte-for-byte. Verified against the pre-fix engine before landing: it produced each loc six times
(36 entries for 6 locs) against the oracle's 6, failing on **both** `:accuracy` and
`:port-accuracy`. This axis can fail, which is the only thing that makes it a gate.

**(d) The `:not` arm.** Deliberately not a separate axis. Clara rejects `[:not [Ghost (= ?k k)]]`
outright (*"Unbound variables: #{?k}"*) and accepts only the unbound `[:not [Ghost]]`, so a
leading-`:not` axis could witness a bare count where this one witnesses the whole set. Both arms
share one code path and one fix, and both are gated on every floor by
`tests/rete/probe_arc278_leading_filter_multiplicity`.

## A6 — user reducers / custom accumulator (`(PV<T>) -> R`; percentile/stddev/top-k)

**(a) wat capability.** `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-8-custom.md` + the differential
`tests/rete/probe_arc278_8custom_native_differential.rs`. Contract: the accumulate slot accepts **any user
wat fn `(PV<T>) -> R`** as the acc-form head when it isn't one of the 8 built-ins; the dispatcher gathers the
bound `?var` values into a `PersistentVector<T>` and evaluates the user fn over that vector
(`accumulate-pass-for-token`'s "8-custom" arm, `wat/rete.wat:2272-2289`). Gate: the fn must be **pure ∧
deterministic** (the same 6a fence `where`/`:test` use), checked at compile — an impure fold is rejected
(proved by `fence_rejects_impure_fold`, `probe_arc278_8custom_native_differential.rs:96-125`). Concrete
example from the repo's own test (`probe_arc278_8custom_native_differential.rs:19-42`):
```
(:wat::core::defn :w::sum-of-squares [xs <- :wat::core::PersistentVector<wat::core::i64>] -> :wat::core::i64
  (:wat::core::foldl (:wat::core::fn [acc <- :wat::core::i64  x <- :wat::core::i64] -> :wat::core::i64
    (:wat::core::i64::+ acc (:wat::core::i64::* x x))) 0 xs))

(:wat::rete::defrule :w::flag
  :when [(:w::Station (?loc <- :location))
         (?s <- (:w::sum-of-squares ?v) :from (:w::Reading (?loc <- :location) (?v <- :value)))
         (:wat::rete::where (:wat::core::= ?s 14))]
  :then (:wat::rete::insert (:w::Flagged ?loc)))
```

**(b) Clara form.** Grounded in `clara/rules/accumulators.cljc:8-36`, the `accum` fn (DESIGN.md notes wat's
naming choice explicitly against it: *"Custom constructor `acc/accumulator` (NOT Clara's truncated `accum`,
a Level-2 mumble)"*):
```clojure
(require '[clara.rules.accumulators :as acc])

(def sum-of-squares
  (acc/accum {:initial-value 0
              :reduce-fn (fn [total value] (+ total (* value value)))
              :combine-fn +
              :convert-return-fn identity}))

(defrule flag
  [Station (= ?loc location)]
  [?s <- sum-of-squares :from [Reading (= ?loc location) (= ?v value)]]
  [:test (= ?s 14)]
  => (insert! (->Flagged ?loc)))
```
`accum`'s accepted keys, per the docstring (`accumulators.cljc:8-22`): *"An initial-value ... A reduce-fn ...
An optional combine-fn ... An optional retract-fn that can remove a retracted fact from a previously reduced
computation ... An optional convert-return-fn"*. `reduce-to-accum` (`accumulators.cljc:47-81`) is the
closer wat-shaped sibling — a single reduce fn over a running value, no explicit initial/combine/retract
required — but it is still a *streaming reduce* (one item at a time via `reduce-fn`), not a batch fold over
a materialized vector.

**(c) Same final derived set — yes, but the two custom-accumulator SHAPES are genuinely different, not just
differently named — flag this for the brief.** wat's custom fold signature is `(PersistentVector<T>) -> R`:
a **pure batch fold over the whole gathered vector**, re-run on every fire (no incrementality, no
`retract-fn` — DESIGN-STONE-8-custom.md: *"a non-total fold only hangs the local single-user engine"* is the
only safety axis, not incrementality). Clara's `accum`/`reduce-to-accum` signature is `(reduce-fn [acc item]
-> acc)` **plus optionally `retract-fn`/`combine-fn`**: a **streaming, incremental, per-item reducer**
designed so a single fact insert/retract updates the aggregate in O(1) rather than O(n) re-fold. For a
fold that is expressible either way (sum-of-squares, count, sum) the **final value is identical** — this is
the case the differential should assert. But **percentile/top-k/mode are not naturally incremental
per-item reducers** (a p95 needs the whole sorted sample; `average`'s own `accumulators.cljc:136-150` shows
even Clara resorts to a `[running-sum count]` pair rather than true streaming percentile math) — so for
those specific folds, the *Clara-idiomatic* form is still batch-flavored under the hood (an `accum` whose
`reduce-fn` conses onto a list and whose `convert-return-fn` does the percentile math at read time), which
converges with wat's `(PV<T>) -> R` shape almost exactly. **Caveat for the brief:** when translating a
specific user fold (e.g. p95), don't force it into Clara's `reduce-fn`/`retract-fn` incremental slots if the
math isn't incrementally decomposable — write the equivalent "collect-then-compute" `accum` (`:reduce-fn
conj`, `:convert-return-fn percentile-math`) so the two sides are computing the SAME thing (a full-population
percentile), not two different approximations.

---

## A7 — minimum-finding-set ("≥N findings to activate")

**(a) wat capability.** `AccumulateNode` (count) + a `where`/`:test`-style threshold predicate, exercised
directly by the repo's own differential test, `tests/rete/probe_arc278_8b_accumulate_native_differential.rs:
59,88-93`:
```
const COUNT: &str = "(?n <- (:wat::rete::acc::count) :from (:w::Reading (?loc <- :location)))";
// "4 — DIFFERENTIAL the minimum-finding-set composition: count >= 3 fires with 3, blocks with 2."
diff(COUNT, "(:wat::core::>= ?n 3)", &[("Oslo", 1), ("Oslo", 2), ("Oslo", 3)], 1);
diff(COUNT, "(:wat::core::>= ?n 3)", &[("Oslo", 1), ("Oslo", 2)], 0);
```
i.e. `AccumulateNode(count) → TestNode(?n >= 3) → ProductionNode`. This is exactly the DDoS "N findings to
activate" primitive named in the grid DESIGN (`DESIGN-clara-grid.md:44`).

**(b) Clara form.** Same accumulate LHS grammar as A5/A6, composed with a `:test` condition — grounded in
`clara/rules/dsl.clj:139-144` (`:test`/`'test` parses to `{:constraints (vec (rest expression))}`) and the
`condition-type` dispatch in `compiler.clj:656-675` (a condition with no `:type`/`:accumulator`/`:not`/
`:exists` key falls through to `:test` — `:else :test`):
```clojure
(defrule flag-repeat-offender
  [Station (= ?loc location)]
  [?n <- (acc/count) :from [Reading (= ?loc location)]]
  [:test (>= ?n 3)]
  => (insert! (->Flagged ?loc)))
```

**(c) Same final derived set — yes, no caveat beyond A5's.** This axis is a straight composition of two
already-grounded primitives (A5's `acc/count` + a `:test` predicate over the bound accumulator result); both
engines gate the SAME boolean (`count(matching-facts) >= N`) before firing, so the derived-set comparison is
exact. The only note carried over from A5/A3: Clara can retract a contributing `Reading` and have the count
(and hence the gate) update incrementally via `retract-fn`; wat recomputes the gather+fold+test on the next
full fire. For a **grid** benchmark (fire-once on a fixed fact set, no interleaved retraction), this
difference doesn't surface at all — both sides compute the same gate against the same static snapshot.

---

## A8 — node-sharing / rule-count (many rules, shared join-prefix)

**(a) wat capability.** `CompileState.dedup: HashMap<String,i64>` (`wat/rete.wat:388-397`, doc comment:
*"maps a structural key to the existing node id; avoids rescanning the network to detect shareable nodes"*).
Concretely:
- `find-or-mint-alpha` (`wat/rete.wat:422-451`): dedup key `"alpha:<write-forms cond>"` — two rules with a
  structurally-identical leading condition (canonicalized via `write-forms`, span-agnostic) get **one**
  shared `AlphaNode`.
- `find-or-mint-hash-join` (`wat/rete.wat:483+`): dedup key `"hashjoin:<parent-id>:<cond-text>"` — a join
  is shared only if **both** the condition text AND the parent node id match, i.e. rules sharing a
  join-prefix (not just the same condition floating at a different position) collapse onto the same beta
  subtree. Only `ProductionNode`s are never shared (`compile-rule`, `wat/rete.wat:781`: *"mint the
  ProductionNode (never shared — one per rule)"*).

**(b) Clara form.** No special DSL — sharing is automatic compiler behavior, grounded directly in Clara's
compiler:
- **Alpha sharing**: `clara/rules/compiler.clj:1724-1761`, `to-alpha-graph`'s `condition-to-node-map`
  (comment at 1734: *"Merge common conditions together."*) — `(reduce (fn [node-map [[condition env]
  node-id]] (if (get node-map [condition env]) (update-in node-map [[condition env]] conj node-id) (assoc
  node-map [condition env] [node-id]))) {} condition-to-node-ids)`: every beta node requiring the same
  `[condition env]` pair is fanned out from ONE compiled alpha node (`:beta-children (distinct node-ids)`).
- **Beta/join-prefix sharing**: `clara/rules/compiler.clj:1113-1208`, `add-conjunctions`'s `update-node->ids`
  / `node-id` resolution (~1168-1197): a new condition node is assigned an **existing** id iff an
  identical node already exists as a child of the identical **parent-id set**
  (comment at 1188-1194: *"We need to validate that the node we intend on sharing shares the same parents as
  the current node we are creating. See Issue 433"*) — otherwise `(create-id-fn)` mints a new node. This is
  structurally the same two-part key wat uses (condition identity + parent identity), just implemented as a
  map-of-maps instead of a single dkey string.
- Benchmark form (no new syntax — just N rules with a common leading condition, mirroring
  `wat-scripts/perf/clara/gen-bench.sh`'s generator style):
  ```clojure
  (defrule r1 [Common (= ?k k)] [Unique1 (= ?k k)] => (insert! (->Out1 ?k)))
  (defrule r2 [Common (= ?k k)] [Unique2 (= ?k k)] => (insert! (->Out2 ?k)))
  ;; ... rN, all sharing the leading [Common (= ?k k)] condition
  ```

**(c) Same final derived set — yes; this axis is fundamentally a NODE-COUNT / activation-cost question, not
an accuracy one.** Both engines are proven (by their own compilers' dedup mechanisms, not by inference) to
collapse a shared leading condition into one alpha node + one root-join, fanning out to N per-rule
continuations — so the *derived fact set* is identical by construction on both sides regardless of rule
count; there's no scenario where sharing changes semantics (it's a compile-time subtree-reuse optimization
under an otherwise-identical logical network). The actual interesting measurement is: does the **per-fact
activation cost** for N rules sharing a prefix grow O(1) (shared prefix, wat and Clara both claim this) or
O(N) (naive N-rule scan)? That is a speed/scaling question the grid should report as a ratio-vs-N curve, not
a pass/fail accuracy row. **STOP-worthy note for the brief:** don't build an "accuracy" assertion for A8 at
all beyond confirming both produce the same `:derived` set once — the axis's entire point is the speed curve
as rule-count N grows with a fixed shared prefix depth.

---

## Summary for the orchestrator

| axis | grounded against | accuracy caveat for the brief |
|---|---|---|
| A2 | `engine.cljc` HashJoinNode activate protocol + repo `chain.clj` | Clara has no "arrival order" axis — frame as accuracy-only, not a Clara-vs-wat mechanism comparison |
| A3 | `rules.cljc` defrule docstring + repo `neg.clj`/README (documents the historical wat bug, now fixed by stratification) | same final set; wat=static stratified fixpoint restart, Clara=incremental TMS retraction — note the mechanism difference when interpreting a speed ratio |
| A5 | `accumulators.cljc` (all 8 folds) + `compiler.clj` `extract-exists` | none material; Clara's accumulators carry `retract-fn` (incremental) wat's don't (pure re-fold each fire) — a speed, not accuracy, distinction |
| A6 | `accumulators.cljc` `accum`/`reduce-to-accum` + `DESIGN-STONE-8-custom.md` + repo differential test | shapes differ (wat: batch fold over PV<T>; Clara: streaming reduce+retract) — for non-incrementally-decomposable folds (percentile/top-k), write Clara's `accum` in collect-then-compute form so both sides compute the identical population statistic |
| A7 | `dsl.clj` `:test` parse + repo differential test (literal `>= ?n 3` case already in the codebase) | none; straight composition of A5 + a boolean gate on a static snapshot |
| A8 | `compiler.clj` `to-alpha-graph` (alpha sharing) + `add-conjunctions` (beta/join-prefix sharing) | this is a speed/node-count axis, not an accuracy axis — don't build an accuracy gate beyond a single derived-set sanity check |

No axis required marking UNVERIFIED — Clara's 0.24.0 source (extracted from the pinned jar) plus the
repo's own existing Clara examples and wat differential tests fully grounded all six forms above.
