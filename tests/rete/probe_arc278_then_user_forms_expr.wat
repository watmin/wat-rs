;; tests/rete/probe_arc278_then_user_forms_expr.wat — Stone B widening (b) GREEN world.
;; Loaded via startup_from_file. `:then`'s value-position operand is a fenced EXPRESSION
;; (`(:wat::rete::core::i64::+ ?n 1 :undefined 0)`) composed of an admitted rete op — the brief's own
;; headline example (BRIEF-then-user-forms.md's opening code block). The item HEAD stays a plain
;; fact-type constructor (`:tf::Rate`) — this fixture exercises widening (b) ALONE.

(:wat::core::defrecord :tf::In   [n <- :wat::core::i64])
(:wat::core::defrecord :tf::Rate [count <- :wat::core::i64])

(:wat::rete::defrule :tf::compute
  :when [(:tf::In (?n <- :n))]
  :then [(:tf::Rate :count (:wat::rete::core::i64::+ ?n 1 :undefined 0))])

(:wat::rete::defquery :tf::q-Rate
  :params []
  :when [(:tf::Rate (?count <- :count))])


;; Fires via the WAT ORACLE (fire-rules-spec) — mirrors probe_arc278_6b_ii_a_where_oracle.wat's
;; own entry-fn convention. Returns the derived Rate's `count` field: n=5 -> 6, an unconfounded
;; witness (no fact of count=6 could pre-exist; only the derivation can produce it).
(:wat::core::defn :user::run-count-oracle [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :tf)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:tf::q-Rate)))
     session (:wat::rete::insert session (:tf::In :n 5))
     fired   (:wat::rete::fire-rules-spec session)
     derived (:wat::rete::query fired (:tf::q-Rate))
     r       (:wat::core::first derived)]
    (:wat::core::Option/expect
      (:wat::core::PersistentMap/get r "?count")
      "q-Rate: ?count")))

;; The SAME rule, fired through the NATIVE delta kernel (fire-rules -> fire-rules') instead of the
;; oracle — this is compile_rhs's compiled `RhsOp::Expr` path (compiled_rhs.rs), not just the
;; interpreted `build_insert_fact` reference. Same expected value proves compiled == interpreted
;; end-to-end, not only in the compiled_rhs.rs unit differential.
(:wat::core::defn :user::run-count-native [] -> :wat::core::i64
  (:wat::core::let
    [rules   (:wat::rete::collect-rules :tf)
     session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:tf::q-Rate)))
     session (:wat::rete::insert session (:tf::In :n 5))
     fired   (:wat::rete::fire-rules session)
     derived (:wat::rete::query fired (:tf::q-Rate))
     r       (:wat::core::first derived)]
    (:wat::core::Option/expect
      (:wat::core::PersistentMap/get r "?count")
      "q-Rate: ?count")))
