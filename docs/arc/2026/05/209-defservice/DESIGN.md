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

---

## Surface settled 2026-05-17 (late) — corrections to slice 1 absorption (THIS IS THE LOCKED SURFACE)

The slice 1 deltas above (lines 105-160 area) were partially absorbed wrong by the orchestrator. Per INTERSTITIAL § 2026-05-17 (late) — defservice trust-recovery sub-story: the user caught the violations + drove the surface to its final form through several refinement rounds. This section is the FINAL LOCKED SURFACE; supersedes the conflicting framings in the slice 1 deltas section above (which stays as historical record of the absorption-failure-and-correction arc).

### The locked defservice form

```scheme
(:wat::service::defservice :counter
  :admin    [Provision   [] -> :counter::User
             Deprovision [user <- :counter::User] -> :wat::core::nil
             Stop        [] -> :wat::core::i64]
  :user     [Get         [] -> :wat::core::i64
             Increment   [n <- :wat::core::i64] -> :wat::core::i64
             Reset       [] -> :wat::core::i64]
  :state    :wat::core::i64
  :handlers [Start       <fn>
             Stop        <fn>
             Provision   <fn>
             Deprovision <fn>
             Get         <fn>
             Increment   <fn>
             Reset       <fn>])
```

**Clojure-shaped square brackets, not nested parens.** Cleaner than the prior list-of-pairs form. The substrate parses `:admin [Name [args] -> ret  Name [args] -> ret ...]` as a flat sequence of declarations.

### Handler shapes

| Handler family | Signature | Notes |
|---|---|---|
| Lifecycle: `Start` | `(state) -> state` | Caller-provided initial state lands here; service can do init work + return possibly-modified state |
| Lifecycle: `Stop` | `(state) -> state` | Final state returned; substrate ships to admin caller as Final<State> for hot-reload |
| Lifecycle: `Provision` | `(state, user) -> state` | Substrate already minted the User capability; handler does prep + returns state |
| Lifecycle: `Deprovision` | `(state, user) -> state` | Cleanup time; returns state |
| Domain (Get/Increment/Reset/...) | `(state, args...) -> (Tuple state return-value)` | Substrate threads state; extracts return-value for user caller |

**Lifecycle handlers return state ONLY** (no value channel — admin has no reason to receive data; if admin wants data they Provision themselves a User and read like any user).
**Domain handlers return (Tuple state return-value)** (substrate threads state; extracts value for user caller).
**All handlers can panic** — substrate catches → wraps as `ServiceError::ServerDied(chain)` per arc 170 slice 1i structured-exit + arc 208 Result pattern.

### Spawn API (transport choice = user choice; 1-ary Start)

```scheme
(:counter::spawn-thread state)   -> :Result<:counter::Admin, :counter::ServiceError>
(:counter::spawn-process state)  -> :Result<:counter::Admin, :counter::ServiceError>
(:counter::spawn-remote state ...) ;; future
```

**Caller provides initial state.** Spawn calls Start with that state. Hot-reload loop closes structurally:

```scheme
(let [s0      0
      admin   (:counter::spawn-thread s0)
      ;; ops
      final   (:counter::stop admin)]
  ...
  (:counter::spawn-process final))  ;; same state; different transport
```

### Admin / User capabilities

- `:counter::Admin` — substrate-generated struct-restricted; obtained from spawn-thread/spawn-process; holds opaque closure for management. Caller doesn't know transport behind it.
- `:counter::User` — substrate-generated struct-restricted; obtained from Provision; holds opaque closure for data ops.

Capabilities discriminate access at the type system layer (wrapper signatures take Admin OR User; type checker enforces). Substrate-generated dispatch routes Wire variants to the right handler arm. Restricted accessors prevent forgery.

### Substrate-internal (NOT user-visible)

- Wire / WireResp enums — process-tier-only multiplex envelopes; substrate hides them inside the per-tier transport adapter; thread tier uses direct channels and has no Wire wrapping
- Server-id minting + startup handshake — substrate generates: parent writes server-id to subprocess stdin at startup via Process/println; subprocess does Process/readln at startup; binds + enters dispatch loop. NOT user-managed.
- Dispatch loop — substrate-generated; routes Wire variants to handlers; threads state.
- Per-tier transport adapter — thread = crossbeam; process = ProcessPeer<I,O> with Wire multiplex + arc 208 Result-returning I/O.

### The agnostic-interface invariant

User-facing surface is **identical at both tiers**. `(:counter::get user!)` returns the same `:Result<:i64, :counter::ServiceError>` whether the service runs on a thread or in a process. Substrate hides the transport difference. No user-visible Wire/WireResp/forms-block/handshake mechanics.

### Two-surface concurrency canon (companion to defservice)

defservice is one of two user-facing canonical concurrency surfaces:

| Use case | wat surface |
|---|---|
| Long-lived state-bearing RPC service | `defservice` + `spawn-thread/process/remote(state)` |
| Fan-out parallel work (Ruby Parallel.map shape) | `run-threads` / `run-processes` (arc 170 D3 + Stones) |

Per INTERSTITIAL § 2026-05-17 (late) ten-greats convergence: both surfaces honor independent convergence with multiple greats. Both coexist; neither subsumes the other.

### Restrict raw spawn-* to substrate-internal

`:wat::kernel::spawn-thread` + `:wat::kernel::spawn-process` should be `restricted_to :wat::` after arc 209 ships. User code accesses concurrency ONLY via defservice + brackets. This eliminates structurally the entire walker-caught misuse class (ProcessJoinBeforeOutputDrain / ProcessJoinHoldsStdinSender / scope-deadlock / orphan / forge-id / silent Process I/O). Substrate refuses to compile user-side raw spawn-*.

**Scope open:** whether this restriction lands in arc 209's final slice OR a follow-up arc — orchestrator + user decide.

### Corrections to slice 1 deltas (the prior section)

| Slice 1 delta | Correction inscribed here |
|---|---|
| Delta 2 (Uuid/nil at process tier as design property) | DELETED — substrate generates startup handshake; server-id is freshly minted per spawn; demo's "out of scope for THIS demo" was demo-scope not defservice-scope |
| Delta 3 (WireResp tier-conditional user-visible) | DELETED user-visible framing; WireResp is substrate-internal multiplex envelope only |
| Delta 5 (forge tests out of scope) | DELETED — forge tests are substrate-generatable per arc 200 splice; defservice ships them as security proof |

Deltas 1, 4, 6 stay as inscribed (strategy = pure defmacro; AdminResp shape divergence hidden by typed capability struct; handler-map syntax — superseded here by the locked square-bracket form).

### Open scope questions for slice 2 BRIEF — RESOLVED 2026-05-17

Both questions settled by user 2026-05-17. Lock state below.

---

## Spawn surface locked 2026-05-17 — reclaim `:wat::kernel::spawn-program`

### The shape

User-facing concurrency entry is exactly ONE verb:

```scheme
(:wat::kernel::spawn-program :tier :service initial-state) -> :service::Admin

;; Examples
(:wat::kernel::spawn-program :thread  :counter 0)
(:wat::kernel::spawn-program :process :counter 0)
;; Future
(:wat::kernel::spawn-program :remote  :counter 0 ...)
```

Raw substrate primitives become substrate-internal:

| Symbol | Audience | Mechanism |
|---|---|---|
| `:wat::kernel::spawn-program` | user-facing | dispatch over `:tier` |
| `:wat::kernel::spawn-thread` | `restricted_to :wat::kernel::` | arc 198 machinery |
| `:wat::kernel::spawn-process` | `restricted_to :wat::kernel::` | arc 198 machinery |
| `:wat::kernel::spawn-remote` (future) | `restricted_to :wat::kernel::` | arc 198 machinery |

### Reclaiming a retired name (substrate-converges-with-itself)

`:wat::kernel::spawn-program` was minted arc 103a/105a for in-thread fresh-world spawn and retired in arc 170 slice 2 with active diagnostic (`src/check.rs:886`) + `BareLegacySpawnProgram` walker arm (`src/check.rs:2476-2504`). The retirement was honest — the SEMANTICS behind the name were wrong (a third-tier "in-thread fresh-world" option the substrate corrected to canonical two-mode).

The NEW semantics (`:tier :service state` dispatch) are different — unified entry OVER the canonical two-mode (plus future remote), not a third tier. The noun "program" was always right; only the prior mechanism was wrong. Reclaiming the noun for its honest meaning is forward-correction per `feedback_inscription_immutable`.

Full narrative: INTERSTITIAL § 2026-05-17 (later) "convergence #11 — the substrate converges with its own prior self."

Walker update: legacy 2-arg `(spawn-program src scope)` stays rejected; new 3-arg `(spawn-program :tier :service state)` form accepted via dispatch type-scheme.

### Four-questions verdict on the reclaim

| | Score |
|---|---|
| Obvious | YES — "program" is what we're spawning per wat doctrine (a wat process IS a wat program; INTERSTITIAL § slice 6 pivot) |
| Simple | YES — one verb; dispatch is the substrate's job; adding `:remote` later is one dispatch arm |
| Honest | YES — retired semantics were dishonest (third-tier shouldn't exist); reclaiming the noun for the honest meaning IS the forward-correction |
| Good UX | YES — readers searching for "how do I spawn?" find the canonical verb under its natural name; flipping `:thread`↔`:process` for crash isolation changes ONE keyword |

YES YES YES YES.

### Why restrict raw spawn-* here, not follow-up

The two-surface canon (defservice + brackets) only delivers its structural-misuse-elimination guarantee if users CANNOT bypass it. Leaving raw spawn-thread/spawn-process user-callable preserves every walker-caught misuse class — ProcessJoinBeforeOutputDrain (arc 170 Gap K), ProcessJoinHoldsStdinSender (arc 202), scope-deadlock (arc 117/126), orphan-process patterns, forge-id attacks (arc 203), silent Process I/O (arc 208). The restriction IS the structural elimination; deferring it preserves the surface that necessitated the walkers.

Per `feedback_no_known_defect_left_unfixed`: ship the restriction in arc 209, not later.

### Slice 2 stones (decomposed per `feedback_iterative_complexity`)

| Stone | Scope |
|---|---|
| **2a** | Mint `:wat::kernel::spawn-program` substrate dispatch (likely defmacro; verify against arc 200 splice + arc 143 computed-unquote during BRIEF drafting). Walker reshape: legacy form stays rejected; new form accepted. |
| **2b** | Apply `restricted_to :wat::kernel::` to raw spawn-thread + spawn-process via arc 198's `#[restricted_to(...)]` machinery. |
| **2c** | Sweep existing user callers to `spawn-program` dispatch. Known sites: bracket macros (run-threads/run-processes from arc 170 D1/D2), test framework (`:wat::test::run-thread` + `:wat::test::run-hermetic`), arc 203 ServiceWithProvisioning proofs, wat-tests/ direct callers. Scope: mechanical sweep on settled foundation. |
| **2d** | Mint `:wat::service::defservice` defmacro per locked surface. Expands to register Admin/User structs + Wire enums + Start/Stop/Provision/Deprovision/domain handler wrappers. Spawn API calls into `:wat::kernel::spawn-program` (NOT raw spawn-thread/process). |

Stepping-stone analysis: 2a + 2b + 2c land the SUBSTRATE FOUNDATION (spawn-program exists; raw spawn-* unreachable from user code; existing callers migrated). 2d lands defservice ATOP the settled foundation — defservice's expansion writes spawn-program calls, which only works honestly once 2a-2c have shipped.

Order: 2a → 2b → 2c (atomic-commit pair; 2b breaks 2c's baseline mid-sweep per recovery doc § atomic-commit) → 2d.

### Sweep scope warning (honest at BRIEF time)

2c is bigger than slice 1 audit predicted. Estimated sites:
- arc 170 D1/D2 bracket macros (run-threads, run-processes) — ~2 files
- `:wat::test::run-thread` / `:wat::test::run-hermetic` macros — `wat/test.wat`
- arc 203 ServiceWithProvisioning proofs — ~6 wat-tests
- counter-actor / counter-service proofs from arc 209 prep — ~4 wat-tests
- Other wat-tests/ direct callers — TBD; pre-flight grep

Pre-flight greps land in BRIEF-STONE-2A drafting per FM 9 baseline-pre-flight discipline.

---

## Compaction-recovery breadcrumb (2026-05-17 late)

**Tip at this commit on `arc-170-gap-j-v5-deadlock-state`.** Arc 209 status:
- Slice 1: SHIPPED `f815c14` (audit + pure-defmacro decision)
- Surface: LOCKED in this section (the "Surface settled 2026-05-17 (late)" section above)
- Slice 2 BRIEF: NOT YET DRAFTED (next move)
- Slice 2 source of truth: SCORE-SLICE-1.md's 19-item checklist + the locked surface in this DESIGN

**Recovery instructions for post-compaction orchestrator:**

1. Read this entire DESIGN file (especially the "Surface settled 2026-05-17 (late)" section — that's the LOCKED surface)
2. Read INTERSTITIAL § 2026-05-17 (late) "defservice is OOP done right" — the architectural recognition narrative
3. Read SCORE-SLICE-1.md for the 19-item slice 2 implementation checklist
4. Verify state: `git -C /home/watmin/work/holon/wat-rs log --oneline | head -10` should show this commit at tip; arc 209 slice 1 at `f815c14`; arc 208 CLOSED at `f1157f1`; arc 207 CLOSED at `ec1e2c5`
5. Next action: draft BRIEF-STONE-2A for `:wat::kernel::spawn-program` reclaim + walker reshape per § "Spawn surface locked 2026-05-17" above. Slice 2 is decomposed into 4 stones (2a→2b→2c→2d); 2a is the foundation stone. Do NOT draft a monolithic slice 2 BRIEF; each stone gets its own BRIEF + EXPECTATIONS + SCORE per `feedback_iterative_complexity`.
6. **CRITICAL** per the trust-failure recovery: orchestrator-side architectural review applies to sonnet's outputs. Every sonnet delta must pass four-questions against design intent BEFORE absorption into DESIGN. See INTERSTITIAL "trust-recovery sub-story" + memory `feedback_sonnet_output_requires_review`.

---

## Surface settled 2026-05-18 — collapsed shape + state-as-self contract (FINAL LOCKED SURFACE)

Today's conversation locked the final surface. Two prior surface-locking sections (yesterday's "Surface settled" + earlier-today "Spawn surface locked 2026-05-17") stay as historical record per `feedback_inscription_immutable`; THIS section supersedes them as the implementation target.

### The collapsed form

```scheme
;; --- preceded by typealias + handler defns in declaration order ---

(:wat::core::typealias :counter::State :wat::core::i64)

(:wat::core::defn :counter::on-start
  [s <- :counter::State]
  -> (:wat::core::Tuple :counter::State)
  (:wat::core::Tuple s))

(:wat::core::defn :counter::on-stop
  [s <- :counter::State]
  -> (:wat::core::Tuple :counter::State :counter::State)
  (:wat::core::Tuple s s))

(:wat::core::defn :counter::on-grant
  [s <- :counter::State]
  -> (:wat::core::Tuple :counter::State :counter::User)
  ...)

(:wat::core::defn :counter::on-revoke
  [s <- :counter::State  user <- :counter::User]
  -> (:wat::core::Tuple :counter::State)
  (:wat::core::Tuple s))

(:wat::core::defn :counter::on-get
  [s <- :counter::State]
  -> (:wat::core::Tuple :counter::State :counter::State)
  (:wat::core::Tuple s s))

(:wat::core::defn :counter::on-increment
  [s <- :counter::State  n <- :counter::State]
  -> (:wat::core::Tuple :counter::State :counter::State)
  (:wat::core::let [new (:wat::core::+ s n)]
    (:wat::core::Tuple new new)))

(:wat::core::defn :counter::on-reset
  [s <- :counter::State]
  -> (:wat::core::Tuple :counter::State :counter::State)
  (:wat::core::Tuple 0 0))

;; --- the collapsed defservice form ---

(:wat::service::defservice :counter
  :state :counter::State

  :admin [Start  :counter::on-start
          Stop   :counter::on-stop
          Grant  :counter::on-grant
          Revoke :counter::on-revoke]

  :user  [Get       :counter::on-get
          Increment :counter::on-increment
          Reset     :counter::on-reset])
```

### What changed from prior surface-locking sections

| Aspect | Prior (yesterday + earlier today) | Today (FINAL) |
|---|---|---|
| `:handlers` section | Separate; `[Start <fn> Stop <fn> ...]` flat alternating | **DISSOLVED** — collapsed into `:admin` / `:user` as `(OpName :handler-keyword)` pairs |
| `:admin` / `:user` content | Operation signatures `[Op [args] -> RetType ...]` | **PAIR-LISTS** `[Op :handler-keyword ...]` — signatures derived from handlers via reflection |
| Handler return shape | "State only OR Tuple<State, V>" (flexible Model B) | **UNIFORM** `(:Tuple :State ...rest-vals)` — always Tuple, state always first |
| State framing | "the service's mutable state" | **STATE IS SELF** — Rust's `&mut self`; state monad's `s` explicit at type level |
| Source-of-truth for signatures | Triple (admin/user/handlers + defn) | **SINGLE** (handler defn; defservice reflects via `signature-of-defn`) |
| Substrate additions needed | TBD (predicted: validate_defservice_handlers helper) | **ZERO** — substrate already has every primitive needed |

### The handler contract (LOCKED — uniform; every handler)

```
[s <- :State, ...args] -> (:Tuple :State ...rest-vals)
```

- First binder MUST be `s <- :State` (state is self; threaded forward)
- Return MUST be `(:Tuple :State ...rest)` where rest is variable-arity (zero, one, or many)
- Rest empty → operation returns `:nil`
- Rest one type → operation returns that type
- Rest multi-type → operation returns `(:Tuple ...rest)`

defservice validates this at expand time via `:wat::runtime::signature-of-defn`. Violations panic at the defservice call_site_span with a teaching diagnostic.

### Why uniform (the lesson from this exchange)

Counter has the degenerate property that state == value (single i64). Earlier proposed a "Model B flexible" rule where state-only handlers could return just `:State` without Tuple-wrapping. The user corrected: **don't optimize the substrate's contract for the trivial case.** Real services have complex state and expose DERIVATIVES (nested fields, computed values, summaries) — never the whole state unless explicitly asked. The uniform `(:Tuple :State ...rest)` shape pays a small verbosity tax in Counter so the contract serves real services honestly.

Per `feedback_simple_is_uniform_composition`: N identical compositions IS simple. The Tuple-always rule is the simplest possible composition for the handler contract.

### State stays internal (transparency model)

The dispatch loop owns the live state. Handlers are pure transforms (per yesterday's handlers-are-monadic recognition):
- Caller sends Wire message → dispatch loop calls handler with current state → handler returns `(Tuple new-state ...rest)` → dispatch loop threads new-state forward + sends ...rest back to caller as the operation's response.

**Caller never receives state UNLESS the service author explicitly puts state in rest.** State exposure is a per-handler choice via rest-vals shape. The substrate doesn't force exposure; the handler declares it.

For Counter: state IS exposed (because state==value); on-get returns `(Tuple s s)`. For a real service: state stays internal because handlers expose only nested fields / derivatives via rest.

### Substrate primitives defservice uses (all verified present)

| Primitive | Purpose | Source |
|---|---|---|
| `:wat::runtime::signature-of-defn` | Look up handler's parsed signature at expand time | `src/check.rs:4859-4892` (arc 143) |
| `:wat::runtime::extract-arg-types` | Extract arg types from a signature HolonAST | arc 201 slice 5 |
| `:wat::runtime::extract-arg-names` | Extract arg names | `src/check.rs:4943` (arc 143 slice 3) |
| `:wat::holon::Bundle/children` / `Bundle/first` | Walk reflected signatures | arc 201 slice 2 |
| `:wat::core::atom-value` | Unwrap leaves | arc 057 |
| `:wat::core::keyword/of` | Synthesize service-namespaced keywords at expand time | `src/macros.rs:602-677` (arc 170 gap A) |
| Computed-unquote `~(:fn args)` | Evaluate helper fns at expand time | `src/macros.rs:1069-1097` (arc 143 slice 2) |
| `:wat::core::Option/expect` | Panic with diagnostic when signature lookup fails | arc 107 |
| `:wat::core::do` splice in macro expansion | Generate multiple top-level forms from one macro call | arc 170 Gap C + Gap J |

**Production precedent**: `wat/runtime.wat:17-32` (the `define-alias` macro) uses signature-of-defn + extract-arg-names + rename-callable-name + computed-unquote + Option/expect in EXACTLY the pattern defservice needs. Zero substrate additions required.

### The Rust convergence (eleventh great)

State as `(:Tuple :State ...rest)` arrived at the same shape as:
- Rust's `fn method(&mut self, args) -> Ret` — self threaded; return is Ret
- Haskell's State monad `s -> (s, a)`
- Erlang's `handle_call(Req, State) -> {reply, Reply, NewState}`

Eleven greats now: Kay + Erlang/OTP + Trio/Loom + Akka + nginx + Capnp + Clojure protocols + Clojure Component + Ruby Parallel + Go + Rust. Each arrived via different constraints; substrate forces them to converge. See INTERSTITIAL § 2026-05-18 convergence #13.

---

## Stone decomposition (FINAL — supersedes prior 2a/2b/2c/2d framings)

Per today's recognitions, the stones simplify because zero substrate Rust changes are needed for defservice itself (Stone C is pure wat). spawn-program defmacro + restricted_to application stay as substrate work; defservice is pure wat-side; counter migration is wat-tests authoring.

| Stone | Scope | Substrate touchpoint |
|---|---|---|
| **A — mint `:wat::kernel::spawn-program` defmacro** | `:tier :service state` dispatch via keyword-concat (`keyword/of`) → calls substrate-internal `:service::-start-tier`. Walker reshape: legacy 2-arg form stays rejected per `BareLegacySpawnProgram` (`src/check.rs:2476-2504`); new 3-arg `:tier :service state` form accepted by adding a typed-pattern arm. | `wat/kernel/spawn_program.wat` (NEW) + `src/check.rs` walker update |
| **B — apply `restricted_to :wat::kernel::` to raw spawn-*** | `:wat::kernel::spawn-thread` + `:wat::kernel::spawn-process` become substrate-internal. User code reaches them only via spawn-program (which lives in `:wat::kernel::` scope so the restriction permits the call). | `src/runtime.rs` eval-handler registrations + `#[restricted_to(...)]` proc-macro per arc 198 |
| **C — mint `:wat::service::defservice` defmacro** | Pure wat in `wat/service.wat`. Uses arc 201 reflection at expand time. Generates: typealias-references; enums (Req/Resp + Wire); capability structs; dispatch loop; per-op wrappers; substrate-internal `:service::-start-thread`/`-start-process` entries (restricted_to `:wat::kernel::` so only spawn-program calls them); expand-time handler-contract validation. | `wat/service.wat` (NEW) + small `src/runtime.rs` stdlib-loader edit |
| **D — counter migration proof** | Rewrite `wat-tests/counter-service-capability-N3.wat` as a defservice; verify the substrate-generated code passes the same tests the hand-rolled version did; ~75% line reduction expected. Becomes the canonical example for USER-GUIDE. | Pure wat-tests; no substrate touch |

Order: A → B → C → D. B+C may pair-commit atomically if B's restriction temporarily breaks C's tests during ship; per recovery doc § atomic-commit.

Each stone gets BRIEF + EXPECTATIONS + SCORE + atomic commit per protocol. Stone A is the foundation; drafting next.

---

## Compaction-recovery breadcrumb (2026-05-18 — supersedes 2026-05-17 breadcrumb above)

**Tip after this commit on `arc-170-gap-j-v5-deadlock-state`.** Arc 209 status:
- Slice 1: SHIPPED `f815c14` (audit + pure-defmacro decision)
- Surface: FINAL LOCKED in § "Surface settled 2026-05-18" above (collapsed shape + state-as-self contract)
- Substrate verified: zero additions needed for defservice itself (Stone C); Stones A + B touch substrate
- Stone A BRIEF: NOT YET DRAFTED (next move)
- Source of truth: this DESIGN (Surface settled 2026-05-18 section) + INTERSTITIAL § 2026-05-18 convergence #13

**Recovery instructions for post-compaction orchestrator:**

1. Read this DESIGN's § "Surface settled 2026-05-18" — the LOCKED surface
2. Read INTERSTITIAL § 2026-05-18 convergence #13 — the architectural narrative
3. Skim SCORE-SLICE-1.md for the audit's substrate-primitive verification
4. Verify state: `git log --oneline | head -10` should show this commit at tip
5. Next action: draft BRIEF-STONE-A for `:wat::kernel::spawn-program` defmacro mint + walker reshape. Stone A is foundation; spawn sonnet against the LOCKED surface (NOT the prior surface-locking sections which stay as historical record).
6. **Discipline reminders (all load-bearing):**
   - `feedback_sonnet_output_requires_review` — orchestrator-side architectural review on sonnet deltas
   - `feedback_inscription_immutable` — prior surface-lockings stay as historical record; forward-correct via new sections
   - `feedback_simple_is_uniform_composition` — Counter's verbosity is the right trade; substrate contract serves real services
   - `feedback_assertion_demands_evidence` — every substrate-claim needs grep before assertion
   - Pre-flight greps before drafting Stone A BRIEF
