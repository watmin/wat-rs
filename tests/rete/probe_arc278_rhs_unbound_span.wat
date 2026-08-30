;; Fixture BESIDE probe_arc278_rhs_unbound_span.rs.
;;
;; `:then` names ?missing, which `:when` never binds. This is SHAPE-valid — the
;; validators check the insert head, fact type, field names and positional arity,
;; and walk nested constructors, but do NOT bind-check `?var` — so `--check` exits 0
;; and the failure surfaces at FIRE time, on the native compiled-RHS path.
;;
;; The operand `?missing` sits at line 16, cols 24..32. Those numbers are asserted:
;; if you edit above this line, fix the .rs.

(:wat::core::defrecord :ubs::Item [k <- :wat::core::i64])
(:wat::core::defrecord :ubs::Out  [k <- :wat::core::i64])

(:wat::rete::defrule :ubs::r
  :when [(:ubs::Item (?k <- :k))]
  :then [(:ubs::Out :k ?missing)])

(:wat::rete::defquery :ubs::q :params [] :when [(?fact <- :ubs::Out)])

(:wat::core::defn :user::fire-unbound [] -> :wat::core::i64
  (:wat::core::let [rules (:wat::rete::collect-rules :ubs)
                    s0    (:wat::rete::compile-all rules (:wat::core::PersistentVector (:ubs::q)))
                    s1    (:wat::rete::insert-all s0 (:wat::core::PersistentVector (:ubs::Item :k 1)))
                    fired (:wat::core::match (:wat::rete::fire-rules s1) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))]
    (:wat::core::length (:wat::rete::query fired (:ubs::q)))))
