;; tests/rete/probe_arc278_7a_negation_oracle.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the alert::unattended rule for negation oracle tests.

(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :ops::Maintenance     [location <- :wat::core::String])
(:wat::core::defrecord :alert::Unattended    [location <- :wat::core::String])

(:wat::rete::defrule :alert::unattended
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius))
   (:wat::rete::not (:ops::Maintenance (?loc <- :location)))]
  :then
  (:wat::rete::insert (:alert::Unattended :location ?loc)))

