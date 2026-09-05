;; wat/capability.wat — the uniform capability surface (arc 170 capability circuit, stone A).
;;
;; Relocated + renamed from wat/service.wat (was :wat::service::Grantable, loaded at stdlib
;; position ~328). The bracket (wat/bracket.wat) and spawn.wat both need to name this surface,
;; so it must load EARLY — before wat/spawn.wat. It deps only on core.wat builtins
;; (defsurface, Struct, Vector, i64, nil).
;;
;; Capability (renamed from Grantable, stone A) — a struct-nature methods-surface every
;; service's `<fqdn>::Handle` satisfies (via the extend-type the defservice macro AUTO-EMITs,
;; routing to the landed <fqdn>/grant & <fqdn>/revoke methods, plus the up-cast `coordinate`
;; hook onto the handle's own addr). This lets a HETEROGENEOUS `(Vector :- [:wat::capability::
;; Capability])` of different services' Handles be grant/revoke'd UNIFORMLY, AND dialed
;; uniformly — `coordinate` hands back the handle's dial address as a bare
;; :wat::kernel::Address', so ONE vector of handles carries both grant and dial. grant/revoke
;; return nil; coordinate returns the bare address.
(:wat::core::defsurface :wat::capability::Capability :nature :wat::core::Struct
  :features
  [(grant      [self <- :wat::capability::Capability  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)
   (revoke     [self <- :wat::capability::Capability  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)
   (coordinate [self <- :wat::capability::Capability] -> :wat::kernel::Address)])

;; `as-capability` (arc 170 N-service kwargs stone) — RETIRED. It forced a Handle's
;; up-cast to Capability at the scalar call boundary before tupling, because a `Tuple`
;; constructor call wasn't component-wise up-cast against its expected type. That gap is
;; closed (arc-check-literal-elems, generalized): `(:wat::core::Tuple k v)` now up-casts
;; each component against its expected position — including nested inside a spliced
;; `(Vector :- [(keyword,Capability)])`, exactly `process/uses`'s (wat/spawn.wat) shape. Zero
;; consumers remain (whole-tree grep) — deleted rather than kept as dead surface.

;; Dialable :- [S R] (arc 170 W1) — a SECOND, PARAMETRIC surface every service's `<fqdn>::Handle`
;; also satisfies (via a second auto-emitted extend-type, wat/service.wat's dialable-extend,
;; beside grantable-extend). Where Capability/coordinate deliberately erases the service type
;; (bare :wat::kernel::Address', for the uniform heterogeneous (Vector :- [Capability]) grant/revoke
;; path), Dialable/coord returns the handle's own TYPED `(:wat::kernel::Address' :- [S R])` — so
;; `(Dialable/coord handle)` resolves per-satisfier to the concrete service address
;; ((Address' :- [Echo::Op Echo::Reply]) vs (Address' :- [Kv::Op Kv::Reply])), and a wrong-service dial is
;; a compile-time discrimination error. Proven by hand in
;; probe-c2-typed-coordinate.wat (pre-auto-emit, since deleted); this bakes the surface so
;; defservice can auto-emit satisfaction without a hand-written extend-type per service.
;; Method name `coord` (not `coordinate`) — deliberately distinct from Capability's
;; `coordinate` so a handle satisfying BOTH surfaces has no unqualified-call ambiguity; callers
;; already qualify by surface (`:wat::capability::Capability/coordinate` vs
;; `:wat::capability::Dialable/coord`), matching the probe's proven shape.
(:wat::core::defsurface :wat::capability::Dialable :- [S R] :nature :wat::core::Struct
  :features
  [(coord [self <- (:wat::capability::Dialable :- [S R])] -> (:wat::kernel::Address :- [S R]))])

;; TypedCapability :- [S R] (arc 170 C2 candidate D) — a THIRD, combined surface every service's
;; `<fqdn>::Handle` also satisfies, via a THIRD auto-emitted extend-type that is deliberately
;; BODILESS (wat/service.wat's typedcap-extend, beside grantable-extend/dialable-extend). It
;; unions Dialable's typed `coord` with Capability's `grant`/`revoke` under ONE surface so a
;; service handle can flow through the N-service kwargs-check TYPED (never erased to a bare
;; Value/Capability) and still be both dialed and granted through that one typed param. A
;; bodiless extend-type registers the satisfaction EDGE only (assignability) — it does NOT
;; re-declare grant/coord/revoke (that would collide with grantable-extend/dialable-extend's
;; own method bodies on the flat `<Type>/<method>` registration key, DuplicateDefine). Runtime
;; dispatch instead serves TypedCapability/coord|grant|revoke off the Handle's EXISTING
;; Capability+Dialable method bodies via that same flat key — a surface-method call resolves
;; by the RECEIVER's concrete type, regardless of which surface named it. Proven this session:
;; probe-v-bodiless.wat (freezes clean, no DuplicateDefine), probe-v-swap.wat (a
;; wrong-service handle is a located TypeMismatch), probe-v-run.wat (TypedCapability/coord
;; dispatches at runtime through the flat key). Method names reused verbatim from Dialable/
;; Capability (coord, grant, revoke) — safe because a handle only ever calls THROUGH one
;; qualified surface at a time; there is no unqualified-call ambiguity to resolve.
(:wat::core::defsurface :wat::capability::TypedCapability :- [S R] :nature :wat::core::Struct
  :features
  [(coord  [self <- (:wat::capability::TypedCapability :- [S R])] -> (:wat::kernel::Address :- [S R]))
   (grant  [self <- (:wat::capability::TypedCapability :- [S R])  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)
   (revoke [self <- (:wat::capability::TypedCapability :- [S R])  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)])
