# SCORE — STONE 255.21 (C-b1b): STOPPED at STOP-1; a live hole witnessed on main

**Executor commit `6173285bc`.** It adds only the witness
`wat-scripts/scratch-pad/255-21-coord-claims-either-transport.wat`; the surface change was reverted.
Weighed by the orchestrator on 2026-09-24. Floor `.floor/2026-09-24T07-15-53Z`: `6042 passed`.

## ⛔ The hole, re-measured on main

The witness `--check`s **rc=0**. On `main`:

- a **thread**-locus service handle's `(Dialable/coord h)` passes as
  `(Address :- [Op Reply :wat::kernel::Wire])`;
- a process handle's `coord` passes as `Shared`.

The rule `a6da457e3` made a type error (a process must not hold a shared-memory address) can be broken
through the capability surface. Today `coord` returns a 2-argument `Address`, and the missing-slot arms
accept any transport.

## Why the fix could not land (the code won)

With `Dialable :- [S R T]` and the Handle's `T` bound into the edge target:

1. **Nothing binds the edge's `T` to the receiver's transport.** `satisfier_method_keys` finds the scheme
   under the letter-rewritten key `(Handle :- [:T])/coord`, but the 118.B2d mapping needs the receiver's
   arity to equal the surface's (1 against 3). `coord` returns `(Address :- [… :T])`, and
   `transport_param_instantiates` admits `:T` as either transport. **All four claims are still
   accepted.**
2. **A concrete handle is refused at a 3-argument `Dialable` parameter.** The edge is registered under the
   exact string `(Handle :- [:T]) <: (Dialable :- [Op Reply :T])`, and `transport_edge_keys` rewrites
   only the actual side.
3. `wat/core.wat:1155`, the kwargs `capswap`, mints `(TypedCapability :- [S R])` from `(Peer :- [S R])`.
   Peer has no transport.

The census with the change applied showed STOP-8 on 13 files.

## The root, one level down: `extend-type` has no binder

`(extend-type :wat::core::Vector (Seqable :- [T]))` (`wat/seq.wat:83`) and every defservice edge
`(Handle :- [T])` use a `T` **nothing declares**. It is a type parameter by spelling alone, the same root
as `is_type_param_letter`. C-b3 ("match a generic edge by unification") needs to know **which names in an
edge are its parameters**, and today only the spelling says.

## Consequence for the order

**C-b3 precedes C-b1b.** C-b3 itself first needs `extend-type` to **declare** its type parameters.
C-b1b's rows (thread-claimed-Wire refused, etc.) become satisfiable only after both.

## Brief errors, recorded

- "≈68 annotations across ≈24 files" counted prose. The live type-position sites are about 15
  (itemised in the executor report).
- The rows assumed "Shared accepted / process accepted" were twins of a refusal. Before C-b3, **every**
  transport claim is accepted, so those rows could not discriminate.
