;; tests/function/probe_arc237_stone3_guard_ensure.wat
;; Arc 237 Stone 237.3 — :guard + :ensure clause-keywords (14 probes).
;; Co-located fixture, slurped via startup_beside(file!()).
;; Startup-fail negative cases are in sibling *.wat.bad files.
;; Runtime-error cases (probes 2, 7) are included here — startup succeeds, eval fails.

;; Probe 1 — :guard true → body fires → 42
(:wat::core::defclause :p01::pick
  ([x <- :wat::core::i64] :guard (:wat::i64::> x 0) -> :wat::core::i64 x))
(:wat::core::defn :user::probe-01 [] -> :wat::core::i64 (:p01::pick 42))

;; Probe 2 — :guard false → NoMatchingClause at RUNTIME (startup succeeds)
(:wat::core::defclause :p02::pick
  ([x <- :wat::core::i64] :guard (:wat::i64::> x 0) -> :wat::core::i64 x))
(:wat::core::defn :user::probe-02-err [] -> :wat::core::i64 (:p02::pick -5))

;; Probe 3 — first :guard false falls through to second :guard true → 42
(:wat::core::defclause :p03::pick
  ([x <- :wat::core::i64] :guard (:wat::i64::> x 100) -> :wat::core::i64 999)
  ([x <- :wat::core::i64] :guard (:wat::i64::> x 0) -> :wat::core::i64 x))
(:wat::core::defn :user::probe-03 [] -> :wat::core::i64 (:p03::pick 42))

;; Probe 4 — factorial via guards (5! = 120)
(:wat::core::defclause :p04::factorial
  ([n <- :wat::core::i64] :guard (:wat::core::= n 0) -> :wat::core::i64 1)
  ([n <- :wat::core::i64] :guard (:wat::i64::> n 0) -> :wat::core::i64
    (:wat::i64::* n (:p04::factorial (:wat::i64::- n 1)))))
(:wat::core::defn :user::probe-04 [] -> :wat::core::i64 (:p04::factorial 5))

;; Probe 6 — :ensure true → result returned → 42
(:wat::core::defclause :p06::positive
  ([x <- :wat::core::i64] -> :wat::core::i64
    :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
              (:wat::i64::> result 0))
    x))
(:wat::core::defn :user::probe-06 [] -> :wat::core::i64 (:p06::positive 42))

;; Probe 7 — :ensure false → postcondition error at RUNTIME (startup succeeds)
(:wat::core::defclause :p07::positive
  ([x <- :wat::core::i64] -> :wat::core::i64
    :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
              (:wat::i64::> result 0))
    x))
(:wat::core::defn :user::probe-07-err [] -> :wat::core::i64 (:p07::positive -5))

;; Probe 11 — both :guard AND :ensure in one clause → 42
(:wat::core::defclause :p11::strict-positive
  ([x <- :wat::core::i64]
    :guard (:wat::i64::> x 0)
    :ensure (:wat::core::fn [result <- :wat::core::i64] -> :wat::core::bool
              (:wat::i64::> result 0))
    -> :wat::core::i64 x))
(:wat::core::defn :user::probe-11 [] -> :wat::core::i64 (:p11::strict-positive 42))

;; Probe 14 — complex demo: 2 same-arity guards + 3-arity with ensure → "result: sum=6"
(:wat::core::defclause :p14::process
  ([x <- :wat::core::i64 y <- :wat::core::i64]
    :guard (:wat::i64::> x y)
    -> :wat::core::String
    (:wat::string::concat "x>y:" (:wat::core::i64/to-string x)))
  ([x <- :wat::core::i64 y <- :wat::core::i64]
    :guard (:wat::i64::< x y)
    -> :wat::core::String
    (:wat::string::concat "x<y:" (:wat::core::i64/to-string y)))
  ([x <- :wat::core::i64 y <- :wat::core::i64 z <- :wat::core::i64]
    :ensure (:wat::core::fn [result <- :wat::core::String] -> :wat::core::bool
              (:wat::string::starts-with? result "result:"))
    -> :wat::core::String
    (:wat::string::concat "result: sum="
      (:wat::core::i64/to-string
        (:wat::i64::+ (:wat::i64::+ x y) z)))))
(:wat::core::defn :user::probe-14 [] -> :wat::core::String (:p14::process 1 2 3))
