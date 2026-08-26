;; 293.R2.3 — construction-form parity: EVERY type-name is its own constructor (bare :T), /new DROPPED.
;;
;; RED at HEAD: structs + newtypes construct only via :T/new — the bare ctor `(:b::Pt 3 4)` and
;; `(:b::Price 38)` are UNRESOLVED ("not a registered function"). Records already construct bare
;; (the defrecord macro emits a bare-name defn ctor); structs/newtypes are the holdouts.
;;
;; GREEN after 293.R2.3: register_struct_methods + register_newtype_methods mint the constructor at
;; the BARE type name (parity with records), /new annihilated. `(:b::probe)` => 41 (3 + 38).

(:wat::core::defstruct :b::Pt    [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::newtype   :b::Price :wat::core::i64)

(:wat::core::defn :b::probe [] -> :wat::core::i64
  (:wat::i64::+
    (:b::Pt/x    (:b::Pt :x 3 :y 4))      ;; bare struct ctor
    (:b::Price/0 (:b::Price 38))))  ;; bare newtype ctor
