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
;; hook onto the handle's own addr). This lets a HETEROGENEOUS `Vector<:wat::capability::
;; Capability>` of different services' Handles be grant/revoke'd UNIFORMLY, AND dialed
;; uniformly — `coordinate` hands back the handle's dial address as a bare
;; :wat::kernel::Address', so ONE vector of handles carries both grant and dial. grant/revoke
;; return nil; coordinate returns the bare address.
(:wat::core::defsurface :wat::capability::Capability :nature :wat::core::Struct
  :features
  [(grant      [self <- :wat::capability::Capability  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)
   (revoke     [self <- :wat::capability::Capability  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)
   (coordinate [self <- :wat::capability::Capability] -> :wat::kernel::Address')])

;; `as-capability` (arc 170 N-service kwargs stone) — RETIRED. It forced a Handle's
;; up-cast to Capability at the scalar call boundary before tupling, because a `Tuple`
;; constructor call wasn't component-wise up-cast against its expected type. That gap is
;; closed (arc-check-literal-elems, generalized): `(:wat::core::Tuple k v)` now up-casts
;; each component against its expected position — including nested inside a spliced
;; `Vector<(keyword,Capability)>`, exactly `process/uses`'s (wat/spawn.wat) shape. Zero
;; consumers remain (whole-tree grep) — deleted rather than kept as dead surface.
