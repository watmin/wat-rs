# NOTE (HORIZON) — a service's return value IS its final state; the symmetry forces state to be wire-serializable

> Captured 2026-06-16, builder: *"the ret-val of the service is its final state… whoever holds the
> blocking call on serve returns their state (a thread, process or remote returns their final state
> object; which could be used as the starting state on the next start)… the implication is that state
> MUST be a record?"* FORWARD-LABELED design intent. NOT built — today `serve` returns `:wat::core::nil`
> on `:Shutdown` (`wat/service.wat`). This is the next evolution, after 6b closes.

## The design (the symmetry)

A service's lifecycle is **state in → run → final state out**:
- `state0` goes IN at `start` (for process/remote it crosses parent→child over the lineage — 6b-ii-α / B3).
- the **final state** comes OUT at shutdown: `serve`'s blocking loop returns its last `state` when
  `:Shutdown` fires (owner dropped the handle), instead of `nil`. Whoever holds the blocking call yields
  that final state object:
  - **thread** — the serve closure returns `St`; the owner gets it via the join.
  - **process** — the child `send'`s its final `St` back over the lineage before exiting; the parent
    `recv'`s it (symmetric with how state0 crossed IN). The `Handle`'s join/await yields it.
  - **remote** — same, over the mTLS channel ([[NOTE-remote-mtls-trust]]).

So `serve`'s return type becomes `:St` (not `:nil`), and `Launched`/`Handle` grow a way to await the
final state on shutdown. **Resumability falls out for free:** `final-state → next start's state0` — a
service is checkpointable/restartable across runs (and across hosts: drain a thread service, hand its
final state to a process service).

## The implication — state MUST be wire-serializable (structured state ⇒ a record)

Yes. The final state must travel child→parent over the lineage (process) and over the network (remote).
By the **record-vs-struct law** (`wat/spawn.wat:116`): a **record is EDN-serializable / wire-safe**; a
**struct holds non-EDN `RustOpaque` fields and can NEVER cross a wire**. Therefore:

- A service's `:state` must be **EDN-serializable** — a **record** (for structured/heterogeneous state),
  or an EDN scalar/collection (the counter's `:wat::core::i64` qualifies). It must **never be a struct**
  (a struct state could run on a thread but could not return its final state from a process/remote — it
  would break the moment the service left shared memory).

- **Parity forces this uniformly** (the narrow-waist law again — see
  [[feedback_four_questions_weigh_hard_constraint_parity]]): the SAME `defservice` must run on
  thread/process/remote behind one client face. So the state constraint can't be per-tier — a structured
  state must be a record for ALL tiers, or the service isn't host-agnostic. State-as-record is what makes
  "swap the host, same service" honest end-to-end (in AND out).

## The enforcement (the no-magic line)

This should be a **defservice CHECK**, not a convention: `:state` must be EDN-serializable, made
**uncompilable** otherwise ([[feedback_no_magic_that_lets_llm_fake_correctness]]). A struct `:state`
should fail at defservice-expansion with a diagnostic ("a service state must round-trip the wire — use a
record, not a struct"), so a lower-tier author cannot write a service that silently can't leave the
thread tier. The type makes the parity guarantee true, not discipline.

## Bar / sequence

Don't build until 6b closes (the process tier lands with `serve` still returning `nil`). Then this is its
own stone: `serve -> :St`, the final-state return over the lineage (symmetric with state0 in), the
`Handle` await, and the defservice `:state`-is-EDN check. Pairs [[project_rendezvous_inherited_capability]]
+ [[project_shared_memory_partition_hosting]] + NOTE-remote-mtls-trust + the record-vs-struct law.
