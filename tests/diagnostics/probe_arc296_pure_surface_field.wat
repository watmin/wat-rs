;; tests/diagnostics/probe_arc296_pure_surface_field.wat — co-located fixture
;;
;; Arc 296 — a Record-natured surface used as a RECURSIVE field type in a pure aggregate.
;;
;; RED at HEAD: is_pure_type returns false for any Surface arm → ImpureFieldInPureAggregate
;; when :probe::Boom declares `causes <- (:wat::core::Vector :- [probe::E])`.
;;
;; GREEN after arc 296 fix: Surface purity mirrors its nature's purity.
;; :probe::E has :nature :wat::core::Record → is_pure → the (Vector :- [E]) field is allowed.

(:wat::core::defsurface :probe::E
  :nature :wat::core::Record
  :features [message <- :wat::core::String
             causes  <- (:wat::core::Vector :- [:probe::E])])

(:wat::core::defrecord :probe::Boom
  [message <- :wat::core::String
   causes  <- (:wat::core::Vector :- [:probe::E])])
