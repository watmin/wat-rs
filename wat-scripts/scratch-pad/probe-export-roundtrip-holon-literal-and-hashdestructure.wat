;; Does a rete program carrying (a) a `#holon` literal and (b) a hash-destructure arm survive the
;; `#wat.rete/Export` wire? Both are 2026-08-28/29 additions and both put NEW shapes into the
;; compiled program: `Expr::Lit(Value::holon__HolonAST)` and `Pat::Fields`.
(:wat::core::defrecord :xr::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :xr::In  [k <- :wat::core::String  p <- :xr::Point  h <- :wat::holon::HolonAST])
(:wat::core::defrecord :xr::Out [k <- :wat::core::String])

(:wat::rete::defrule :xr::rule
  :when
  [(:xr::In (?k <- :k) (?p <- :p) (?h <- :h))
   (:wat::rete::where
     (:wat::rete::core::and
       (:wat::rete::core::i64::=
         (:wat::rete::core::match ?p ({vx :x  vy :y} (:wat::rete::core::i64::+ vx vy :undefined 0)))
         42)
       (:wat::rete::holon::coincident? ?h #holon [1 2 3])))]
  :then
  [(:xr::Out :k ?k)])

(:wat::rete::defquery :xr::q :params [] :when [(?fact <- :xr::Out)])

(:wat::core::defn :xr::seed [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::insert
    (:wat::core::match (:wat::rete::insert s (:xr::In :k "hit"  :p (:xr::Point :x 40 :y 2) :h #holon [1 2 3])) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None)))
    (:xr::In :k "miss" :p (:xr::Point :x 1 :y 1) :h #holon [7 8 9])) ((:wat::rete::InsertOutcome::Inserted __staged) __staged) ((:wat::rete::InsertOutcome::MemoryCeilingExceeded __limit __used __count) (:wat::kernel::assertion-failed! "insert: session memory ceiling exceeded while staging" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules (:wat::rete::collect-rules :xr)
     qs    (:wat::core::PersistentVector (:xr::q))]
    (:wat::core::do
      (:wat::kernel::println "DIRECT (no wire):")
      (:wat::kernel::println
        (:wat::core::length
          (:wat::rete::query
            (:wat::core::match (:wat::rete::fire-rules (:xr::seed (:wat::rete::compile-all rules qs))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None))) (:xr::q))))
      ;; ⚠ NOT DEMONSTRATED HERE BECAUSE IT PANICS: `(:wat::edn::write <this Export>)`.
      ;; An Export carrying a `#holon` literal cannot be written as EDN text — the literal is an
      ;; UNCLASSIFIED bundle, and `edn::write` refuses (panics) on unclassified holon algebra.
      ;; That is the DEFERRED `watast_to_holon` defect, not an export defect: the same data lifted
      ;; via `to-holon` writes fine, as `#wat/holon [1 2 3]`. See
      ;; ~/work/NOTE-holon-classifier-contract-is-unenforced-and-the-holon-tag-breaks-it.md
      (:wat::kernel::println "ROUND-TRIPPED through export/import:")
      (:wat::kernel::println
        (:wat::core::length
          (:wat::rete::query
            (:wat::core::match (:wat::rete::fire-rules
              (:xr::seed (:wat::rete::import (:wat::rete::export (:wat::rete::compile-all rules qs))))) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-rules: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-rules: fixpoint round cap exceeded" :wat::core::None :wat::core::None)))
            (:xr::q)))))))
