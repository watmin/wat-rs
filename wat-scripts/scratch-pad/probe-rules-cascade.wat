;; Arena crux: does rete CASCADE within one seed's fire (a rule firing on a DERIVED Lemma)?
;; Record → Lemma → Deduction, single lineage, one insert, fire-to-fixpoint.
;;   hot:      Temp c>50  → Hot       (Hot is a LEMMA — fired-upon below)
;;   alert:    Hot        → Alert     (fires on the DERIVED Hot — the cascade; Alert = Deduction)
;;   critical: Temp c>90  → Critical  (graded, parallel; Critical = Deduction)
;; Lemma  = derived ∩ fired-upon = {Hot}
;; Deduction = derived − fired-upon = {Alert, Critical}
;; Expect: Temp=60 → Hot=1, Alert=1 (cascade!), Critical=0  → 1 Deduction
;;         Temp=95 → Hot=1, Alert=1, Critical=1             → 2 Deductions

(:wat::core::defrecord :usr::Temp     [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot      [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Alert    [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Critical [c <- :wat::core::i64])

(:wat::rete::defrule :usr::hot
  :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
  :then [(:usr::Hot :c ?c)])
(:wat::rete::defrule :usr::alert
  :when [(:usr::Hot (?c <- :c))]
  :then [(:usr::Alert :c ?c)])
(:wat::rete::defrule :usr::critical
  :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 90))]
  :then [(:usr::Critical :c ?c)])

(:wat::core::defn :usr::fire-one [template <- :wat::rete::Session seed <- :usr::Temp] -> :wat::core::String
  (:wat::core::let
    [fired (:wat::rete::fire-rules (:wat::rete::insert template seed))
     h (:wat::core::length (:wat::rete::query fired :usr::Hot))
     a (:wat::core::length (:wat::rete::query fired :usr::Alert))
     cr (:wat::core::length (:wat::rete::query fired :usr::Critical))]
    (:wat::core::string::concat "Hot=" (:wat::core::string::concat (:wat::core::str h)
      (:wat::core::string::concat " Alert=" (:wat::core::string::concat (:wat::core::str a)
        (:wat::core::string::concat " Critical=" (:wat::core::str cr))))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules    (:wat::core::PersistentVector (:usr::hot) (:usr::alert) (:usr::critical))
     template (:wat::rete::compile rules)]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "Temp=60: " (:usr::fire-one template (:usr::Temp :c 60))))
      (:wat::kernel::println (:wat::core::string::concat "Temp=95: " (:usr::fire-one template (:usr::Temp :c 95)))))))
