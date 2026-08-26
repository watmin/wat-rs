;; tests/function/probe_arc237_stone2_defclause_substrate.wat
;; Arc 237 Stone 237.2 — defclause substrate mint (12 probes).
;; Co-located fixture, slurped via startup_beside(file!()).
;; Negative (startup-fail) cases are in sibling *.wat.bad files.

;; Probe 1 — single-clause defclause parses + type-checks (basic foundation)
(:wat::core::defclause :p01::identity
  ([x <- :wat::core::i64] -> :wat::core::i64 x))

;; Probe 2 — multi-arity dispatches by arity at call site (3-arg → 60)
(:wat::core::defclause :p02::add
  ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
    (:wat::i64::+ x y))
  ([x <- :wat::core::i64 y <- :wat::core::i64 z <- :wat::core::i64] -> :wat::core::i64
    (:wat::i64::+ (:wat::i64::+ x y) z)))
(:wat::core::defn :user::probe-02 [] -> :wat::core::i64 (:p02::add 10 20 30))

;; Probe 3 — same-arity multi-clause dispatches by arg type (i64 clause → 10)
(:wat::core::defclause :p03::sum
  ([x <- :wat::core::i64 y <- :wat::core::i64] -> :wat::core::i64
    (:wat::i64::+ x y))
  ([x <- :wat::core::f64 y <- :wat::core::f64] -> :wat::core::f64
    (:wat::f64::+ x y)))
(:wat::core::defn :user::probe-03 [] -> :wat::core::i64 (:p03::sum 7 3))

;; Probe 4 — typeunion-typed arg accepts via bounded existential (Stone 237.1 integration)
(:wat::core::typeunion :p04::Numeric [:wat::core::i64 :wat::core::f64])
(:wat::core::defclause :p04::identity-num
  ([x <- :p04::Numeric] -> :p04::Numeric x))
(:wat::core::defn :user::probe-04 [] -> :wat::core::nil
  (:wat::core::do (:p04::identity-num 42) (:p04::identity-num 3.14) nil))

;; Probe 5 — shared return type applies to all clauses (Option A) → 12
(:wat::core::defclause :p05::pick -> :wat::core::i64
  ([x <- :wat::core::i64] x)
  ([x <- :wat::core::i64 y <- :wat::core::i64] (:wat::i64::+ x y)))
(:wat::core::defn :user::probe-05 [] -> :wat::core::i64 (:p05::pick 5 7))

;; Probe 6 — per-clause return types; caller picks via clause match → i64(42)
(:wat::core::defclause :p06::process
  ([x <- :wat::core::i64] -> :wat::core::i64 x)
  ([x <- :wat::core::f64] -> :wat::core::f64 x))
(:wat::core::defn :user::probe-06 [] -> :wat::core::i64 (:p06::process 42))

;; Probe 9 — runtime computed result correct: n*n → 49
(:wat::core::defclause :p09::factorial-like
  ([n <- :wat::core::i64] -> :wat::core::i64
    (:wat::i64::* n n)))
(:wat::core::defn :user::probe-09 [] -> :wat::core::i64 (:p09::factorial-like 7))

;; Probe 10 — single-clause defclause equivalent to defn → 42
(:wat::core::defclause :p10::double
  ([n <- :wat::core::i64] -> :wat::core::i64
    (:wat::i64::* n 2)))
(:wat::core::defn :user::probe-10 [] -> :wat::core::i64 (:p10::double 21))
