;; tests/rete/probe_constructor_meta_enum_variant_green.wat — BRIEF-construction-total-three-
;; walls.md #3, the "STILL WORKS" counterpart to `probe_constructor_meta_surface_total_enum.wat.bad`
;; (the wrong-arity REJECT proof). A CORRECT-arity nested tagged-enum-variant constructor
;; (`(:cg::Status::Active 7)` — `Active` declares exactly one field) must compile AND fire,
;; through both the oracle and the native kernel — the new freeze-time arity wall
;; (`walk_nested_constructors`, `src/rete/validate.rs`) must not reject a legal call.

(:wat::core::defenum :cg::Status :wat::enum::Pure
  :Active [level <- :wat::core::i64])

(:wat::core::defrecord :cg::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :cg::Wrap   [s <- :cg::Status])

(:wat::rete::defrule :cg::gather
  :when [(:cg::Anchor (?x <- :x))]
  :then [(:cg::Wrap :s (:cg::Status::Active 7))])

(:wat::rete::defquery :cg::q-Wrap
  :params []
  :when [(:cg::Wrap (?s <- :s))])


;; Fires via the WAT ORACLE.
(:wat::core::defn :user::run-oracle [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Wrap)))
     session (:wat::rete::insert session (:cg::Anchor :x 0))
     fired   (:wat::rete::fire-rules$oracle session)
     derived (:wat::rete::query fired (:cg::q-Wrap))
     r       (:wat::core::first derived)
     s       (:wat::core::Option/expect
               (:wat::map::get r "?s")
               "q-Wrap: ?s")]
    (:wat::core::match s
      ((:cg::Status::Active lvl) lvl))))

;; Fires via the NATIVE KERNEL — same rule, same expected value, through the compiled RHS path.
(:wat::core::defn :user::run-native [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cg)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cg::q-Wrap)))
     session (:wat::rete::insert session (:cg::Anchor :x 0))
     fired   (:wat::rete::fire-rules session)
     derived (:wat::rete::query fired (:cg::q-Wrap))
     r       (:wat::core::first derived)
     s       (:wat::core::Option/expect
               (:wat::map::get r "?s")
               "q-Wrap: ?s")]
    (:wat::core::match s
      ((:cg::Status::Active lvl) lvl))))
