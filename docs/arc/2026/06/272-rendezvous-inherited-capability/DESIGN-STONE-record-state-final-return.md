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
defservice CHECK: `:state` must resolve to a **registered record type** — uncompilable otherwise (a
scalar/collection/struct `:state` fails at expansion with a diagnostic). The no-magic line: a
structureless state can't be written down. Migrate the counter examples (`probe_arc209_c3_*`,
`probe_arc272_6b_defservice_on_process`, `wat-tests/service-locus-parity.wat`) from `:state :wat::core::i64`
to a record, e.g. `(:wat::Record::def :…::CounterState [count <- :wat::core::i64])`, `:state :…::CounterState`.
- **Probe (RED):** a defservice with `:state :wat::core::i64` → expansion error "service state must be a
  record". GREEN once the check fires + the examples carry a record state.
- Self-contained; no serve/await change. Build FIRST (it forces records, which rs-2/rs-3 need).

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
