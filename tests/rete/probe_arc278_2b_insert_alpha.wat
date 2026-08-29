;; tests/rete/probe_arc278_2b_insert_alpha.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :user::Temp record used by the insert/fire tests.

(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])

;; Shared lifecycle: one rule `(:user::Temp (?t <- :value) (> ?t 20))`; stage a matching fact (25) and
;; a non-matching one (15), fire-once, and inspect alpha-memory (the three probe assertions below).
;;
;; arc 278 "alpha is fire-scoped" (v2): fires via `fire-once` — native single-pass — not `fire-rules`.
;; `fire-once` mirrors the oracle's `fire-once$oracle`, which genuinely populates alpha (wat/rete/oracle/fire.wat:167), so
;; it stays a truthful home; the fixpoint verb `fire-rules` now clears alpha before freeze (it agrees
;; with the oracle's `fire-rules$oracle`, which returns alpha empty via `fire-stratified`). The rule's
;; RHS is empty, so single-pass and fixpoint coincide for these three assertions regardless.

(:wat::core::defn :test::compile-temp-rule [] -> :wat::rete::Session
  (:wat::core::let
    [cond  (:wat::core::quote (:user::Temp (?t <- :value) (:wat::rete::core::i64::> ?t 20)))
     rule  (:wat::rete::Rule :name "r" :lhs (:wat::core::PersistentVector cond) :rhs (:wat::core::PersistentVector))]
    (:wat::rete::compile (:wat::core::PersistentVector rule))))

(:wat::core::defn :test::seed-temps [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::insert s (:user::Temp :value 25))
    (:user::Temp :value 15)))

(:wat::core::defn :test::fire-once [s <- :wat::rete::Session] -> :wat::rete::Session
  (:wat::core::match (:wat::rete::fire-once s) ((:wat::rete::FireOutcome::Fired __fired) __fired) ((:wat::rete::FireOutcome::MemoryCeilingExceeded __limit __used __rounds) (:wat::kernel::assertion-failed! "fire-once: session memory ceiling exceeded" :wat::core::None :wat::core::None)) ((:wat::rete::FireOutcome::RoundCapExceeded __cap __still) (:wat::kernel::assertion-failed! "fire-once: fixpoint round cap exceeded" :wat::core::None :wat::core::None))))

(:wat::core::defn :test::fired-temp-alpha [] -> :wat::rete::Session
  (:test::fire-once (:test::seed-temps (:test::compile-temp-rule))))

(:wat::core::defn :user::compile-then-fire-empty-alpha [] -> :wat::core::i64
  (:wat::core::let
    [fired (:test::fire-once (:test::compile-temp-rule))
     ;; rune:vocare(vantage-bypass-test) — empty :rhs so the caller mouth cannot see the match; implementer alpha layout
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

(:wat::core::defn :user::seed-temps-fact-count [] -> :wat::core::i64
  (:wat::core::length (:wat::rete::Session/facts (:test::seed-temps (:test::compile-temp-rule)))))

;; (1) exactly one AlphaNode populated (one condition; one of two staged facts matches).
(:wat::core::defn :user::alpha-populated-count [] -> :wat::core::i64
  (:wat::core::let
    [fired (:test::fired-temp-alpha)
     ;; rune:vocare(vantage-bypass-test) — empty :rhs so the caller mouth cannot see the match; implementer alpha layout
     amem  (:wat::rete::Session/alpha-memory fired)]
    (:wat::core::length (:wat::core::PersistentMap/keys amem))))

;; (2) the populated alpha holds ONE Element — 15 was rejected by (> ?t 20).
(:wat::core::defn :user::alpha-matching-element-count [] -> :wat::core::i64
  (:wat::core::let
    [fired (:test::fired-temp-alpha)
     ;; rune:vocare(vantage-bypass-test) — empty :rhs so the caller mouth cannot see the match; implementer alpha layout
     amem  (:wat::rete::Session/alpha-memory fired)
     aid   (:wat::core::Option/expect (:wat::core::get (:wat::core::PersistentMap/keys amem) 0) "aid")
     elems (:wat::core::Option/expect (:wat::core::PersistentMap/get amem aid) "elems")]
    (:wat::core::length elems)))

;; (3) the stored Element's bindings carry ?t = 25 — bindings flow from alpha-match into the Element.
(:wat::core::defn :user::alpha-element-t-binding [] -> (:wat::core::Option :- [:wat::core::i64])
  (:wat::core::let
    [fired (:test::fired-temp-alpha)
     ;; rune:vocare(vantage-bypass-test) — empty :rhs so the caller mouth cannot see the match; implementer alpha layout
     amem  (:wat::rete::Session/alpha-memory fired)
     aid   (:wat::core::Option/expect (:wat::core::get (:wat::core::PersistentMap/keys amem) 0) "aid")
     elems (:wat::core::Option/expect (:wat::core::PersistentMap/get amem aid) "elems")
     elem  (:wat::core::Option/expect (:wat::core::get elems 0) "elem")
     binds (:wat::rete::Element/bindings elem)]
    (:wat::core::PersistentMap/get binds "?t")))
