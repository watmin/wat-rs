# DESIGN — record-state + final-state-return: a service's value is its final state

> Opened 2026-06-16. Grounded against HEAD `c3cf58b5`. Builds the design pinned in
> `NOTE-service-final-state-return.md`. Sequenced AFTER 6b locus-parity (shipped). defservice = arc 209;
> kept here for continuity with the locus-parity arc + the NOTE.

## The feature

A service's return value IS its final state. The blocking serve loop, on `:Shutdown`, returns its last
`state` (today it returns `nil`, `service.wat:355`). Whoever holds the blocking serve yields that final
state: thread via join, process/remote over the lineage. Resumability falls out:
`final-state → next start's state0`. The state, strictly, must be a **record** (the wire-conformant,
named-typed contract — `NOTE-service-final-state-return.md`).

## Decomposition (probe-first per stone)

### rs-1 — `:state` MUST be a record (base or holon-derived) — ✅ HARD REQUIREMENT (builder 2026-06-16); NOT blocked on arc 273
**HARD REQUIREMENT (builder 2026-06-16):** *"make it a hard requirement that a service's state must be a
record (base or holon derived)."* The earlier "blocked on arc 273" framing was WRONG — it conflated two
predicates. **arc 273** is the runtime VALUE predicate `record?` (`runtime.rs:3860` — true iff
`Value::wat__holon__Record`, holon-only); rs-1 does NOT need it. **rs-1 needs the TYPE-level check** "does
`:state-ty` name a record TYPE (base or holon-derived)" — and THAT ALREADY EXISTS + is in use:
`src/collection/infer.rs:378-381` = `is_subtype(ty, ":wat::Record", env) || is_subtype(ty,
":wat::holon::Record", env)` over the `subtype_edges` registry (`:wat::holon::Record` derives `:wat::Record`;
`TypeDef::Record` is a first-class variant, Stone S-B.1; `env.types().get(name)` resolves the keyword). So
rs-1 is **UNBLOCKED**. ⚠ OPEN (the stone's own design call): WHERE the check fires — defservice is a wat macro
expanded BEFORE check, and its output carries no "this is the service state" marker. Candidates: (a) defservice
emits a check-time assertion form (e.g. `(:wat::type::assert-record! state-ty)`) that the checker resolves +
validates against the registry; (b) a macro-time type-registry-query intrinsic (the "does a macro need it?"
boundary — [[feedback_does_a_macro_need_it_intrinsic_boundary]]). Probe-first to pick. (Intent + migration:)
#### The reasoning chain — why a CHECK, and why the type system does NOT catch this on its own

This was worked out with the builder 2026-06-16 (preserve it; it is the load-bearing *why*):

1. **The type system is SOUND — there is no type error to catch.** `:state :wat::core::i64` declares the
   state type as `i64`, a perfectly valid type. defservice generates `StopResponse[state <- i64]`,
   `start [state0 <- i64]`, `stop -> i64`, `serve [state <- i64]` — all well-typed; the rs-2 PROCESS probe
   *proves* an `i64` state round-trips across a socket. "State must be a record" is a **domain constraint we
   choose to impose** (conformance / resumability / evolvability), NOT a soundness property. The checker
   enforces what the code *declares*; nothing in the generated code declares "must be a record."

2. **defservice is a MACRO that monomorphizes user-defined state.** `serve` (and `start`, and every method)
   is a generated `defn`. It does **not** declare "I want a record" — it declares **"I want `~state-ty`"**, the
   *concrete* type the user chose. With `i64`, serve declares `[state <- :wat::core::i64]` — a true signature
   the checker rightly accepts (serve genuinely wants an i64). The constraint is therefore NOT on the
   generated defns (they are monomorphic and honest about the concrete type) — it is on the user's **choice**
   of `:state`, i.e. on the **macro's argument**.

3. **A macro-argument constraint has exactly ONE channel to the checker: an emitted form.** By the time the
   checker runs, the macro is gone and `:state` is just concrete `i64` woven through the code. There is no
   "service" marker and no surviving generic parameter. So the only way to state "your chosen state type must
   be a record" to the checker is for the macro to EMIT a form that says exactly that.

4. **It must be CHECK-time, not macro-expand-time** (phase order, `freeze.rs:691-695`): expand (step 4) →
   register types (step 5) → check (step 8). At expand time the type registry is empty AND records are
   themselves minted by macros (chicken-and-egg) — so the macro cannot inspect `state-ty`'s recordness as it
   expands. By check time (step 8) every type is registered. The emitted form is validated there.

5. **The (a) check vs (b) native-bound fork resolves to (a).** A native bound `∀S <: :wat::Record` *would*
   let the type system reject `i64` by subtyping — but that requires a surviving **bounded type parameter**,
   and the macro already monomorphized `S := i64` into concrete defns. Path (b) would mean making the state a
   real bounded generic parameter (a typed construct, not a macro) — a paradigm move, not this stone. Given
   defservice **is** a macro with **user-defined** state, the emitted check-time assertion is the honest
   bridge — NOT a hand-rolled duplicate of the type system, but the only surface where the constraint can be
   spoken. (Parallel: Rust `derive` macros can't resolve types either — same syntactic/pre-type boundary.)

#### The contract (pin)

A new **check-time** form `(:wat::type::assert-record! <type-keyword>)` — name is an intueri candidate
(`require-record` / `record-bound` are alternates; the `!` reads as "raises a check error"):
- **Check (`src/check.rs`):** recognized in the head dispatch; resolves the keyword against the `TypeEnv` and
  asserts `is_subtype(ty, ":wat::Record") || is_subtype(ty, ":wat::holon::Record")` (the exact pattern in
  `collection/infer.rs:378-381`). On failure → a `CheckError` ("a service's state must be a record (base or
  holon-derived); `<ty>` is not a record type"). Types to `:wat::core::nil`.
- **Runtime (`src/runtime.rs`):** a no-op → `nil` (the work is entirely at check time; it must still *eval*
  cleanly because it rides the generated `do`).
- **defservice (`wat/service.wat`):** emits `(:wat::type::assert-record! ~state-ty)` once in the final `do`.

#### Build + migration (the blast radius — substrate-as-teacher cascade)

rs-1 makes scalar state **uncompilable**, so all 12 existing `i64`-state service definitions break and migrate
to a single-field record (e.g. `(:wat::Record::def :…::CounterState [count <- :wat::core::i64])`,
`:state :…::CounterState`) IN THE SAME STONE — handlers wrap/unwrap the field (`(CounterState/count s)` to
read, `(:…::CounterState v)` to build). The files: `probe_arc209_c1`/`c2`/`c3`/`locus_agnostic_start`/
`naming_conversion`, `probe_arc265_acronym_registry`, `probe_arc272_6b_defservice_on_process`,
`probe_arc272_rs2_{thread,process}_stop_returns_final_state`, `probe_arc272_rs2_crash_surfaces_to_client`,
`wat-tests/service-locus-parity.wat`. The `probe_arc272_rs1` NEGATIVE case STAYS `i64` (un-ignore → GREEN: it
proves rejection); its positive case already carries a record.
- **Probe (RED, committed `03b47b93`):** `scalar_state_is_rejected` (#[ignore], RED) + `record_state_is_accepted`
  (GREEN). GREEN once the check fires + the examples carry record state.
- No serve/start change beyond the one emitted assert form.

### rs-2 — a service's value IS its final state — ✅ SHIPPED (a57b9f0b, 2026-06-16) via the `:Stop` terminal op
**Shipped by a DIFFERENT (better) mechanism than the framing below.** We did NOT make `serve` return `St`
and add a join/lineage `await`. Instead: the `:Stop` terminal op (gen_server `{stop, State}`) — the final
state comes back as a `:Stop` REPLY over the CLIENT connection (`(<svc>/stop c) -> <state-ty>`); `serve`
STAYS `-> :nil` (the state rides as the reply); constant-shape across thread/process/remote, no lineage
reshape, no new substrate. Probes: `tests/probe_arc272_rs2_{thread,process}_stop_returns_final_state.rs` +
`…_crash_surfaces_to_client.rs`. The original (now-superseded) framing is kept below for the record:

#### (superseded framing) `serve` returns the final state on `:Shutdown` + the THREAD await
- serve-body: `(:wat::spawn::ServiceEvent::Shutdown nil)` → `(:wat::spawn::ServiceEvent::Shutdown state)`;
  serve's return type `-> :wat::core::nil` → `-> :St`. (Confirm `poll'`/`match` allow the non-nil arm.)
- Thread delivery: the thread's serve returns `St` in-memory; the Handle gains an **await** that joins the
  thread and yields `St`. Crawl needed: does the thread `spawn-program'`/`Thread'` join return the body's
  value? If join is value-returning, thread await is `join → St`. The `Launched`/`Handle` grows an await
  field/accessor.
- **Probe (RED):** a THREAD service; after shutdown, `(<svc>/await h)` returns the final state record
  (e.g. increment 5 then await → `CounterState{count 5}`). Thread only.

### rs-3 — the PROCESS await + unified await — ❌ REJECTED (builder 2026-06-16)
**Cut, not deferred.** rs-3 was "await the final state *on owner-drop*" — recover a service's dying state
when the owner just DROPS the handle (RAII), which on the process tier would need the child to push `St` up
the lineage at exit (the `ServiceUp` tagged-channel reshape of the proven 6a/6b handoff). REJECTED because
the **trigger already encodes intent, with no third case**:
- **Want the final state** → call `(<svc>/stop c)` → it comes back as the `:Stop` reply (rs-2, SHIPPED).
- **Don't care** → drop the handle; the EXISTING owner-drop → `ServiceEvent::Shutdown` path
  (`wat/service.wat:355`) reaps it cleanly. Proven green by the arc-209 c3 probe, which never calls `stop`
  and lets the handle drop — if drop-shutdown hung, c3 would hang.

"I dropped it but also want its dying words" is a contradiction in intent. So the `ServiceUp` reshape
evaporates, there is no `await` verb, no lineage reshape — and **the final-state feature is COMPLETE with
rs-2 alone**. Not tracked elsewhere; nothing deferred.

## Open decisions (pin before each sub-stone)
- **Shutdown trigger for await:** today owner-drop → `:Shutdown` (fire-and-forget). `await` must trigger
  shutdown AND block for the final state — so `await` drains the owner link (like drop) then waits for
  serve's return / the `:Final` message. (vs a self-stop `:Stop` Outcome — the banked C.4 terminal op;
  await covers owner-initiated; `:Stop` is separate.)
- **`Launched`/`Handle` shape:** gains an await path. For thread, the join handle; for process, the
  lineage peer to recv `:Final`. Keep the constant interface (per-tier await arm, narrow-waist).

## Pairs
[[NOTE-service-final-state-return]] + the record-vs-struct law + [[feedback_no_magic_that_lets_llm_fake_correctness]]
(the check) + [[project_shared_memory_partition_hosting]] (thread join vs process lineage) +
DESIGN-STONE-6b-ii-beta-IDEALIZED (the per-tier-arm pattern the await mirrors).
