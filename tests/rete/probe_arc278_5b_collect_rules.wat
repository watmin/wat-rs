;; tests/rete/probe_arc278_5b_collect_rules.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Two weather defrules, one non-rule defn, one other-ns defrule.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])
(:wat::rete::defrule :weather::cold-and-windy
  :when [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::rete::i64::< ?c 20))
         (:weather::WindSpeed    (?loc <- :location) (?k <- :kph)     (:wat::rete::i64::> ?k 30))]
  :then [(:weather::ColdAndWindy :location ?loc)])
(:wat::rete::defrule :weather::cold-temp
  :when [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::rete::i64::< ?c 0))]
  :then [(:weather::ColdAndWindy :location ?loc)])
(:wat::core::defn :weather::helper [] -> :wat::core::i64 42)
(:wat::rete::defrule :other::windy
  :when [(:weather::WindSpeed (?loc <- :location) (?k <- :kph))]
  :then [(:weather::ColdAndWindy :location ?loc)])

;; :weather has 2 defrules (+ a non-rule defn `helper` that must NOT be counted).
(:wat::core::defn :user::weather-rule-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::collect-rules :weather)))

;; :other has exactly one rule; :weather's rules are NOT collected under :other.
(:wat::core::defn :user::other-rule-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::collect-rules :other)))

;; no rules in :nonexistent → empty PV.
(:wat::core::defn :user::nonexistent-rule-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::collect-rules :nonexistent)))

;; Sorted-by-name order: "cold-and-windy" < "cold-temp".
(:wat::core::defn :user::first-collected-rule-name [] -> :wat::core::String
  (:wat::core::let [rs (:wat::rete::collect-rules :weather)]
    (:wat::rete::Rule/name (:wat::core::Option/expect (:wat::core::get rs 0) "r0"))))

(:wat::core::defn :user::second-collected-rule-name [] -> :wat::core::String
  (:wat::core::let [rs (:wat::rete::collect-rules :weather)]
    (:wat::rete::Rule/name (:wat::core::Option/expect (:wat::core::get rs 1) "r1"))))
