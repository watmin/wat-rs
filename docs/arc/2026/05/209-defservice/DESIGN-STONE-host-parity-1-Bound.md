# DESIGN — Stone host-parity-1: `Bound<S,R>` (listener' returns a named struct, not a bare tuple)

> Arc 209, the **protocol-tooling leg** (host-agnostic `start` on `defprotocol`), sub-stone 1 of 5:
> `Bound` → `SpawnHandle` → `listener'` leveling → `Host` protocol → generic `start`.
> Grounded against HEAD `7051338c` (branch `arc-170-gap-j-v5-deadlock-state`).

## Why

`listener'` (thread tier) returns `Tuple<Listener'<S,R>, Address'<S,R>>`, destructured by callers
with positional `first`/`second`. A bare tuple is anonymous structure — "tuple until it deserves a
name." It deserves a name now: the thing `listener'` mints is the **listening state** — a server
end (`Listener'`) plus the address clients dial (`Address'`). Naming it as a struct (a) gives the
fields meaning (`Bound/listener`, `Bound/address`) over fragile positions, and (b) is the shape the
later `listener'` *leveling* (sub-stone 3) makes uniform across tiers.

## What it delivers

A parametric struct minted in `wat/spawn.wat` (sibling to the `ServiceEvent<I,O>` defenum already
there), returned by `listener'`'s **thread tier**:

```wat
(:wat::core::defstruct :wat::kernel::Bound<S,R>
  [listener <- :wat::kernel::Listener'<S,R>
   address  <- :wat::kernel::Address'<S,R>])
```

After this stone, the thread-tier `listener'` call site reads:

```wat
[b    (:wat::kernel::listener' (:wat::spawn::thread) :Op :Reply)
 l    (:wat::kernel::Bound/listener b)
 addr (:wat::kernel::Bound/address  b)]
```

## Why a STRUCT, not a record (the one contract decision)

`Bound`'s fields are `Listener'`/`Address'` — `RustOpaque` kernel entities, **not EDN-expressible**.
The builder's design intent: *structs hold non-EDN things; records hold EDN data*. Direct precedent
in this exact file — `ServiceEvent :Connection [peer <- :wat::kernel::Peer'<I,O>]` (spawn.wat:103)
carries an opaque `Peer'` field, and the shipped defservice `Handle` record carries
`addr <- :wat::kernel::Address'<…>` (service.wat:522). So opaque parametric field types resolve.
(Records-aren't-parametric is the *orthogonal* arc 266 question; it does **not** drive this — even if
records were parametric, `Bound` is a struct because its contents are non-EDN.)

## Scope — thread tier ONLY

`listener'` is asymmetric at HEAD: thread tier returns `Tuple<Listener',Address'>`; **process tier
returns a bare `Listener'`** (runtime.rs:18721 — it binds a named UDS, no address minted). This stone
changes ONLY the thread tier (the tuple → `Bound`). The process tier is untouched; making it mint an
address + return `Bound` uniformly is sub-stone 3 (`listener'` leveling). **Rejected (out of scope):
process-tier changes, `SpawnHandle`, any `Host`/`start` work.**

## The three edits + the migration

1. **`wat/spawn.wat`** — add the `defstruct` (after the `ServiceEvent` defenum, ~line 107).
2. **`src/check.rs`** — `listener_tuple(s,r)` (the helper, check.rs:10361) returns
   `TypeExpr::Parametric { head: "wat::kernel::Bound", args: [s, r] }` instead of the 2-tuple.
   Rename it `bound_type`. It is called at 5 sites (10270, 10285, 10289, 10349, 10355) — the
   thread-tier success path (10289) and four error-recovery fallbacks; all should yield `Bound<S,R>`.
   The **process-tier** return (`Listener'<S,R>`, ~10290+) is NOT this helper — leave it.
3. **`src/runtime.rs`** — `eval_listener_prime` thread tier (runtime.rs:18752) returns
   `Value::Struct(Arc::new(StructValue { type_name: ":wat::kernel::Bound".into(), fields: vec![<listener opaque>, <address opaque>] }))`
   instead of `Value::Tuple`. The process tier (18721, bare `Listener'`) is untouched.
   Precedent for Rust building a wat struct: `Value::Struct(StructValue{ type_name: ":wat::kernel::Thread", … })` (runtime.rs:20347) — leading colon in `type_name`.
4. **Migrate all thread-tier callers** off `first`/`second` (mandatory — `first`/`second` do not
   work on a struct, so un-migrated callers break):
   - `wat/service.wat:509-510` (the defservice `start` template: `~l-sym (first ~pair-sym)` /
     `~addr-sym (second ~pair-sym)` → `Bound/listener` / `Bound/address`) + its comment block 488-494.
   - `tests/probe_arc209_c2_defservice_dispatch.rs` (`first pair`/`second pair`).
   - `tests/nursery/probe_arc209_c0b1b_select_listener.rs:74-75` (`first pair`/`second pair`).

## Probe

`tests/probe_arc209_bound_listener.rs` (committed RED) — c0b1b reduced to one client with exactly two
accessor swaps; isolates the failure to `Bound`. RED at HEAD: `Bound/*` unresolved → `addr` is an
unbound var → `connect'` rejects it. GREEN when the three edits land.

## Done is done

No deferrals. The migration is in-strike (all three callers). Process-tier leveling is the *named*
next sub-stone (3), not a deferral of this one.
