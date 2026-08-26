;; tests/rete/probe_arc278_then_user_forms_expr.wat — Stone B widening (b) GREEN world.
;; Loaded via startup_from_file. `:then`'s value-position operand is a fenced EXPRESSION
;; (`(:wat::rete::core::i64::+ ?n 1 :undefined 0)`) composed of an admitted rete op — the brief's own
;; headline example (BRIEF-then-user-forms.md's opening code block). The item HEAD stays a plain
;; fact-type constructor (`:tf::Rate`) — this fixture exercises widening (b) ALONE.

(:wat::core::defrecord :tf::In   [n <- :wat::core::i64])
(:wat::core::defrecord :tf::Rate [count <- :wat::core::i64])

(:wat::rete::defrule :tf::compute
  :when [(:tf::In (?n <- :n))]
  :then [(:tf::Rate :count (:wat::rete::i64::+ ?n 1 :undefined 0))])

(:wat::rete::defquery :tf::q-Rate
  :params []
  :when [(:tf::Rate (?count <- :count))])


(:wat::core::defn :test::compile-tf [] -> :wat::rete::Session
  (:wat::rete::compile-all
    (:wat::rete::collect-rules :tf)
    (:wat::core::PersistentVector (:tf::q-Rate))))

(:wat::core::defn :test::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert s (:tf::In :n 5)))

(:wat::core::defn :test::count-rate [s <- :wat::rete::Session] -> :wat::core::i64
  (:wat::core::Option/expect
    (:wat::core::PersistentMap/get
      (:wat::core::first (:wat::rete::query s (:tf::q-Rate)))
      "?count")
    "q-Rate: ?count"))

(:wat::core::defn :test::run
  [fire <- :wat::core::Fn(wat::rete::Session)->wat::rete::Session]
  -> :wat::core::i64
  (:test::count-rate (fire (:test::seed (:test::compile-tf)))))

;; Fires via the WAT ORACLE (fire-rules$oracle) — mirrors probe_arc278_6b_ii_a_where_oracle.wat's
;; own entry-fn convention. Returns the derived Rate's `count` field: n=5 -> 6, an unconfounded
;; witness (no fact of count=6 could pre-exist; only the derivation can produce it).
(:wat::core::defn :user::run-count-oracle [] -> :wat::core::i64
  (:test::run :wat::rete::fire-rules$oracle))

;; The SAME rule, fired through the NATIVE delta kernel (fire-rules) instead of the
;; oracle — this is compile_rhs's compiled `RhsOp::Expr` path (compiled_rhs.rs), not just the
;; interpreted `build_insert_fact` reference. Same expected value proves compiled == interpreted
;; end-to-end, not only in the compiled_rhs.rs unit differential.
(:wat::core::defn :user::run-count-native [] -> :wat::core::i64
  (:test::run :wat::rete::fire-rules))
