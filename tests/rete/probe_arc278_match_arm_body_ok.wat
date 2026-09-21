;; strike-match-arm-is-not-a-call (D5) — THE ARM-BODY CONTROL, and it is load-bearing.
;;
;; The cure skips an arm's PATTERN. The cheapest wrong cure is to skip the whole `match` form, which
;; makes the three spelling probes green and silently retires four error kinds
;; (`UnknownField`, `RhsMissingFields`, `RhsArityMismatch`, `RhsPositionalConstructionRetired`)
;; inside every match arm BODY — the exact shape strike-nested-wall found and fixed one strike ago.
;;
;; This file and its `.wat.bad` twin are the pair that separates the two cures: here a CORRECT
;; nested constructor sits in an arm body and must still compile and FIRE with the right values;
;; there a misspelled one must still be REFUSED. A cure that stops walking match forms passes this
;; file and fails its twin.

(:wat::core::defenum :macb::E :wat::enum::Pure :A :B)

(:wat::core::defrecord :macb::Inner [n <- :wat::core::i64])
(:wat::core::defrecord :macb::In    [k <- :wat::core::i64  v <- :macb::E])
(:wat::core::defrecord :macb::Out   [k <- :wat::core::i64  inner <- :macb::Inner])

(:wat::rete::defrule :macb::r
  :when [(:macb::In (?k :- :k) (?v :- :v))]
  :then [(:macb::Out :k ?k
           :inner (:wat::rete::core::match ?v
                    [:macb::E.A {} (:macb::Inner :n 10)]
                    [:macb::E.B {} (:macb::Inner :n 20)]))])

(:wat::rete::defquery :macb::by-inner
  :params [?inner]
  :when [(:macb::Out (?inner :- :inner) (?k :- :k))])

(:wat::core::defn :macb::world [] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-rules
    (:wat::core::match (:wat::rete::insert
      (:wat::core::match (:wat::rete::compile-all
                           (:wat::core::PersistentVector (:macb::r))
                           (:wat::core::PersistentVector (:macb::by-inner)))
        [:wat::rete::CompileOutcome.Compiled {:session __s} __s]
        [:wat::rete::CompileOutcome.MayNotTerminate {:rule __r :fact-type __f}
          (:wat::kernel::assertion-failed! :message "compile: may not terminate")])
      (:macb::In :k 1 :v :macb::E.A)
      (:macb::In :k 2 :v :macb::E.B)
      (:macb::In :k 3 :v :macb::E.A))
      [:wat::rete::InsertOutcome.Inserted {:session __st} __st]
      [:wat::rete::InsertOutcome.MemoryCeilingExceeded {:limit __a :used __b :staged __c}
        (:wat::kernel::assertion-failed! :message "insert: ceiling")]))
    [:wat::rete::FireOutcome.Fired {:value __f} __f]
    [:wat::rete::FireOutcome.MemoryCeilingExceeded {:limit __l :used __u :rounds __r2}
      (:wat::kernel::assertion-failed! :message "fire: ceiling")]
    [:wat::rete::FireOutcome.RoundCapExceeded {:cap __c :still-deriving __s}
      (:wat::kernel::assertion-failed! :message "fire: round cap")]))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [w (:macb::world)]
    (:wat::kernel::println
      (:wat::string::concat
        (:wat::string::concat "n10=" (:wat::i64::to-string
          (:wat::core::length (:wat::rete::query w (:macb::by-inner) :?inner (:macb::Inner :n 10)))))
        (:wat::string::concat " n20=" (:wat::i64::to-string
          (:wat::core::length (:wat::rete::query w (:macb::by-inner) :?inner (:macb::Inner :n 20)))))))))
