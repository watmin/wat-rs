# Arc 209 — `:wat::service::defservice` meta-form (protocols arc)

**Status:** OPEN 2026-05-17.

**Priority:** **BLOCKING.** Per arc 203 DESIGN § "What arc 203 demands from upstream" demand 1: protocols arc / defservice meta-form. Arc 203 closure depends on this; arc 170 closure depends on arc 203 closure; lab reconstruction depends on arc 170 closure.

**Pedigree:**
- **Arc 203 slices 1-3f-naming + 3f** hand-rolled "the one pattern" for service architecture at thread tier and process tier: struct-restricted Admin + User capabilities + Wire enum protocol + dispatch loop with secret-witness validation + Result-bearing wrappers with typed ServiceError. The shape works; the proof shipped.
- **Arc 207** minted typed `:wat::core::Uuid` (server-id and user-id are now type-honest at substrate level).
- **Arc 208** flipped Process/readln + Process/println to Result-returning (process tier transport now mirrors thread tier's discipline). Demand 2 of arc 203 satisfied.
- **Demand 1 — protocols meta-form** unopened until now. Arc 209 opens it.

**Realization** (per arc 203 DESIGN line 224): what arc 203 hand-rolled IS Clojure's protocols pattern (independent convergence per `user_no_literature`). The natural next step is a substrate meta-form that abstracts the repetition. Per `feedback_simple_is_uniform_composition`: N identical compositions IS simple; abstracting them into one form is the simplest possible composition.

## The crack arc 209 closes

Today, every service the substrate vends repeats the arc 203 hand-rolled pattern by construction:
- Each service defines its own Wire + WireResp enums
- Each service defines its own ServiceError variants (AccessDenied / PeerDied / ServerDied / Disconnected — same shape per service)
- Each service mints its own Admin + Client/User capability structs with struct-restricted accessors
- Each service writes its own dispatch loop with select + route + validate-server-id + handler dispatch
- Each service writes its own client-side Result-bearing wrappers (one per operation)
- Each service writes its own per-tier transport adapter (thread crossbeam vs process stdio multiplex)

Per arc 203 slice 3f SCORE: each Result-propagation site is a 3-7 level nested match. Per the depth-3 decomposition rule (arc 203 DESIGN line 281+): substrate-generated wrappers MUST follow depth-3 by construction — hand-rolling N services per the slice 3f pattern would propagate the depth problem.

The boilerplate IS the consumer pressure for defservice. Per arc 207 carry-forward discipline: when consumer pressure exists on disk (here: N × hand-rolled service repetition), the substrate primitive belongs in scope.

## Goal — the meta-form surface (sketch; slice 1 audit refines)

Locked from arc 203 DESIGN § "Post-3f pivot" lines 239-261:

```scheme
(:wat::service::defservice :counter
  :admin    {Provision   [initial :wat::core::i64]                        -> :counter::User
             Deprovision [user    :counter::User]                          -> :wat::core::nil
             Stop        []                                                 -> :wat::core::nil}
  :user     {Get         []                                                 -> :wat::core::i64
             Increment   [n :wat::core::i64]                                -> :wat::core::i64
             Reset       []                                                 -> :wat::core::i64}
  :state    :wat::core::i64
  :handlers {<keyword-map of operation-name → handler-fn>})
```

**Substrate auto-synthesizes (all the artifacts arc 203 hand-rolled):**
- `:counter::Wire` + `:counter::WireResp` enums (Admin/User tagged variants per operation)
- `:counter::ServiceError` enum (standard variants: AccessDenied / PeerDied / ServerDied / Disconnected — Vector-typed chains per arc 113)
- `:counter::Admin` + `:counter::User` capability structs (struct-restricted per arc 203's substrate primitive)
- Server dispatch loop (select + route + validate server-id + handler invocation; depth-3 decomposed by construction)
- Client-side Result-bearing wrappers (one per operation; mirror thread-tier + process-tier shapes)
- Per-tier transport adapter (thread = crossbeam Sender/Receiver; process = ProcessPeer<I,O> with arc 208 Result-returning I/O)

**Substrate validates at freeze time:**
- Every operation in `:admin` + `:user` has a registered handler in `:handlers`
- Handler signatures match operation signatures
- Missing handler → PANIC with diagnostic (no silent omission)
- Handler signature mismatch → PANIC with diagnostic

## What the consumer writes (vs what defservice generates)

Today (arc 203 hand-rolled, ~500 lines per service):
- Wire + WireResp enums (~30 lines)
- ServiceError enum (~10 lines)
- Admin + User struct-restricted declarations (~40 lines)
- Server dispatch loop with select + route + 3-7 layer match per arm (~200 lines)
- 6-10 client wrappers, each 3-7 layer match (~150 lines)
- Per-tier transport setup (~50 lines)

After arc 209 (defservice generates everything; consumer writes only handlers):
- Protocol declaration (~20-30 lines of operations + signatures)
- Handler map (~50-100 lines depending on service complexity)
- Total: ~70-130 lines per service

**~75% reduction in service-implementation surface.** And — load-bearing — the substrate-generated parts are CORRECT BY CONSTRUCTION. No service author can:
- Forget to validate server-id (substrate generates the validation)
- Forget to propagate ServerDied (substrate generates the Result-bearing wrapper)
- Forget to drain stdin before join (substrate-generated transport adapter enforces lockstep)
- Exceed depth-3 in generated wrappers (substrate decomposes by construction)

## Out of scope (affirmatively, NOT deferral per arc 207 carry-forward discipline)

- **Non-RPC-shaped services** (e.g., pure-fan-out, pure-fan-in, multicast). defservice models request-response services. If a service genuinely needs different shape, the user writes it hand-rolled — same primitives, no defservice wrapper.
- **Mixed-tier services** (one service has both thread-tier and process-tier endpoints). Arc 209 ships per-tier defservice instances; a service that spans tiers spawns two defservices and bridges them at the user level.
- **Dynamic protocol evolution** (services that add/remove operations at runtime). defservice is static-checked at freeze time per substrate doctrine; runtime mutation is not in scope.
- **Authentication beyond secret-witness server-id** (signed requests, mTLS, etc.). Arc 203's secret-witness pattern is the substrate's authentication primitive; richer auth is consumer-side.
- **Rate limiting / quota / load shedding** at the substrate level. These are application concerns; defservice provides the dispatch + lifecycle, application adds policy in handlers.
- **Cross-process / cross-machine remoting beyond process-tier stdio**. The third tier (`run-remotes` per INTERSTITIAL § 2026-05-15 fractal wat-vm tree) is future substrate work; arc 209 ships thread + process tiers only.
- **Bracket combinator integration with defservice's spawn** (arc 170 D3 + Stones E/F/G/H). When arc 170 brackets ship, defservice's transport adapter consumes them; until then defservice uses the existing spawn-thread + spawn-process primitives directly.

## Slicing (sketch — slice 1's audit refines based on implementation strategy decision)

Likely 5-6 slices. Per `feedback_iterative_complexity`: build small. Per arc 207 + arc 208 proven cadence:

| Slice | Status | What | Notes |
|---|---|---|---|
| **1 — audit + implementation strategy decision** | SHIPPED 2026-05-17 | Audit complete (SCORE-SLICE-1.md). Strategy: option (a) — PURE DEFMACRO. Zero substrate changes; defmacro expands to `(:wat::core::do ...)` containing all generated artifacts; do-splice pipeline (arc 170 Gap C + Gap J) handles top-level splicing identically to individually-declared forms. `deftest` proves this pattern in production. One new file: `wat/service.wat` carrying the defmacro. | Audit OVERTURNED orchestrator hypothesis of "hybrid" — substrate-as-teacher cascade working as designed. Honest deltas surfaced (see "Honest deltas from slice 1 audit" below). |
| **2 — mint defservice defmacro** | BLOCKS on 1 | Per slice 1 SCORE 19-item checklist: ship the defmacro in `wat/service.wat`. Synthesizes Wire enum + ServiceError + Admin/User structs + dispatch loop + wrappers via expand-time computed unquote. Handler-completeness validation at expand time (no check.rs hook needed). Minimal test: a "Hello" service with 1 admin op + 1 user op + thread tier only. | Pure wat-side work; zero substrate changes. Stepping stone proves synthesis works on simplest case. |
| **3 — process tier transport** | BLOCKS on 2 | Add process-tier conditional path to defservice. Per honest delta 3: process tier has WireResp + Provisioned-with-id (vs thread tier no WireResp + Provisioned-with-channels). Per honest delta 2: process-tier server-id is always `Uuid/nil` (forms-block can't capture runtime values; design characteristic, not limitation). Same Hello service for both tiers; tests cover both. | Per-tier adapter completes the meta-form |
| **4 — Counter demo migration (validation)** | BLOCKS on 3 | Migrate `wat-tests/counter-service-{capability,process}-N3.wat` + `counter-client-capability-proof.wat` from hand-rolled to defservice USES. Hand-rolled pattern retires; demos become canonical defservice examples. ~75% line reduction expected; tests still pass with identical semantics. Per honest delta 5: forge-test helpers stay hand-rolled (test utilities; not defservice scope) — either retained or retired per slice 4 audit. | THIS proves the meta-form works for the canonical pattern |
| **5 — arc 203 slice 3g/3h/3i (vended services)** | BLOCKS on 4 | wat-lru CacheService + HologramCacheService + stdio services convert to defservice. Each was hand-rolled with the slice-3f-era pattern; each migrates to defservice. The substrate's "the one pattern" enforcement now applies uniformly. | Closes arc 203's vended-services scope |
| **6 — closure paperwork** | BLOCKS on 5 | INSCRIPTION (FM 11 grep clean) + DESIGN status CLOSED + USER-GUIDE entry for defservice + 058 row. Cross-references arc 203 (the originating consumer pressure) + arc 207 + arc 208 (the load-bearing substrate it depends on) + arc 146 (dispatch infrastructure) + arc 200 (macro splice) + arc 150 (variadic macros) + arc 170 Gap C + Gap J (do-splice pipeline that makes option-a possible). | Arc 209 closes → arc 203 demand 1 satisfied → arc 203 closure unblocks → arc 170 closure unblocks → lab reconstruction unblocks |

## Honest deltas from slice 1 audit (absorbed into DESIGN per FM 13)

Slice 1 audit (`SCORE-SLICE-1.md`) surfaced 6 honest deltas; all absorbed forward:

1. **Strategy decision overturned hypothesis.** EXPECTATIONS predicted option (c) hybrid; audit found option (a) pure defmacro is sufficient because the do-splice pipeline (arc 170 Gap C in `src/runtime.rs:1731–1754` + Gap J in `src/types.rs:1450–1481`) handles `(:wat::core::do ...)` expansion at every pipeline stage identically to individually-declared forms. `deftest` family (`wat/test.wat:298–307`) proves the pattern. Counter-service tests (909/905 lines) prove it scales to the full artifact set defservice generates. Substrate-as-teacher cascade working: the audit grounded the right answer that speculation got wrong.

2. **Process-tier server-id is always `:wat::core::Uuid/nil`.** Forms-block (per arc 170 Stone C2 + Slice 6) cannot capture runtime-minted values across the spawn boundary. Process-tier services use Uuid/nil as their secret-witness sentinel; the substrate enforces the dispatch validation with `(= wire-sid (:wat::core::Uuid/nil))` inline comparison. This is a design characteristic of the static forms-block contract, not a limitation. Slice 3 BRIEF inscribes the pattern explicitly.

3. **`WireResp` enum is process-tier-only.** Thread tier has no WireResp because thread-tier responses route via per-user Sender/Receiver pairs (not multiplexed over a single stream). Process tier needs WireResp to multiplex Admin + User responses over the single stdio stream. Defservice synthesis is tier-conditional: thread-tier expansion omits WireResp; process-tier expansion generates it. Slice 2 ships thread-tier only; slice 3 adds the conditional process-tier path.

4. **`AdminResp::Provisioned` shape differs by tier.** Thread tier: `Provisioned` carries the channel pair (Sender + Receiver) for the new user. Process tier: `Provisioned` carries only the `user-id` (channels are demultiplexed from the single stdio stream by the parent). Slice 2 generates thread-tier shape; slice 3 BRIEF handles the divergence explicitly.

5. **Forge-test helpers are NOT generated by defservice.** Tests that exercise rejection paths (forged server-id, wrong-id Admin rejection) are TEST UTILITIES, not part of the service surface. Slice 4 migration of counter demos retains forge-test helpers as hand-rolled OR retires them per the slice's audit; defservice does not auto-generate test utilities.

6. **Handler-map syntax: list-of-pairs `((OpName handler-fn) ...)`.** Slice 1 audit recommends this shape over a literal map-keyword form for defmacro compatibility (the existing defmacro infrastructure handles list-of-pairs cleanly; map literals would need extra parser work). Slice 2 BRIEF locks the syntax; final defmacro signature is:

```scheme
(:wat::service::defservice :counter
  :admin    ((Provision   [initial :i64]                        -> :counter::User)
             (Deprovision [user    :counter::User]              -> :wat::core::nil)
             (Stop        []                                     -> :wat::core::nil))
  :user     ((Get         []                                     -> :wat::core::i64)
             (Increment   [n :wat::core::i64]                    -> :wat::core::i64)
             (Reset       []                                     -> :wat::core::i64))
  :state    :wat::core::i64
  :handlers ((Provision  <handler-fn>)
             (Deprovision <handler-fn>)
             (Stop       <handler-fn>)
             (Get        <handler-fn>)
             (Increment  <handler-fn>)
             (Reset      <handler-fn>)))
```

All operation lists + handler maps are list-of-pairs. Substrate-grouped under `:admin` / `:user` / `:state` / `:handlers` keyword tags. Goal section's earlier sketch (map-literal `{...}` form) was DESIGN imprecision; this list-of-pairs form is the locked surface.

## Substrate touchpoints (preliminary; slice 1's audit refines)

- `src/macros.rs` — defmacro infrastructure (arc 150 variadic + arc 200 splice symmetry)
- `src/runtime.rs` — quasiquote + struct-to-form (arc 091 slice 8) for AST manipulation
- `src/check.rs` — type scheme registration; potential new dispatch logic for defservice expansion-time validation
- Arc 203's substrate primitive (struct-restricted) — defservice uses it for capability struct generation
- Arc 146 dispatch — defservice may use it for operation-name → handler-fn routing
- Arc 198 restricted_to — defservice may use it for capability accessor restriction
- Arc 207 typed Uuid — server-id + user-id types in generated Admin/User structs
- Arc 208 Process I/O Result — process-tier transport adapter uses the new Result-returning verbs

## Connection to broader work

Arc 209 is the protocols arc — arc 203 demand 1 per arc 203 DESIGN § "What arc 203 demands from upstream." Arc 208 (demand 2) shipped earlier today; arc 209 closes the demand pair.

**Forward chain after arc 209 closes:**

```
Arc 209 closes (defservice meta-form + counter migration + arc 203 vended-services migration)
            ↓
Arc 203 closure (slice 3j: INSCRIPTION + 058 + USER-GUIDE)
            ↓
Arc 170 closure (D3 + Stones E/F/G/H — bracket combinators completed on demonstrated user pattern)
            ↓
Lab reconstruction (per project_lab_reconstruction — substrate impeccable; lab rebuilds against canonical defservice + brackets)
```

## Discipline carry-forward (load-bearing for the INSCRIPTION when arc 209 closes)

This arc embodies the meta-pattern discipline:

**When N services repeat the same architectural pattern with N× the surface area and N× the inconsistency surface, abstract the pattern into a substrate meta-form.** The pattern lives in code once, generated correctly each instance; the application writes only what's domain-specific (operations + handlers).

Per `feedback_simple_is_uniform_composition`: N identical compositions IS simple; the abstraction is the simplest possible composition.

Per the Clojure-protocols convergence (INTERSTITIAL § 2026-05-16 seven-greats): independent design walking into Hickey's protocols pattern IS the validation that the engineering is on a known-good path. Per `user_no_literature`: foundational questions surface AFTER the practice; we built the hand-rolled pattern in arc 203 + recognized it as protocols + arc 209 mechanizes the recognition.

The depth-3 decomposition rule (arc 203 DESIGN line 281+) becomes structurally enforced via substrate-generated code rather than discipline at consumer sites. Auto-generation has no excuse for nesting beyond 3; the generator decomposes by construction.

## What arc 209 does NOT do (clear scope)

- Does NOT introduce new naming conventions beyond `:wat::service::*` (the namespace defservice lives in)
- Does NOT change arc 203 substrate primitive (struct-restricted) — uses it as-is
- Does NOT touch arc 110/111/146/198/200/203/207/208 substrate — composes them
- Does NOT mint a runtime registry of services or runtime dispatch — defservice is static-expansion + freeze-time validation
- Does NOT solve the orphan-process leak (arc 170 INTERSTITIAL leak notes are the diagnostic for that separate concern)
