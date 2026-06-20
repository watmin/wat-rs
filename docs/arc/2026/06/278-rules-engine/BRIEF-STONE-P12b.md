# BRIEF — Stone P12b: the `explain` walk (in WAT) → greens the EXPLAIN north-star

**Executor:** one **sonnet** Shadowdancer. **No sub-agents. No `git`. No worktrees.** Do NOT run
`./target/release/wat` (orchestrator-only). `cargo test` is yours. The walk lives in **WAT** (Decision A,
DESIGN-STONE-P12) — builder-readable; do NOT write it in Rust.

## Scope (the b/c split — read this first)
P12b builds the **structural** why-tree: the recursive walk producing a node-only `Why {fact, via}`. It greens
the P12 north-star (which asserts only `Why/via` counts). The **rich payload** — the rule name, the bound vars,
and `:met` (the satisfied conditions with concrete values) — and the node/edge distinction are **P12c**, with
their own probe. Do NOT build `:met`/`:bound`/`WhyVia` here. (Grounded reasons: `:met` is a per-EDGE payload
needing the edge type; the node-only self-recursive `Why` needs no Option construction and no mutual recursion,
both verified-risky, so P12b stays minimal and certain.)

## The work (one paragraph)
Add a recursive wat fn `(:wat::rete::explain <Explained> <fact>) -> :wat::rete::Why` that walks the support
index P12a built. For a derived fact (present in `Explained/support`), it returns `Why{fact, via}` where `via`
is the list of child `Why`s — one per support-chain edge, each produced by recursively `explain`-ing the
supporting fact. For a base fact (absent from the index), it returns `Why{fact, via=[]}` (the leaf). Then
un-ignore the 2 north-star tests; both go green.

## The record (self-recursive — verified to type-check)
```clojure
(:wat::Record::def :wat::rete::Why
  [fact <- :wat::Record
   via  <- :wat::core::PersistentVector<wat::rete::Why>])   ;; self-recursive; empty via ⟺ base/leaf
```
(A self-recursive record def type-checks — confirmed by probe this session. `Why/via` is the auto-accessor the
north-star reads.)

## Read in order (the rooms)
1. `wat/rete.wat` — the substrate P12a added: `Explained {session, support}` and `Support {rule, token}` (near
   the Session def), and `Token {matches, bindings}` (~:28). `Token/matches` is
   `PersistentVector<(wat::Record, wat::core::i64)>` — the support chain; each entry a `Tuple(fact, alpha-id)`.
2. `wat/rete.wat` — copy the IDIOMS already here: `PersistentMap/get` → Option (e.g. :201 with `Option/expect`),
   the Option-match pattern, `foldl`/`map` over a PV, the empty-typed-PV constructor (how `facts`/production
   memory build an empty PV — avoid the `(PersistentVector :Type)`-captures-the-ctor footgun), and a recursive
   wat fn (render-dag :191 / deporder recursion).
3. `tests/probe_arc278_P12_explain_walk.rs` — the north-star. It binds `fired (fire-rules-explain session)`
   (an `Explained`), calls `(explain fired <derived-fact>)`, and asserts `(length (Why/via …))` == 2 (for
   `ColdAndWindy`) and == 1 (for `WeatherAlert`). Remove the 2 `#[ignore]` lines.

## Implementation sketch (fill it; copy rete.wat idioms for exact verb forms)
```clojure
(:wat::core::defn :wat::rete::explain
  [ex <- :wat::rete::Explained  fact <- :wat::Record]
  -> :wat::rete::Why
  (:wat::core::let [support (:wat::rete::Explained/support ex)
                    sv-opt  (:wat::core::PersistentMap/get support fact)]
    ;; match sv-opt: Some(support-entry) ⇒ derived (recurse on each chain edge); None ⇒ base leaf.
    ;; derived:  via = map/foldl over (Token/matches (Support/token sv)) →
    ;;           for each tuple m: (explain ex (first m))   ;; first m = the supporting fact
    ;;           collected into a PersistentVector<Why>
    ;; base:     via = empty PersistentVector<Why>
    ;; both:     (:wat::rete::Why fact <via>)
    ))
```
- `(:wat::core::first m)` extracts the supporting fact from a matches `Tuple(fact, alpha-id)`.
- `via` MUST be a `PersistentVector<Why>` (use `map` if it yields a PV, else `foldl` + `PersistentVector/conj`).
- The recursion terminates: base facts are not in `support`, so the None branch returns a leaf — no infinite
  loop (the support DAG is acyclic by the fixpoint's round structure).

## Blast radius (bounded)
- `wat/rete.wat` — the `Why` Record def + the `explain` fn. Additive only.
- `tests/probe_arc278_P12_explain_walk.rs` — remove 2 `#[ignore]` lines.
- **NO Rust changes** (the walk is wat). **NO** change to `Explained`/`Support`/`Token`/`Session`/the fire
  paths. **NO** `:met`/`:bound`/`WhyVia`/rule-on-Why (that's P12c). **NO** EDN round-trip for `Why`.

## STOP triggers (halt and surface — do not improvise)
1. **STOP if a self-recursive `Why` Record def is rejected** by the checker — surface the exact error (the
   probe said it type-checks; if your form differs and fails, do not work around it blindly).
2. **STOP if `explain` cannot be written as pure wat** (needs a Rust primitive) — surface what's missing. The
   walk should be expressible with `PersistentMap/get` + `match` + `Token/matches` + `first` + `map`/`foldl` +
   recursion, all already in rete.wat.
3. **STOP if making the north-star green needs `:met`/`:bound`/an edge type** — it must not; the assertions are
   via-counts only. If you think it does, re-read the probe — you are over-building (that's P12c).
4. **STOP if any rete differential or floor regresses** (rete.wat is the oracle — additive only).

## Acceptance (un-ignore the 2 north-star tests)
- `cargo test --release -p wat --test probe_arc278_P12_explain_walk` → **2 passed; 0 failed**
  (`explain_cold_and_windy_reaches_its_two_inputs`, `explain_weather_alert_has_one_derived_support`).
- Differential UNCHANGED: `cargo test --release -p wat --test probe_arc278_P4a_native_fire_rules --test probe_arc278_P12a_explain_substrate` → green.
- Floors UNCHANGED: lib `940 / 36`, deftest `264 / 1`, deporder `1 / 0`, nursery `~893 / 4`.
- `cargo build --release` clean (no new warnings).

## Prior comparable (copy the shape)
- `render-dag` (rete.wat:191) — a recursive wat walk over the network using `PersistentMap/keys`/`get` +
  `foldl` + `Option/expect`.
- `seed-token`/`extend-token` (rete.wat:562/:700) — Tuple construction + `Token`/`Element` record building.
- `query-by-type-string` (rete.wat:1068) — reading + filtering a PV from a Session.
