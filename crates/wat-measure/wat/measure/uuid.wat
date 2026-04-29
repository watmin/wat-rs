;; :wat::measure::uuid::v4 — wat surface for fresh v4 UUID generation.
;;
;; The Rust shim at :rust::measure::uuid::v4 mints via wat-edn (arc
;; 092's `new_uuid_v4`) and renders to canonical 8-4-4-4-12
;; hyphenated hex. This file is the wat-side re-export under the
;; curated `:wat::measure::*` namespace.
;;
;; Usage:
;;   (let* (((id :String) (:wat::measure::uuid::v4)))
;;     ...)
;;
;; The `::` separator places `v4` as a free function under the
;; `:wat::measure::uuid::*` sub-namespace — same convention as
;; `:wat::edn::write` or `:wat::core::vec`. The `/` separator is
;; reserved for type-method calls (e.g. `Type/method`).
;;
;; Arc 091 slice 2.

(:wat::core::use! :rust::measure::uuid::v4)

(:wat::core::define
  (:wat::measure::uuid::v4 -> :String)
  (:rust::measure::uuid::v4))
