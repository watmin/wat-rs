# Arc 291 — Strike 4b-iv: cross-process contract distribution (the actor-network bridge)

**Status: DESIGN SETTLED (2026-06-24), STRIKE-READY for the next session.** This is the FINAL 291
manifestation: a service can be a *client of another service across processes*. 291 closes by **proving 290
is buildable** — not by building 290. The deliverable is the **telemetry-bridge probe's process tier going
GREEN**; that green probe IS the template the 290 migration references.

## What's already proven (committed)

- **4b-ii-a** (`2cea3f45`) — the struct-State re-tool. **4b-ii-b** (`75e29a2d`) — lineage→Status rename.
- **4b-iii — the bridge composition** (`wat-tests/service-telemetry-bridge.wat`): a `worker` whose `:init`
  dials a `recorder` and records through the stored client. **Thread tier + hibernate/resume tier GREEN** —
  the in-locus "service holds a client to another service" + the reconnect-on-resume both work. The
  **process tier is IGNORED** (the gap below) — it is the RED gap-marker for this strike.

## The gap (grounded)

A service B (in its own forked **process**) that is a *client* of service A fails: B's child universe can't
resolve `A/method`/`A/x-request`. Reason (`wat/service.wat` final `do` vs `service-forms-def`): a
defservice's **full client face** (Op/Reply enums + Request/Response records + request ctors + per-op
methods) is emitted into the **parent** universe where the macro expanded; `service-forms` ships only the
**server** subset to the forked child. So B's child has B's *own* forms, not A's client contract. **ocap
(272) hands the `Address'` (the grant to *reach* A) — but a cap is not a contract.** Reach + contract are
two halves; we built reach, not contract distribution. (Full detail: `DETOUR-wat-reader-discovery.md` sibling
+ the research synthesis — see the 4b-iv thread.)

## The design — settled by FOUR-QUESTIONS

### Decision 1 — distribution mechanism: **A wins** (compile-time `client-forms` bundle + `:calls` clause)
Four-questioned A/B/C/D (full table in the session thread): **A** (compile-time bundle) passed Obvious/Simple/
Honest/UX. **B** (runtime contract-ship at connect) FAILED Honest — it must bypass the mutation-form gate
(runtime refuses `defn`/`Record::def`). **C** (hand-duplicate records) FAILED Honest — the copy drifts from
source. **D** (first-class `defcontract`) FAILED Simple — splits one concept into two for no near-term gain.

### Decision 2 — names: **`client-forms` + `:calls`** (intueri, weighed)
- **`client-forms`** — exact structural mirror of `service-forms` (server-face ↔ client-face).
- **`:calls [:recorder]`** — bare verb fitting the `:durable`/`:ephemeral`/`:ops` clause family; honest (the
  point of the declaration IS to make calls). Beat `:client-of` (prepositional, breaks register), `:clients`
  (inverts the relation), `:dials` (metaphor).

### Decision 3 — the called service's ADDRESS is NOT durable: **address-as-`:init`-arg wins** (the stale-endpoint fix)
Four-questioned. **α — address on the `:durable` record FAILED Honest, decisively**: a durable address is
*where the server was at hibernation*; on resume the server may have moved/restarted → reconnect to a corpse.
**β — address passed to `:init`, provided fresh by `start`/`resume`**: passed all four. The address is the
**most ephemeral thing** — live topology, owned by whoever's wiring it (the caller today, the orchestrator
tomorrow, using the dependency graph). The service holds the **client**, never the address. *(This corrects
the original 4b-iii probe, which wrongly put the address on `:durable`.)*

### Convention — `:init` before `:ops` (constructor before methods). Canonical clause order:
`:durable → :ephemeral → :calls → :init → :ops`. (Macro is order-independent; this is what we DEMONSTRATE.)

## The corrected user UX (the template 290 references)

```clojure
;; recorder — the callee. NEEDS NOTHING SPECIAL: every defservice auto-emits its client-forms.
(:wat::service::defservice :my::recorder
  :durable [total <- :wat::core::i64]
  :ops [(:Record [s <- :State n <- :wat::core::i64] -> [ok <- :wat::core::bool] …)])

;; worker — the caller.
(:wat::service::defservice :my::worker
  :durable   [job-count <- :wat::core::i64]                               ;; the SOUL — NO address here
  :ephemeral [recorder  <- :wat::kernel::Peer'<my::recorder::Op,my::recorder::Reply>]   ;; the client we hold
  :calls     [:my::recorder]                                             ;; bundle recorder's client-forms into our child
  :init (:wat::core::fn [r <- :my::worker::Record
                         recorder-addr <- :wat::kernel::Address'<my::recorder::Op,my::recorder::Reply>]
          -> :my::worker::State
          (:my::worker::State/new r (:wat::kernel::connect' recorder-addr)))   ;; address IN → client attached → address GONE
  :ops [(:Work [s <- :State n <- :wat::core::i64] -> [done <- :wat::core::bool]
          (:wat::core::let [_ (:my::recorder/record (:my::worker::State/recorder s)
                                (:my::recorder/record-request n))]
            (:wat::service::Outcome::Reply s (:my::worker::WorkResponse true))))])
;; start/resume thread the LIVE address:
;;   (worker/start  locus (Record 0) recorder-addr)
;;   (worker/resume locus saved-record recorder-addr-NOW)   ;; ← CURRENT location, not the hibernated one
```

## THE BUILD (the macro change — `wat/service.wat`) — re-ground sites before editing

1. **Emit `:<fqdn>::client-forms`** — a `(defn :<fqdn>::client-forms [] -> :Vector<WatAST> (forms …))`
   carrying the CLIENT face: `request-records` + `response-records` + `Op` enum + `Reply` enum +
   `constructors` + the **per-op methods**. **NOT** the owner methods (`stop`/`hibernate` take a `Handle`).
   ⚠ The `methods` binding currently = per-op methods then `(conj … stop-method)` `(conj … hibernate-method)`
   — CAPTURE the per-op methods (`op-methods`) BEFORE those conjs, and client-forms uses `op-methods`.
   Sites: constructors `~706-753`, methods foldl `~768` (before the stop/hibernate conj at `~838`/`~861`),
   the `forms`/service-forms-def pattern `~857-873`, the final `do` `~945-963` (emit `client-forms-def`).
2. **`:calls [svcs]` clause** (all-kwargs clause-map): for each declared svc, B's `service-forms` body
   becomes `(concat (:<svc>::client-forms) … (forms <B's own forms>))` — prepend the callees' contracts so
   the child loads them FIRST. `service-forms` is evaluated in the PARENT (start-body calls
   `(service-forms-kw)`, passes the result to `launch` — `spawn.wat:262`), so `(:<svc>::client-forms)`
   resolves (both parent). Empty/absent `:calls` → service-forms unchanged.
3. **Multi-param `:init`** (THE CONSEQUENCE — the macro currently assumes a SINGLE-param `:init`).
   **THE LAW (builder, 2026-06-24): there is NO hard requirement that `:init` have a constant shape.** The
   honest contract is *"you must accept your state AND optionally anything else you need to operate"*:
   ```
   :init : (Record, …operating-inputs) → State
            ▲ MANDATORY first           ▲ 0+ optional (addresses, config — live topology, never durable)
            "accept your state"         "anything else you need to operate"
   ```
   The first param MUST be `:<fqdn>::Record` (the macro can enforce it — `ship-ty` already pins `record-ty`).
   Params 2+ are live inputs provided FRESH by `start`/`resume` (this is the address-as-`:init`-arg of
   Decision 3 — DI, typed + explicit). **Why it's only a small change (grounded `wat/service.wat`):** the
   *signature* already generalizes — `start-params = `[locus ~@init-param]`` (line 977) splices ALL `:init`
   binders, so a 2-param `:init` already yields a 3-param `start`. The single-shape assumption lives in exactly
   two downstream spots: `ship-ref = (first init-param)` (line 187, ships only the first binder) +
   `ship-ty = record-ty` / `Admin::Init [seed <- ~ship-ty]` (lines 188-189, 389 — the wire frame carries ONE
   value). Generalize: the Admin::Init/Resume ship carries a TUPLE of all live-input values; `dispatch-admin`
   applies `:init` to all of them in-locus; `resume-params` likewise. (This is "the orchestrator injects
   current endpoints" — K8s service-discovery shaped.)
4. **Prove:** rewrite `service-telemetry-bridge.wat` to the corrected UX (address as `:init` arg, `:calls`),
   un-ignore the process-tier deftest → GREEN. Keep thread + hibernate green. SET-diff vs HEAD = ∅.
   **That green process tier = the 290 template + R1 FULL PROBATUM EST → PAUSE (builder's) → INSCRIPTION.**

## OPEN sub-decisions (each its own FOUR-QUESTIONS when building — builder's rule)
- **(a)** Does `:calls [:recorder]` **auto-derive** the `:ephemeral` client field + the `connect'` (less
  typing, more derived structure, but consistent with `:durable`→Record derive), or stay **hand-declared**
  (the UX above)? Lean: settle at build.
- **(b)** Address-at-spawn (β, above) vs **connect-by-name** to a well-known service (the name resolves the
  CURRENT server → no address threading at all; fits a well-known telemetry service). connect-by-name exists
  (272, task #225). Lean: β is the general mechanism; γ is a clean special case for well-known infra.

## FORWARD NOTE (NOT 291 — the orchestration arc)
`:calls` declarations ARE a **service dependency graph**, for free (declared, not inferred). Harvest the
`(service-calls B A)` edges via wat-fix (code-is-data: it already reads `.wat` → spanned forms) → feed
**rete** → recursive rules answer transitive-deps / cycle-detection / **topological spawn order** /
failure-blast-radius. That is R5's control plane (K8s-from-first-principles) reasoning over the actor network
with the rules engine we already built (278 dual-impl oracle). **Capture only — build in the orchestration
arc, when a real need surfaces.** The hard part (honest dependency data) is done by the `:calls` declaration.
