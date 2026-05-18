;; :wat::telemetry::uuid::v4 — wat surface for fresh v4 UUID generation.
;;
;; Backward-compat alias. Arc 206 promoted UUID minting to the
;; substrate-core path :wat::core::uuid::v4. Arc 207 slice 2 replaced
;; that namespace-form with the typed :wat::core::Uuid/v4 constructor
;; (returns :wat::core::Uuid, not :wat::core::String). Arc 207 slice 3
;; retargets this alias to :wat::core::Uuid/v4 and retires the namespace
;; form entirely. Existing callers receive a typed :wat::core::Uuid now.
;;
;; New code should reach for :wat::core::Uuid/v4 directly.
;;
;; Arc 091 slice 2 minted; arc 206 slice 3 retired the duplicate impl;
;; arc 207 slice 3 retargets to the typed constructor.

(:wat::core::define
  (:wat::telemetry::uuid::v4 -> :wat::core::Uuid)
  (:wat::core::Uuid/v4))
