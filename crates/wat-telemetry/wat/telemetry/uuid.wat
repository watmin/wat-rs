;; :wat::telemetry::uuid::v4 — wat surface for fresh v4 UUID generation.
;;
;; Backward-compat alias. Arc 206 promoted UUID minting to the
;; substrate-core path :wat::core::uuid::v4 (reachable without any
;; telemetry dep). This file remains so existing :wat::telemetry::uuid::v4
;; callers keep compiling; it now delegates straight to the substrate
;; primitive instead of duplicating the impl through a :rust::telemetry
;; shim. New code should reach for :wat::core::uuid::v4 directly.
;;
;; Usage (unchanged):
;;   (let (((id :wat::core::String) (:wat::telemetry::uuid::v4)))
;;     ...)
;;
;; Arc 091 slice 2 minted; arc 206 slice 3 retired the duplicate impl.

(:wat::core::define
  (:wat::telemetry::uuid::v4 -> :wat::core::String)
  (:wat::core::uuid::v4))
