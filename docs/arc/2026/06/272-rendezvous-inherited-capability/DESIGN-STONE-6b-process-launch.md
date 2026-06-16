# DESIGN — Arc 272 step 6b: the ProcessOpts `Host/launch` arm (the process service is born)

> Opened 2026-06-16. Grounded against HEAD `10c97f1e`. Continues
> [DESIGN-STONE-6a-capability-handoff.md] (6a-i shipped `f35bcfb5`: the child-mints capability
> handoff is proven — `tests/probe_arc272_6a_capability_handoff.rs` GREEN) and arc-209
> [DESIGN-STONE-host-parity-4a-start.md] (the thread `launch` arm + the host-blind `start` codegen,
> both shipped). 6b builds the **process** arm so a defservice runs on a forked OS process the same way
> it runs on a thread — closing 272 → defservice host-parity → the deftest flip.

## Why this change

`start` is already host-blind (`service.wat:483-517` routes through `:wat::spawn::Host/launch`, passes
`serve` by runtime keyword). The `Host` protocol exists (`spawn.wat:182`) with the **thread** `launch`
arm built (`:193` — captures a closure over the in-memory listener). The **ProcessOpts arm does not
exist** — `spawn.wat` ends at the thread arm. Until it does, `(start (process) state0)` cannot launch a
service: a closure can't cross a fork.

## The crux (grounded, the fact that decides the design)

A forked child's universe is **baked stdlib + the spliced forms, and nothing else** —
`run_forms_as_server_child` (`verbs.rs:426-429`) calls `startup_from_forms`, which registers the baked
stdlib ahead of the passed forms (`freeze.rs:789-795`). The parent's defservice expansion — `<fqdn>::serve`,
the `<fqdn>::Op`/`Reply` enums, the state record — **does not cross**. So the process child must carry
the service's code *inside its forms*, and its runtime `state0` must arrive *over the wire*.

## The contract decisions (four-questions verdict, informed — see § Rejected for the losers)

1. **A1 — the service's code crosses as a defservice-emitted forms bundle.** defservice (which owns the
   defns at expansion) emits `<fqdn>::child-forms : Vector<wat::WatAST>` — the `Op`/`Reply` enums, the
   state record, `serve`, and a `:user::main` driver — for the ProcessOpts arm to ship via the constant
   2-arg `spawn-program'`. (Thread keeps capturing; process ships forms — the shared-memory partition,
   [[project_shared_memory_partition_hosting]].)
2. **B3 — `state0` crosses as a message over the lineage channel** (the self-peer), the same channel the
   minted `Address'` already rides (6a). `state0` is EDN-representable (the record-vs-struct law;
   records round-trip on the wire since 234.7a/b). No `env-fn` repurpose, no value→AST quoting.
3. **`launch` returns `Launched<S,R>{handle, address}`** (a new stdlib struct) and mints the listener
   *internally*, so `start` stays host-agnostic over the constant `spawn-program'` surface. `start`
   unwraps it into `(Handle svc addr)`. (It cannot return the per-service `Handle` — that's
   defservice-generated, invisible to the stdlib protocol.)

## The algorithm

**The ProcessOpts `launch` arm (parent side), child-mints + lineage handoff:**
```
launch (process) state0 serve  =>
  svc  = (spawn-program' (process) <child-forms>)   ; constant 2-arg surface
  addr = (recv' svc)                                ; blocks until child autobinds + reports (6a)
  _    = (send' svc state0)                          ; B3: hand the child its initial state
  Launched{ handle: svc, address: addr }
```
**The child `:user::main` (inside `<fqdn>::child-forms`), the mirror of 6a + the serve loop:**
```
b     = (listener' (process) :S :R)   ; autobind, no name (child mints — 6a)
self  = (self-peer :Address'<S,R> :St-or-wire)
_     = (send' self (Bound/address b))   ; hand the parent the capability (lock-step)
state0= (recv' self)                      ; B3: receive initial state
(serve self (Bound/listener b) (Vector) state0)   ; the poll' loop (same serve as thread)
```
`serve`'s shape is unchanged: `(serve self-peer listener clients state) -> nil`. The thread arm already
invokes it by keyword via `apply`; the process child invokes the same `serve` (now *defined in its own
universe* via the child-forms bundle).

## `Launched<S,R>` (new stdlib struct, sibling to `Bound`)

```wat
;; Launched<S,R> — what Host/launch returns: the spawn handle + the service's dial address.
;; A STRUCT, not a record: `address` is an Address' RustOpaque (non-EDN); `handle` is :Spawned.
(:wat::core::defstruct :wat::spawn::Launched<S,R>
  [handle  <- :wat::spawn::Spawned
   address <- :wat::kernel::Address'<S,R>])
```

## Files touched

- `wat/spawn.wat` — `Launched<S,R>` defstruct; the `launch` signature reshape (drop the `listener`/
  `clients0` params, mint internally); the ProcessOpts `extend-type … launch` arm.
- `wat/service.wat` — `start` codegen: stop minting the listener in `start`; call the reshaped `launch`;
  unwrap `Launched`. Emit `<fqdn>::child-forms` (the process forms bundle).
- `tests/probe_arc272_6b_process_service.rs` — 6b-i disconfirming probe (hand-rolled, no defservice).
- `tests/probe_arc209_c3_*` — the deftest counter proof gains a `(process)` arm (6b-iii).

## Out of scope = rejected (the four-questions losers)

- **A2 — child re-runs the parent's whole program source.** A new inherit-source spawn mode; collides
  with `fork_program_from_source` (slated to die, arc-213 — [[feedback_dont_patch_the_grave]]); may
  re-fire parent top-level effects. Obvious/Simple/Honest all NO. CUT.
- **B1 — `state0` via `env-fn`/user.program.** Repurposes the config-as-typed-fn channel (3b-e) to ship
  runtime state; needs state0 serialized to a source string. Obvious NO. CUT (env-fn stays the config
  mechanism, not the state-transport).
- **B2 — splice `state0` into the child forms as an AST literal.** Needs a runtime value→AST quote
  primitive; the macro builds forms at expansion but state0 is a runtime arg, so it can't anyway. CUT.

## The blocking dep — CONFIRMED, 6b-ii-β is blocked on it (2026-06-16)

The reshaped `launch` mints `(listener' self :S :R)` *inside a generic method body*, instantiating the
method's own type-params `<S,R>` as type-args to the `listener'` intrinsic. **PROBED at HEAD `611d68e3`
(`/tmp/tparam_probe.wat`) — RED:** a generic protocol method called with explicit type-args resolves as
`unknown callee: :user::Mk/mk<wat::core::i64,wat::core::i64>`, and (per the 4a probes, still true) the
implicit form treats `:S`/`:R` as the literal type `Path(":S")`. So the launch-mints-internally shape is
blocked on an unbuilt capability: **generic-method type-argument application** (call `:P/m<T,T>` + flow
the type-params into the body's intrinsic type-args).

**Four-questions, with PARITY as a hard criterion (zero central edit for a new transport):** the
alternative — defservice generating per-tier programs so `launch` need not mint — was rejected. It keeps
`start`/`spawn-program'` constant but makes the *transport seam* central: adding remote would edit
defservice's codegen, not just add an `extend-type`. That fails the narrow-waist requirement (Honest/UX).
So there is **no contention**: only the generic-method shape gives the constant `launch<S,R,St>` interface
where a new transport is one `extend-type` impl, zero central edit. The dep's `Simple? = NO` means
**decompose** (block-and-build it as its own stone — [[feedback_deferred_dep_becomes_necessary_block_and_build]]),
not abandon. See `DESIGN-STONE-6b-DEP-generic-method-type-application.md`.

## Decomposition (sub-stones)

- **6b-i — ✅ DONE** (probe GREEN). The disconfirming probe surfaced the real gap: the socket-tier
  `recv' self` arm decoded with NO type registry (`peer.recv()`), so the child raised `NoTypeRegistry`
  on the crossed `#user/Counter` record and exited — the parent→child *send* worked; the *receive*
  side was broken. 6b-ii-α fixed it: socket-tier `recv'` now does `recv_wire()` + `decode_trusted_wire(sym.types())`
  (new `Receiver::recv_wire_raw` + `Peer::recv_wire`), mirroring the PROCESS arm. lib 929/36, all sibling
  socket-tier probes GREEN.
  - Original framing (kept for the WHY): isolating the ONE genuinely-unproven bit. Already proven
  on disk (do NOT re-probe): the process `poll'` serve loop + owner-drop termination
  (`probe_arc209_c0b3aii_process_service_loop.rs`, GREEN); a record crossing the fork **child→parent**
  over the lineage channel (`probe_arc272_6c2_record_ipc_derisk.rs`, GREEN). The gap B3 needs is the
  **parent→child** direction over the lineage channel — `(send' svc state0)` reaching the child's
  `(recv' self)`. Every existing test sends child→parent over the lineage and parent→child only over a
  *separate socket*. So 6b-i: child autobinds → sends addr (proven) → `recv' self` for `state0` (NEW) →
  serves threading it; parent recv's addr → `send' svc state0` (NEW) → connects → drives one op whose
  reply is derived from `state0`, proving the state actually crossed. RED isolates exactly the
  parent→child lineage send (+ its `send'`/`recv'` inference).
- **6b-ii** — probe the type-param-instantiation gap; build `Launched`, the `launch` reshape, the
  ProcessOpts arm, the `<fqdn>::child-forms` emission, the `start` unify.
- **6b-iii** — the deftest counter proof gains a `(process)` arm (parity with thread).

Pairs [[project_rendezvous_inherited_capability]] + [[project_shared_memory_partition_hosting]]
+ ZERO-MUTEX (the addr + state handoff IS the synchronization) + [[feedback_author_adjacent_prime_drop]]
+ DESIGN-STONE-6a-capability-handoff + arc-209 DESIGN-STONE-host-parity-4a-start.
