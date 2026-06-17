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

## The implication — state MUST be a RECORD (stricter than "EDN-serializable")

Yes — and **be strict about it: a record, not merely an EDN value.** The final state must travel
child→parent over the lineage (process) and over the network (remote), so wire-serializability is
necessary — but it is NOT the bar. An `int` happens to be EDN, yet it is **structureless**: no named
fields, no declared shape, no conformance. A **record** carries **strong conformance** — every field
named and typed, a malformed shape **uncompilable** (the no-magic / typed-record law,
[[feedback_no_magic_that_lets_llm_fake_correctness]]). Tying a service to **/some record/** makes its
state a **first-class typed contract** — exactly what state that checkpoints, resumes, evolves (add a
field), and crosses hosts must be.

- A service's `:state` must be **a record** — full stop. NOT a bare scalar (an `i64` is EDN but not a
  record → rejected), NOT a collection, and NEVER a struct (a struct holds non-EDN `RustOpaque` fields
  and can't cross a wire at all — `wat/spawn.wat:116`). The state is *some specific record type*; that
  record is the contract for BOTH `state0` (in) and the final state (out).

- **Parity forces this uniformly** (the narrow-waist law — see
  [[feedback_four_questions_weigh_hard_constraint_parity]]): the SAME `defservice` runs on
  thread/process/remote behind one client face, so the state contract can't be per-tier. State-as-record
  is what makes "swap the host, same service" honest end-to-end (in AND out).

- ⚠ **Migration consequence:** the counter examples use `:state :wat::core::i64` (a bare scalar — the
  loose form this note now forbids). When this stone lands, they migrate to a record state (e.g.
  `:my::counter::CounterState {count <- :wat::core::i64}`), and `serve`/start/the final-state return all
  carry that record. (β-2 ships with the scalar state still accepted — the check is THIS stone, not β-2.)

## The enforcement (the no-magic line)

A **defservice CHECK**, not a convention: `:state` must resolve to a **registered record type**, made
**uncompilable** otherwise. A scalar/collection/struct `:state` fails at defservice-expansion with a
diagnostic ("a service state must be a record — it is the typed, wire-conformant contract for state0 in
and the final state out"). A lower-tier author cannot write a service whose state is structureless or
can't leave the thread tier. The type makes the contract true, not discipline.

## Bar / sequence

Don't build until 6b closes (the process tier lands with `serve` still returning `nil`). Then this is its
own stone: `serve -> :St`, the final-state return over the lineage (symmetric with state0 in), the
`Handle` await, and the defservice `:state`-is-EDN check. Pairs [[project_rendezvous_inherited_capability]]
+ [[project_shared_memory_partition_hosting]] + NOTE-remote-mtls-trust + the record-vs-struct law.

## PRIOR-ART COLLISION — Erlang/OTP gen_server terminate + supervised state handover

**Surfaced 2026-06-16** (builder, on the `start`/`stop`-returns-final-state form: *"wut — we just
stumbled into erlang's tooling again? outstanding"*). This DEEPENS the already-noted collision
(arc-209 C.2 REALIZATIONS: *defservice ≡ gen_server at both ends* — the loop + the `Outcome` callback-return
`{reply,R,S}|{noreply,S}|{stop,…}`). The final-state-return adds the **lifecycle end**:

- **`gen_server:terminate(Reason, State)`** — OTP's shutdown callback RECEIVES the final `State`. Our
  `(<svc>/stop h) -> St` is that, handed to the owner: terminate-with-state, returned not just logged.
- **The `:Stop` Outcome** (banked C.4) = gen_server's `{stop, Reason, State}` handler return — the
  self-initiated termination, distinct from owner-initiated `stop`.
- **Resumability** (`final-state → next start's state0`) = OTP **supervised state recovery / takeover** —
  a child restarts and is handed (a derivation of) its prior state; hot-handover across nodes. Our
  version: drain a service, hand its final state to the next `start` — on the SAME or a DIFFERENT host.
- **state-as-record** = OTP's structured `State` term, made a typed, wire-conformant contract.

**What is genuinely ours** (the substrate guarantees *around* the textbook model): OTP's stateful-server
lifecycle on a **typed-ADT-on-Rust** substrate where the State is a typed RECORD that crosses thread /
process / remote via the ONE EDN capability — so resumability + handover fall out of the **wire-conformant
state itself**, no bespoke persistence/handoff layer; and **host-parity** (the same lifecycle on every
tier) comes from the narrow-waist + the per-host `launch` arm, where OTP gets it from the BEAM VM. Another
`WE-LAND-ON-THE-GREATS` beat: high taste + first-principles derivation re-deriving OTP's gen_server.

**Date:** 2026-06-16. Pairs [[feedback_note_prior_art_collisions]] + arc-209 REALIZATIONS (defservice ≡
gen_server) + the record-vs-struct law.
