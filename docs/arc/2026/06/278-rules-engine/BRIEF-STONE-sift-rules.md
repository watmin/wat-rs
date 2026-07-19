# BRIEF — Sieve::Rules: the rete filter (the chaos engine's inference form)

> Executor tier: sonnet shadowdancer. Orchestrator weighs by own re-run; commits.
> Task #6, the level-up from the Predicate. The Predicate SELECTS (survivors ⊆ input); the Rules form INFERS —
> per log item, fire the user's rules; one item can derive MANY facts, so the returned count can EXCEED the input.
> "the flood becomes inference." Primes ONLY (spawn-program' entry via /start, connect'/send'/recv'); NEVER the
> legacy bare spawn-program / spawn-program-ast.

## What is already PROVEN by a run (do not re-derive; build on these)
- **★ STOP-1 CLEARED — a macro CAN generate a service (LANDED `26e4eace`, 2026-07-19).** The substrate gap is fixed:
  a macro emitting `(do (defsurface … :messages […]) (defservice :satisfies …))` now hoists the nested surface's
  `:messages` accessors + `::Variant` ctors (Option A — `expand.rs` do-splice re-enters the per-form dispatch).
  Exemplar + regression gate: `tests/macros/probe_arc278_macro_generates_service.{rs,wat}`. **Do NOT re-run STOP-1;
  build straight on it.** ⚠ The generated per-op message types MUST follow defservice's S1/gRPC naming convention
  (`service.wat:894`): op `sift-rules` → `<Surface>::SiftRulesRequest` / `<Surface>::SiftRulesResponse` (PascalCase
  of the op name). A mis-named message → a TypeMismatch in the generated client stub.
- **Def-feeding = surface-splice.** A `:satisfies` service ships its surface's `:messages` to its forked worker's
  freeze — the arena proved it: the producer's `:prod::*` types lived in its surface `:messages` and were present at
  its worker's freeze (`tests/services/probe_arc278_sift_arena.wat`). So the user's `defrecord`s go into the adhoc
  service's surface `:messages`; the worker freezes with them. Runtime `eval-ast!` registration does NOT work
  (probed: it fails) — types are freeze-time. This is why the def must ride the surface.
- **★ Rete-in-a-service core — RE-PROVEN 2026-07-19 (`scratchpad/probe-rules-core.wat`, `cargo wat`):**
  `compile [rules] → template Session` (WM empty, rete.wat:796) → per seed `(insert template seed)` (:827) →
  `fire-rules` (:1832) → `query fired :Type` (:1919) / or set-diff on `Session/facts`. Three things proven in one run:
  (1) **per-item RESET is FREE** — a `Session` is an immutable value, so each item fires from the SAME `template`
  and one seed never poisons the next (`hot=2`, then `cold=0` from the same template → `total=2`); (2) **inference
  output > input** — one hot `Temp` seed fired TWO rules → TWO deductions; (3) **heterogeneous return** — the
  deductions (a `Hot` AND a `Warn`) collect into one `PersistentVector<wat::core::Value>` (the reply-wire carrier;
  Value is the universal top, R7). Syntax reference: `tests/rete/probe_arc278_query_type_safe.wat`.
- **Deducing the derived facts (recommended, rule-agnostic):** deductions = post-fire facts − pre-fire facts —
  `(Session/facts (fire-rules pre)) \ (Session/facts pre)` where `pre = (insert template seed)`. This excludes the
  seed and needs NO per-rule knowledge (the RED gate wants derived-only: 30 hot × 2 = 60, the seeds NOT counted).
  Proven-alternative fallback: explicit `(query fired :DerivedType)` per type (probe-rules-core), if the macro
  emits them. Pick the rule-agnostic set-diff unless `Session/facts` set-diff isn't cleanly expressible → STOP-3b.
- **The macro/splice pattern.** `defservice` IS a `defmacro` (`wat/service.wat:71`) and splices via `~@` throughout
  (`~@init-arg-names`, `~@serve-op-arms`, `~@init-param`). Splicing the user's forms into a generated service is the
  same move — now proven end to end by the Option-A exemplar above.

## The shape (grounded end to end)
```
user (compile-time — their defs + rules are literal forms):
  (sift-rules-defsvc  :name  :usr::my-sift
                      :defs  [(defrecord :usr::Temp [c <- :i64]) …]   ;; ~@spliced into the adhoc surface :messages
                      :rules [(defrule :usr::hot :when [(:usr::Temp (?c <- :c) (> ?c 50))]
                                                 :then (insert (:usr::Hot :c ?c))) …])   ;; compiled in the op
  → macro emits: a defsurface (:messages = ~@defs + SiftRulesRequest/Response) + a :satisfies defservice
      (:peers [Journal]; :init compiles ~@rules → a Session template)
  /start (spawn-program', prime) → worker FREEZES with :usr::* present
  op sift-rules [namespace window limit cursor]:
    read a page from Journal (Sieve::All / query-logs) → for each Log:
      typed-read (Log/message)   ;; uses the spliced defs; a type NOT among :defs → FAILS the request
        → insert into a FRESH Session (reset per item — one seed never poisons another, alpha-only structural)
        → fire → query the derived facts (Deductions)
    flat-map all deductions → reply ; user blocks till the page drains, requests page after page
```

## Read in order (the rooms)
1. `scratchpad/probe-rules-inference.wat` — the PROVEN per-item fire (compile → insert one seed → fire → query;
   output ≥ input). Copy this fire chain verbatim.
2. `tests/rete/probe_arc278_query_type_safe.wat` — the rete surface syntax (`defrule`, `compile`, `insert`,
   `fire-rules`, `query`, `Rule` literal).
3. `tests/services/probe_arc278_sift_arena.wat` — THE model: an adhoc `:satisfies` service with domain types in its
   surface `:messages`, `:peers [Journal]`, process-tier, grant-before-dial, paging via a bounded foldl, the exact
   start/connect' orchestration. The Rules service is this consumer with rete in place of the pure predicate.
4. `wat/service.wat:71` + the `~@` splice sites — `defservice` is a `defmacro`; the splice mechanism the macro copies.
5. `wat/rete.wat:1954-1996` (`defrule` macro) + `:52` (the `Rule` record) — the rule-as-data model.
6. `wat/telemetry/journal.wat:164-192` (`query-logs`/`sift-logs`) + `tests/services/probe_arc278_sift_logs.wat` —
   the page read + `Sieve` + the paging shape.

## STOP-1 — CLEARED (do NOT re-run). Proceed straight to the full build.
The macro-emits-a-defservice-with-spliced-defs crux is PROVEN + LANDED this session (Option A, `26e4eace`; exemplar
`tests/macros/probe_arc278_macro_generates_service.{rs,wat}`). The whole Rules-form UX rested on it; it now stands.
The remaining trap-doors are STOP-2 (per-item reset — also proven, see probe-rules-core) and STOP-3 (return type —
proven) + the new STOP-3b (deduction set-diff) and STOP-J (Journal page-read integration), below.

## The full build (after STOP-1 clears)
- **The `sift-rules-defsvc` macro** — emits the adhoc `defsurface` (`:messages` = `~@:defs` + `SiftRulesRequest
  [namespace time-lo time-hi limit cursor]` + `SiftRulesResponse :Pure [:Deductions [items <- (Vector :Value)] |
  :Fatal [err <- :wat::query::Fatal]]`) and the `:satisfies` `defservice` (`:peers [:wat::telemetry::Journal]`;
  `:init` compiles `~@:rules` into a Session template held in state; the op below). The Deduction item type: the
  derived facts are the user's own `:usr::*` records — return them as a `(Vector :wat::core::Value)` (heterogeneous;
  the caller, holding the defs, matches them) OR re-serialized EDN text — pick the simpler that type-checks and note it.
- **The op** `sift-rules [s req]`: read a page from the Journal (namespace + window + limit + cursor; use the
  existing `query-logs`/`Sieve::All` path); accumulate deductions across the page via a foldl (mirror the arena's
  bounded-page foldl); per Log: typed-`read` the `message` (a type NOT among `:defs` → return `::Fatal`, the
  no-hidden-failures floor), `insert` into a FRESH Session (reset per item), `fire-rules`, `query`/collect the
  deductions; flat-map; reply `::Deductions`.
- **Both loci** (R31/R32 — non-negotiable): the RED gate runs thread AND process (process needs grant-before-dial to
  the Journal, copy the arena).

## Out of scope = rejected
- The live-WM-across-messages streaming form (R0, task #7) — this is the PAGED per-item-fire form.
- Beta joins across records (one seed per fire ⇒ alpha-only is structural — the design; do not attempt page-local joins).
- Foreign facts in rete (rete is typed-only — the user MUST supply the defs; ruled).

## STOP triggers (halt + surface — findings that guide R0)
- **STOP-1** (above): the macro cannot emit a defservice / the spliced def doesn't reach the worker's freeze.
- **STOP-2:** per-item Session reset — if a fresh Session per item is not achievable cheaply (compile-once, re-seed),
  report the mechanism; do NOT let facts leak across items (one seed never poisons another).
- **STOP-3:** the derived-fact return type — if `(Vector :Value)` can't carry heterogeneous user records across the
  reply wire, STOP and report (re-serialized EDN is the fallback; note which).

## The RED gate (install + make green)
`tests/services/probe_arc278_sift_rules.{rs,wat}` (copy the arena's harness):
1. A user supplies defs (`:usr::Temp`, `:usr::Hot`, `:usr::Warn`) + two rules (hot Temp → Hot; hot Temp → Warn). The
   producer floods N Logs whose messages are `:usr::Temp` (some hot, some cold). The Rules service sifts a page and
   returns the flat-mapped deductions: a hot Temp yields 2 deductions, a cold one yields 0 — so the returned count
   EXCEEDS the count of hot inputs (assert the exact number: e.g. 30 hot × 2 = 60 deductions from 30 hot of a 240 page).
2. Fail-closed: a Log whose message type is NOT among the supplied defs → the request returns `::Fatal` (not a crash,
   not a silent skip).
3. BOTH loci (thread + process).

## Expectations
| what | command | expected |
|---|---|---|
| the gate green | `cargo test --release -p wat sift_rules` | pass (exact deduction count; fail-closed; both loci) |
| nothing else breaks | `cargo nextest run --release` (Summary) | 0 new failures |
Runtime: ~20–30 min (largest strike). Trap-doors: STOP-1 (the macro), STOP-2 (reset), STOP-3 (return type).

## Report back (raw facts): (1) STOP-1 result (paste the probe); (2) the gate output + the exact deduction count +
## how it exceeds the input; (3) the nextest Summary proving 0-new; (4) files+lines touched; (5) any STOP surfacing.
## Do NOT commit.
