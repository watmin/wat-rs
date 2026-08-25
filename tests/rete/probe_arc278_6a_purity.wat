;; tests/rete/probe_arc278_6a_purity.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines test functions for purity/determinism classification.

(:wat::core::defn :test::pure-double [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::* n 2))

(:wat::core::defn :test::nondet-uuid [] -> :wat::core::Uuid
  (:wat::core::Uuid/v4))

(:wat::core::defn :test::io-fn [] -> :wat::io::IOReader
  (:wat::io::IOReader/open-file "x"))

(:wat::core::defn :test::countdown [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::<= n 0)
    0
    (:test::countdown (:wat::core::- n 1))))

;; just-eval wrappers — `(:wat::rete::<pred> (:wat::core::quote <expr>))` -> bool, one per assertion.

;; ─── THE orthogonality proof: Uuid/v4 is pure ∧ non-deterministic ──────────────
(:wat::core::defn :user::uuid-v4-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::core::Uuid/v4))))
(:wat::core::defn :user::uuid-v4-deterministic? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:wat::core::Uuid/v4))))
(:wat::core::defn :user::uuid-v5-deterministic? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:wat::core::Uuid/v5 (:wat::core::Uuid/nil) "x"))))

;; ─── pure? axis (effect-free) ───────────────────────────────────────────────────
(:wat::core::defn :user::pure-arithmetic-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::core::> (:wat::core::- 5 3) 1))))
(:wat::core::defn :user::pure-string-predicate-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::string::starts-with? "abc" "a"))))
(:wat::core::defn :user::io-op-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::io::IOReader/open-file "x"))))
(:wat::core::defn :user::transitively-effectful-user-fn-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:test::io-fn))))
(:wat::core::defn :user::pure-user-fn-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:test::pure-double 5))))
(:wat::core::defn :user::unknown-head-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:not::a::real::op 1))))
(:wat::core::defn :user::self-recursive-pure-fn-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:test::countdown 3))))
(:wat::core::defn :user::pure-cond-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::core::cond ((:wat::core::> 5 3) 1) (true 0)))))
(:wat::core::defn :user::cond-with-io-body-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::core::cond ((:wat::core::> 5 3) (:wat::io::IOReader/open-file "x")) (true 0)))))
(:wat::core::defn :user::pure-match-with-ctor-pattern-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::core::match ?x  ((:wat::core::Some v) v) (:wat::core::None 0)))))
(:wat::core::defn :user::match-with-io-body-pure? [] -> :wat::core::bool
  (:wat::rete::pure? (:wat::core::quote (:wat::core::match ?x  ((:wat::core::Some v) (:wat::io::IOReader/open-file "x")) (:wat::core::None nil)))))

;; ─── deterministic? axis (referential transparency) ─────────────────────────────
(:wat::core::defn :user::pure-arithmetic-deterministic? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:wat::core::> (:wat::core::- 5 3) 1))))
(:wat::core::defn :user::transitively-nondeterministic-user-fn-deterministic? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:test::nondet-uuid))))
(:wat::core::defn :user::io-op-deterministic? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:wat::io::IOReader/open-file "x"))))
(:wat::core::defn :user::match-on-nondeterministic-scrutinee-deterministic? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:wat::core::match (:wat::core::Uuid/v4)  (:wat::core::None nil)))))
(:wat::core::defn :user::self-recursive-fn-deterministic? [] -> :wat::core::bool
  (:wat::rete::deterministic? (:wat::core::quote (:test::countdown 3))))

