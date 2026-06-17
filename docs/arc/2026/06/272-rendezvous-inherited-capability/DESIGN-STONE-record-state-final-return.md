# DESIGN — record-state + final-state-return: a service's value is its final state

> Opened 2026-06-16. Grounded against HEAD `c3cf58b5`. Builds the design pinned in
> `NOTE-service-final-state-return.md`. Sequenced AFTER 6b host-parity (shipped). defservice = arc 209;
> kept here for continuity with the host-parity arc + the NOTE.

## The feature

A service's return value IS its final state. The blocking serve loop, on `:Shutdown`, returns its last
`state` (today it returns `nil`, `service.wat:355`). Whoever holds the blocking serve yields that final
state: thread via join, process/remote over the lineage. Resumability falls out:
`final-state → next start's state0`. The state, strictly, must be a **record** (the wire-conformant,
named-typed contract — `NOTE-service-final-state-return.md`).

## Decomposition (probe-first per stone)

### rs-1 — `:state` MUST be a record (the guard) + migrate the examples — ⛔ DEFERRED onto arc 273
**Blocked + deferred (2026-06-16):** the check needs a TYPE-level record predicate (is `:state` a
registered record type) — and the crawl found `record?` is a VALUE predicate that ALSO only recognizes
HOLON records, not base records (`runtime.rs:3860`). Both gaps are stubbed as **arc 273 (record reflection
completeness)**; rs-1 builds when 273 lands (or sooner if it becomes a now-thing). The FEATURE (rs-2/rs-3)
does NOT depend on rs-1 — it works with any EDN-serializable state — so the guard waits without blocking
the feature. (Original intent retained below for when rs-1 activates:)
defservice CHECK: `:state` must resolve to a **registered record type** — uncompilable otherwise (a
scalar/collection/struct `:state` fails at expansion with a diagnostic). The no-magic line: a
structureless state can't be written down. Migrate the counter examples (`probe_arc209_c3_*`,
`probe_arc272_6b_defservice_on_process`, `wat-tests/service-host-parity.wat`) from `:state :wat::core::i64`
to a record, e.g. `(:wat::Record::def :…::CounterState [count <- :wat::core::i64])`, `:state :…::CounterState`.
- **Probe (RED):** a defservice with `:state :wat::core::i64` → expansion error "service state must be a
  record". GREEN once the check fires + the examples carry a record state.
- Self-contained; no serve/await change. Build FIRST (it forces records, which rs-2/rs-3 need).

### rs-2 — `serve` returns the final state on `:Shutdown` + the THREAD await
- serve-body: `(:wat::spawn::ServiceEvent::Shutdown nil)` → `(:wat::spawn::ServiceEvent::Shutdown state)`;
  serve's return type `-> :wat::core::nil` → `-> :St`. (Confirm `poll'`/`match` allow the non-nil arm.)
- Thread delivery: the thread's serve returns `St` in-memory; the Handle gains an **await** that joins the
  thread and yields `St`. Crawl needed: does the thread `spawn-program'`/`Thread'` join return the body's
  value? If join is value-returning, thread await is `join → St`. The `Launched`/`Handle` grows an await
  field/accessor.
- **Probe (RED):** a THREAD service; after shutdown, `(<svc>/await h)` returns the final state record
  (e.g. increment 5 then await → `CounterState{count 5}`). Thread only.

### rs-3 — the PROCESS await (the wrinkle) + the unified await
- The child's serve returns `St`; the child must send `St` **up** the lineage at shutdown. But the
  self-peer `S` is already `Address'` (the 6a startup handoff). Two types up the same channel = the
  conflict. **Candidate:** a tagged up-channel — `S = ServiceUp<…>` (`:Address [a]` at startup |
  `:Final [s <- St]` at shutdown); parent `recv'`s `:Address` to connect, and the await `recv'`s `:Final`
  → `St`. (Alternative considered: a separate fd / the process output channel — weigh in rs-3's design.)
  This reshapes the child main's self-peer type + the 6a handoff sites; do it author-adjacent.
- Unify `await` across tiers: `(<svc>/await h) -> St` — thread joins, process recv's `:Final`. The Handle
  carries enough to dispatch (the `Spawned` handle is per-tier; await dispatches like launch).
- **Probe (RED):** a PROCESS service; after shutdown, await → the final state record (over the lineage).
- HARDEST; flag the `ServiceUp` reshape for builder confirm before building (it touches the proven 6a/6b
  handoff). Probe-first.

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
