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

;; ── as-capability — the explicit scalar up-cast (arc 170 N-service kwargs stone) ──
;; The checker up-casts a concrete Capability-deriving value (a `<fqdn>::Handle`) to
;; `:wat::capability::Capability` at an ORDINARY scalar call-arg position (derive-based
;; assignability) — but NOT component-wise inside a `Tuple` constructor call (measured,
;; scratchpad/probe-c1-capability-upcast.wat: a bare `(as-cap eh)` call type-checks; a
;; `(Tuple :name eh)` checked against `(keyword,Capability)` does not — tuples aren't
;; covariant on their components today). `process/uses` (wat/spawn.wat) needs to build a
;; `(keyword,Capability)` pair from an arbitrary concrete Handle, so it forces the up-cast
;; HERE, at the scalar boundary (proven to work), before tupling — sidesteps the gap
;; entirely rather than needing a checker change. Identity at runtime; the return type
;; annotation IS the up-cast.
(:wat::core::defn :wat::capability::as-capability [c <- :wat::capability::Capability] -> :wat::capability::Capability c)
