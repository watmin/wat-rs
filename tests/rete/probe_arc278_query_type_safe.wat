;; tests/rete/probe_arc278_query_type_safe.wat — type-safe `query` mouth
;; `(:wat::rete::query session Query [kwargs…])` (`wat/rete/syntax.wat`). Records + BOTH
;; rule-construction shapes — the `defrule`-macro-generated defn path, and a hand-built inline
;; `Rule` literal path — each queried via `query`.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::rete::i64::< ?c 20))
   (:weather::WindSpeed    (?loc <- :location) (?k <- :kph)     (:wat::rete::i64::> ?k 30))]
  :then
  [(:weather::ColdAndWindy :location ?loc)])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact <- :weather::ColdAndWindy)])


;; ── defn-freeze path: the Rule comes from the `defrule`-macro-generated defn ────────────────

(:wat::core::defn :user::query-defrule-path [] -> :wat::core::i64
  (:wat::core::let
    [rules (:wat::core::PersistentVector (:weather::cold-and-windy))
     sess0 (:wat::rete::compile-all rules (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-ColdAndWindy)))))

;; ── inline path: a hand-built Rule literal, no defrule macro ────────────────────────────────

(:wat::core::defn :user::query-inline-path [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:weather::ColdAndWindy ?loc))
     rule  (:wat::rete::Rule :name "weather::cold-and-windy" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile-all (:wat::core::PersistentVector rule) (:wat::core::PersistentVector (:weather::q-ColdAndWindy)))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query fired (:weather::q-ColdAndWindy)))))

