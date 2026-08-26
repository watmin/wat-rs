;; tests/wat_lang/wat_arc154_kill_let_star.wat — co-located fixture for the sibling probe (.rs).
;; Covers positive (startup-ok) tests: sequential let semantics, tail-call, nested,
;; fn body, empty bindings, walker narrowness, reflection.
;; Negative tests use separate *.wat.bad files via startup_from_file.

; test 1: sequential let — b references a
(:wat::core::defn :t::compute1 [] -> :wat::core::i64
  (:wat::core::let [a 5 b (:wat::i64::+ a 1)] b))

; test 4: let in tail position (tail-call countdown)
(:wat::core::defn :t::countdown4 [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0) 
    n
    (:wat::core::let [m (:wat::i64::- n 1)]
      (:t::countdown4 m))))

; test 5: nested lets — outer visible to inner
(:wat::core::defn :t::nested5 [] -> :wat::core::i64
  (:wat::core::let [a 10]
    (:wat::core::let [b (:wat::i64::+ a 5)] b)))

; test 6: fn body with let — sequential inside fn
(:wat::core::defn :t::add5-6 [x <- :wat::core::i64] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64] -> :wat::core::i64
     (:wat::core::let [a x b (:wat::i64::+ a 5)] b))
   x))

; test 7: empty bindings — (let [] body) is legal
(:wat::core::defn :t::empty-let7 [] -> :wat::core::i64
  (:wat::core::let [] 42))

; test 8: walker narrowness — do + multiple lets, no let*
(:wat::core::defn :t::multi-let8 [] -> :wat::core::i64
  (:wat::core::do
    (:wat::core::let [x 1] x)
    (:wat::core::let [y 2] y)))

; test 10: sequential let as reflection probe
(:wat::core::defn :t::lookup-probe10 [] -> :wat::core::i64
  (:wat::core::let [a 1 b (:wat::i64::+ a 2)] b))
