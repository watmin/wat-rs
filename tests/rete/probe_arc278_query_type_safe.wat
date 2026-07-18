;; tests/rete/probe_arc278_query_type_safe.wat — RED-gate fixture for arc 278 query (a): the
;; type-safe front door `(:wat::rete::query fired :Type)` (wat/rete.wat's `query`, restored as a
;; `defmacro` over the PRIME type-ref). Records + BOTH rule-construction shapes this rete surface
;; supports — the `defrule`-macro-generated defn path, and a hand-built inline `Rule` literal path
;; (mirrors probe_arc278_5a_defrule_query_with_rule.wat / probe_arc278_5a_defrule_query_plain.wat) —
;; each queried BOTH via `query` (the type-safe macro under test) and via `query-by-type-string`
;; (the untyped escape hatch) on the SAME fired session, so the .rs harness can assert the counts
;; agree exactly.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature (?loc <- :location) (?c <- :celsius) (:wat::core::< ?c 20))
   (:weather::WindSpeed    (?loc <- :location) (?k <- :kph)     (:wat::core::> ?k 30))]
  :then
  (:wat::rete::insert (:weather::ColdAndWindy :location ?loc)))

;; ── defn-freeze path: the Rule comes from the `defrule`-macro-generated defn ────────────────

(:wat::core::defn :user::query-defrule-path [] -> :wat::core::i64
  (:wat::core::let
    [rules (:wat::core::PersistentVector (:weather::cold-and-windy))
     sess0 (:wat::rete::compile rules)
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy))))

(:wat::core::defn :user::query-by-type-string-defrule-path [] -> :wat::core::i64
  (:wat::core::let
    [rules (:wat::core::PersistentVector (:weather::cold-and-windy))
     sess0 (:wat::rete::compile rules)
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query-by-type-string fired "weather::ColdAndWindy"))))

;; ── inline path: a hand-built Rule literal, no defrule macro ────────────────────────────────

(:wat::core::defn :user::query-inline-path [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))
     rule  (:wat::rete::Rule :name "weather::cold-and-windy" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query fired :weather::ColdAndWindy))))

(:wat::core::defn :user::query-by-type-string-inline-path [] -> :wat::core::i64
  (:wat::core::let
    [c1    (:wat::core::quote (:weather::Temperature (?loc <- :location) (?t <- :celsius)))
     c2    (:wat::core::quote (:weather::WindSpeed (?loc <- :location) (?w <- :kph)))
     rhs1  (:wat::core::quote (:wat::rete::insert (:weather::ColdAndWindy ?loc)))
     rule  (:wat::rete::Rule :name "weather::cold-and-windy" :lhs (:wat::core::PersistentVector c1 c2) :rhs (:wat::core::PersistentVector rhs1))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     s1    (:wat::rete::insert sess0 (:weather::Temperature :celsius 15 :location "Oslo"))
     s2    (:wat::rete::insert s1 (:weather::WindSpeed :kph 45 :location "Oslo"))
     fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query-by-type-string fired "weather::ColdAndWindy"))))
