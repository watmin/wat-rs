;; tests/rete/probe_constructor_meta_kwargs_full_green.wat — BRIEF-construction-total-three-
;; walls.md #2, the "STILL WORKS" counterpart to `probe_constructor_meta_kwargs_undersupply.wat.bad`
;; (the under-supply REJECT proof). `:cr2g::Rate` declares TWO fields; this `:then` item supplies
;; BOTH — the new `RhsMissingFields` freeze-time wall (`validate_and_reorder_then`,
;; `src/rete/validate.rs`) must not reject a legal, fully-supplied kwargs construction. Compiles
;; AND fires, through both the oracle and the native kernel.

(:wat::core::defrecord :cr2g::Anchor [x <- :wat::core::i64])
(:wat::core::defrecord :cr2g::Rate   [count <- :wat::core::i64 window <- :wat::core::i64])

(:wat::rete::defrule :cr2g::gather
  :when [(:cr2g::Anchor (?x <- :x))]
  :then [(:cr2g::Rate :count 7 :window 9)])

(:wat::rete::defquery :cr2g::q-Rate
  :params []
  :when [(:cr2g::Rate (?count <- :count) (?window <- :window))])


;; Fires via the WAT ORACLE.
(:wat::core::defn :user::run-oracle [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cr2g)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cr2g::q-Rate)))
     session (:wat::rete::insert session (:cr2g::Anchor :x 0))
     fired   (:wat::rete::fire-rules$oracle session)
     derived (:wat::rete::query fired (:cr2g::q-Rate))
     r       (:wat::core::first derived)]
    (:wat::i64::+
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get r "?count")
        "q-Rate: ?count")
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get r "?window")
        "q-Rate: ?window"))))

;; Fires via the NATIVE KERNEL — same rule, same expected value, through the compiled RHS path.
(:wat::core::defn :user::run-native [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :cr2g)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cr2g::q-Rate)))
     session (:wat::rete::insert session (:cr2g::Anchor :x 0))
     fired   (:wat::rete::fire-rules session)
     derived (:wat::rete::query fired (:cr2g::q-Rate))
     r       (:wat::core::first derived)]
    (:wat::i64::+
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get r "?count")
        "q-Rate: ?count")
      (:wat::core::Option/expect
        (:wat::core::PersistentMap/get r "?window")
        "q-Rate: ?window"))))
