;; :wat::telemetry::uuid::v4 — wat surface for fresh v4 UUID generation.
;;
;; The Rust shim at :rust::telemetry::uuid::v4 mints via wat-edn (arc
;; 092's `new_uuid_v4`) and renders to canonical 8-4-4-4-12
;; hyphenated hex. This file is the wat-side re-export under the
;; curated `:wat::telemetry::*` namespace.
;;
;; Usage:
;;   (let* (((id :String) (:wat::telemetry::uuid::v4)))
;;     ...)
;;
;; The `::` separator places `v4` as a free function under the
;; `:wat::telemetry::uuid::*` sub-namespace — same convention as
;; `:wat::edn::write` or `:wat::core::vec`. The `/` separator is
;; reserved for type-method calls (e.g. `Type/method`).
;;
;; Arc 091 slice 2.

(:wat::core::use! :rust::telemetry::uuid::v4)

(:wat::core::define
  (:wat::telemetry::uuid::v4 -> :String)
  (:rust::telemetry::uuid::v4))
