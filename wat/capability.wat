;; wat/capability.wat — the uniform capability surface (arc 170 capability circuit, stone 2).
;;
;; Relocated + renamed from wat/service.wat (was :wat::service::Grantable, loaded at stdlib
;; position ~328). The bracket (wat/bracket.wat) and spawn.wat both need to name this surface,
;; so it must load EARLY — before wat/spawn.wat. It deps only on core.wat builtins
;; (defsurface, Struct, Vector, i64, nil).
;;
;; Grantable — a struct-nature methods-surface every service's `<fqdn>::Handle` satisfies (via
;; the extend-type the defservice macro AUTO-EMITs, routing to the landed <fqdn>/grant &
;; <fqdn>/revoke methods). This lets a HETEROGENEOUS `Vector<:wat::capability::Grantable>` of
;; different services' Handles be grant/revoke'd UNIFORMLY. Both features return nil.
(:wat::core::defsurface :wat::capability::Grantable :nature :wat::core::Struct
  :features
  [(grant  [self <- :wat::capability::Grantable  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)
   (revoke [self <- :wat::capability::Grantable  pids <- (:wat::core::Vector :wat::core::i64)] -> :wat::core::nil)])
