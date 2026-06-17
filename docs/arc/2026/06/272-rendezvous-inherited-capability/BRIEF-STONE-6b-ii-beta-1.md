# BRIEF — Stone 6b-ii-β-1: the launch reshape (Launched + mint-internally + start unify), thread tier

> Single-hop sonnet Shadowdancer. Do NOT spawn sub-agents. Work only in `~/work/holon/wat-rs`. Commit
> nothing; the orchestrator weighs the diff + re-runs the gate. Grounded against HEAD `f24db63d`,
> branch `arc-170-gap-j-v5-deadlock-state`. Design: `DESIGN-STONE-6b-process-launch.md`.

## The work (one paragraph)

Today `defservice`'s `start` mints the listener in the PARENT (`(listener' host Op Reply)`,
`wat/service.wat:508`) and passes it to `Host/launch`. That's wrong for the process tier (the child must
mint its own listener — arc 272 6a). Reshape so listener-minting moves INTO `launch`, and `start` becomes
host-agnostic over the constant `spawn-program'` surface: `launch` mints (per tier) and returns a new
`Launched<S,R>{handle,address}` struct; `start` calls `Host/launch<Op,Reply>` with EXPLICIT type-args
(the arc-232 dep, shipped `986795d8`) and unwraps `Launched` into the `Handle`. THIS STONE does the THREAD
tier only; the ProcessOpts arm is β-2. Gate: the existing thread test
`probe_arc209_c3_defservice_client_face` stays GREEN through the reshaped path.

## Build

**1. `wat/spawn.wat` — `Launched<S,R>` struct** (sibling to `Bound`, near line 130):
```wat
;; Launched<S,R> — what Host/launch returns: the spawn handle + the dial address.
;; A STRUCT, not a record (address is an Address' RustOpaque; handle is :Spawned).
(:wat::core::defstruct :wat::spawn::Launched<S,R>
  [handle  <- :wat::spawn::Spawned
   address <- :wat::kernel::Address'<S,R>])
```

**2. `wat/spawn.wat` — reshape the `launch` protocol method** (`defprotocol :Host`, line 182):
```wat
(:wat::core::defprotocol :wat::spawn::Host
  (launch<S,R,St> [self   <- :wat::spawn::Host
                   state0 <- :St
                   serve  <- :wat::core::keyword] -> :wat::spawn::Launched<S,R>))
```
Drop the `listener` and `clients0` params (launch mints the listener; clients start empty).

**3. `wat/spawn.wat` — reshape the ThreadOpts `launch` arm** (line 193). Mint the listener internally via
the now-working `(listener' self :S :R)` (the dep: the method's type-params S,R flow as type-args), build
the serve closure capturing the minted listener + an empty clients vector + state0, spawn, return
`Launched`:
```wat
(:wat::core::extend-type :wat::spawn::ThreadOpts :wat::spawn::Host
  (launch [self state0 serve]
    (:wat::core::let
      [b  (:wat::kernel::listener' self :S :R)
       sp (:wat::kernel::spawn-program' self
            (:wat::core::fn [self-peer <- :wat::kernel::Peer'<R,S>] -> :wat::core::nil
              (:wat::core::apply -> :wat::core::nil serve self-peer
                (:wat::spawn::Bound/listener b)
                (:wat::core::Vector :wat::kernel::Peer'<R,S>)
                state0 [])))]
      (:wat::spawn::Launched sp (:wat::spawn::Bound/address b)))))
```
The body uses the method type-params `:S`/`:R` (`listener'`, the closure's `Peer'<R,S>`, the empty
`Vector Peer'<R,S>`). The dep proved `(listener' self :S :R)` works; the `Peer'<R,S>` spots are the same
mechanism (type-params as type-args, resolved at runtime as keywords + statically from the sig).

**4. `wat/service.wat` — reshape `start` codegen** (the `start-body` quasiquote, ~line 507). Drop the
parent-side `(listener' host Op Reply)` + the `l`/`addr`/`pair` binders. New body:
```wat
;; (defn <fqdn>/start [host <- :Host  state0 <- <state-ty>] -> <fqdn>::Handle
;;   (let [lr (:wat::spawn::Host/launch<Op,Reply> host state0 (keyword/from-string "<fqdn>::serve"))]
;;     (<fqdn>::Handle (Launched/handle lr) (Launched/address lr))))
```
The `Host/launch<Op,Reply>` call-head carries EXPLICIT type-args (Op/Reply are the `enum-name`/`reply-name`
fqdns the macro already holds). Build that head — a keyword call-head with a `<…>` suffix whose args are
the Op/Reply fqdns WITHOUT the leading colon (e.g. `<my::counter::Op,my::counter::Reply>`); mirror how the
macro builds other call-heads/keywords (`keyword/from-string`, `symbol-node`, `string::concat`). St is
inferred from `state0` (the dep binds the explicit args and fresh-vars the rest). Unwrap `Launched` via
`Launched/handle`/`Launched/address` into the `Handle` constructor.

## Rooms (read in order)
1. `wat/spawn.wat:117-198` — Spawned/Bound/Host/launch + the ThreadOpts arm (the shapes to mirror + reshape).
2. `wat/service.wat:483-547` — the `start` codegen + the final `do` assembly (how heads/keywords are built).
3. `tests/probe_arc209_c3_defservice_client_face.rs` — the THREAD gate (must stay green).
4. `tests/probe_arc232_generic_method_type_application.rs` — the dep probe (the `(:P/m<T,T> …)` + `(listener' self :S :R)` shape proven GREEN).
5. `wat/service.wat:50-110` — the macro's binder/keyword-building helpers (peer-ty, enum-name, serve-name-str).

## STOP triggers (halt + report; ship nothing)
1. STOP if building the explicit-type-arg call-head `Host/launch<Op,Reply>` in the macro needs a NEW
   node/keyword primitive (beyond `keyword/from-string`/`symbol-node`/`string::concat`) — report what's missing.
2. STOP if a body type-param spot (`Vector Peer'<R,S>` or the closure `Peer'<R,S>`) fails where
   `(listener' self :S :R)` works — that's a deeper type-param-in-body gap than the dep covers; report it
   precisely (do not work around with `:Any` or an untyped vector).
3. STOP if the reshape would change the `spawn-program'` public surface or the thread-tier client-face shape.

## Gate (orchestrator re-runs)
- `cargo test --release -p wat --test probe_arc209_c3_defservice_client_face -- --test-threads=1` → GREEN (5).
- `cargo test --release -p wat --test probe_arc272_6b_defservice_on_process -- --include-ignored --test-threads=1`
  → should ADVANCE past the "ProcessOpts not a :Host" error to a NEW failure (the missing ProcessOpts arm /
  child-forms — that's β-2; still RED, but a DIFFERENT red). Report the new error.
- `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → 929/36 (zero new).
- `cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"` → 893/4 baseline.
- `cargo build --release -p wat` → clean.

Report: exact files+lines changed, how you built the `Host/launch<Op,Reply>` head, the gate results from
your OWN runs (pasted), the NEW red error from the process probe, and any STOP hit.
