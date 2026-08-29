;; The per-session memory ceiling, driven at its floor. A legitimate 500-round transitive closure
;; — the same workload `..._round_cap_deep.wat` proves the ROUND cap tolerates — is refused here
;; only because the ceiling is set to one page.
;;
;; ⛔ THE POINT IS THE AXIS THE ROUND CAP CANNOT SEE. A fanout divergence multiplies WITHIN a round,
;; so it reaches the allocator while `rounds_run` is still in single digits: measured 2026-08-29 as
;; an allocator abort at 6.2s, no wat error and no rule named. This ceiling is checked BEFORE the
;; round cap for exactly that reason, and the `:rounds 1` in the error is the tell — a low round
;; count says the growth was fanout, not depth.
(:wat::config::rete::set-max-session-bytes! 4096)

(:wat::core::defrecord :sm::N    [k <- :wat::core::i64])
(:wat::core::defrecord :sm::Edge [a <- :wat::core::i64  b <- :wat::core::i64])

(:wat::rete::defrule :sm::step
  :when
  [(:sm::N (?k <- :k))
   (:sm::Edge (?a <- :a) (?b <- :b))
   (:wat::rete::where (:wat::rete::core::i64::= ?k ?a))]
  :then [(:sm::N :k ?b)])

(:wat::rete::defquery :sm::q :params [] :when [(?fact <- :sm::N)])

(:wat::core::defn :sm::seed [s <- :wat::rete::Session  n <- :wat::core::i64] -> :wat::rete::Session
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::rete::Session  i <- :wat::core::i64] -> :wat::rete::Session
      (:wat::rete::insert acc (:sm::Edge :a i :b (:wat::core::i64::+ i 1))))
    s (:wat::core::range 0 n)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::core::length
    (:wat::core::let
      [rules (:wat::rete::collect-rules :sm)
       s (:wat::rete::compile-all rules (:wat::core::PersistentVector (:sm::q)))
       s (:sm::seed s 500)
       s (:wat::rete::insert s (:sm::N :k 0))
       f (:wat::rete::fire-rules s)]
      (:wat::rete::query f (:sm::q))))))
