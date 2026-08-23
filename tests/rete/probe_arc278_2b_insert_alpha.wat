;; tests/rete/probe_arc278_2b_insert_alpha.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :user::Temp record used by the insert/fire tests.

(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])

;; Shared lifecycle: one rule `(:user::Temp (?t <- :value) (> ?t 20))`; stage a matching fact (25) and
;; a non-matching one (15), fire, and inspect alpha-memory (the three probe assertions below).
;;
;; arc 278 "alpha is fire-scoped" (v2): fires via `fire-once'` — native single-pass — not `fire-rules`.
;; `fire-once'` mirrors the oracle's `fire-once`, which genuinely populates alpha (rete.wat:1462), so
;; it stays a truthful home; the fixpoint verb `fire-rules` now clears alpha before freeze (it agrees
;; with the oracle's `fire-rules-spec`, which returns alpha empty via `fire-stratified`). The rule's
;; RHS is empty, so single-pass and fixpoint coincide for these three assertions regardless.

;; (1) exactly one AlphaNode populated (one condition; one of two staged facts matches).
(:wat::core::defn :user::alpha-populated-count [] -> :wat::core::i64
  (:wat::core::let
    [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
     rule  (:wat::rete::Rule :name "r" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))
     sess2 (:wat::rete::insert sess1 (:user::Temp :value 15))
     fired (:wat::rete::fire-once sess2)
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

;; (2) the populated alpha holds ONE Element — 15 was rejected by (> ?t 20).
(:wat::core::defn :user::alpha-matching-element-count [] -> :wat::core::i64
  (:wat::core::let
    [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
     rule  (:wat::rete::Rule :name "r" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))
     sess2 (:wat::rete::insert sess1 (:user::Temp :value 15))
     fired (:wat::rete::fire-once sess2)
     amem  (:wat::rete::Session/alpha-memory fired)
     aid   (:wat::core::Option/expect (:wat::core::get (:wat::core::PersistentMap/keys amem) 0) "aid")
     elems (:wat::core::Option/expect (:wat::core::PersistentMap/get amem aid) "elems")]
    (:wat::core::length elems)))

;; (3) the stored Element's bindings carry ?t = 25 — bindings flow from alpha-match into the Element.
(:wat::core::defn :user::alpha-element-t-binding [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::let
    [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::core::> ?t 20)))
     rule  (:wat::rete::Rule :name "r" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))
     sess0 (:wat::rete::compile (:wat::core::PersistentVector rule))
     sess1 (:wat::rete::insert sess0 (:user::Temp :value 25))
     sess2 (:wat::rete::insert sess1 (:user::Temp :value 15))
     fired (:wat::rete::fire-once sess2)
     amem  (:wat::rete::Session/alpha-memory fired)
     aid   (:wat::core::Option/expect (:wat::core::get (:wat::core::PersistentMap/keys amem) 0) "aid")
     elems (:wat::core::Option/expect (:wat::core::PersistentMap/get amem aid) "elems")
     elem  (:wat::core::Option/expect (:wat::core::get elems 0) "elem")
     binds (:wat::rete::Element/bindings elem)]
    (:wat::core::PersistentMap/get binds "?t")))
