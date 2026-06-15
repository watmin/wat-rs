# INSCRIPTION — arc 232 (defprotocol / extend-type)

**Closed 2026-06-14.** defprotocol/extend-type — Clojure's protocol, typed — exists end-to-end:
declare a protocol's single-receiver method signatures, extend a type to it, type a parameter over
the protocol bound, and call its methods with dispatch on the receiver's concrete type. The open
type-bound that wat lacked (proven necessary by a disconfirming probe: an abstract forwarded arg got
`NoMatchingClauseAtCallSite`) is now real.

## What shipped

- **`string::to-lowercase`** (`e2386205`) — the lowercase primitive, lifted into its own honest
  commit (a substrate gap surfaced mid-C.3; mirrors `to_lowercase`, pure+total, on the macro fence).
- **232.1 — forms + registry** (`203ece72`): `:wat::core::defprotocol` + `:wat::core::extend-type`
  as Rust special forms on the defclause mold — `parse_*_form` → `Value::wat__core__protocol_def` /
  `wat__core__extend_def` in `runtime_def_values` → `CheckEnv` `protocol_registrations` /
  `extend_registrations` via `from_symbols`. Single-receiver invariant enforced (method arg 0 typed
  `:P`). Anti-fake registry unit test.
- **232.2 — the satisfaction edge** (`5764cc20`): `extend-type :T :P` registers a subtype-parent
  edge `T → P` in `splice_type_decls`; a `:P`-typed param accepts any extender, rejects a
  non-extender. `assignable`/`is_subtype` UNCHANGED — the edge flows through the existing Path→Path
  consultation (no `TypeDef::Protocol`: the subtype hierarchy is orthogonal to the TypeDef registry).
- **232.3 — method dispatch** (`0fd47727`): `(<P>/<method> receiver args…)`, namespaced (Record-
  accessor precedent; four-Q decided). Check-time arm resolves via the method sig + the 232.2 edge;
  runtime dispatches on the receiver's `class_fqdn` via the extend registry → the impl `Clause`.
  Resolver exemption + protocol-name pre-registration (`resolve/walk.rs` + `freeze.rs` step 6.95) so
  the head survives to dispatch. Missing impl → a clean error naming the type + protocol, never a
  panic. Keystone probe: a `:P`-bound fn dispatches Robot→"beep", Dog→"woof" (concrete-type, general).

## Realization

- **The seats were already carved — the substrate keeps us in check** (`REALIZATIONS.md`): every
  collision in this arc pointed onto a path already laid (the unified Comm* layer, the parked
  defprotocol, `register_subtype`'s orthogonal-hierarchy comment written for records). reach-stumble
  (a gap → build it) and reach-find (a tool already shaped) are one mechanism, two faces; the tell is
  whether the probe fails (build here) or a pre-laid seat appears (you're home). The check runs
  through both: the substrate holds the structure, the builder holds where the flags are planted.
  (To fold into the 170 chronicle later, per builder.)

## Out of arc 232's scope (affirmative cuts — each scoped out cleanly)

- **Default method impls / protocol inheritance / Parametric protocols** — out of arc 232's scope;
  not tracked elsewhere because no caller has surfaced demand. If/when one does, a NEW arc opens;
  arc 232 commits to none of them.
- **The host consumer** — the `Host` protocol, `SpawnHandle` sum, `Endpoint` record, and host-
  agnostic defservice `start` are out of arc 232's scope; tracked in **arc 209** (defservice), which
  resumes ON this mechanism. Arc 232 ships the mechanism + a generic proof, not the host wiring.
- **Migrating the existing defclause kernel intrinsics to protocols** — out of arc 232's scope;
  tracked in **arc 256** (banked, generic defclause).
- **Bare-name call form** (`(greet r 3)`, Clojure style) — out of scope; the namespaced
  `(<P>/<method> …)` form was decided by four-Q (Obvious+Simple+Honest+UX) and is the wat-honest
  default. A bare-form arc opens only if Clojure-parity later demands symbol-head dispatch.

## Gate at close (Inquisitor's own re-run)

probe_arc232_1 1/1 · probe_arc232_2 2/2 · probe_arc232_3 1/1 (beep+woof+clean-missing-impl-error) ·
lib 917/36 (zero new beyond the pre-existing 36 baseline; +2 new passing tests this arc) ·
nursery 895/4 (baseline) · workspace compiles.
