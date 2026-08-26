;; tests/wat_lang/wat_arc168_let_flat_shape.wat — co-located fixture.
;; Covers positive tests for arc 168: flat-shape vector bindings + implicit-do body.
;; Negative tests use separate *.wat.bad files.

; test 1: single binding [x 1]
(:wat::core::defn :t::test1-single [] -> :wat::core::i64
  (:wat::core::let [x 1] (:wat::i64::+ x 1)))

; test 2: multiple bindings [x 1 y 2]
(:wat::core::defn :t::test2-multi [] -> :wat::core::i64
  (:wat::core::let [x 1 y 2] (:wat::i64::+ x y)))

; test 3: sequential references — later RHS sees earlier names
(:wat::core::defn :t::test3-seq [] -> :wat::core::i64
  (:wat::core::let [x 1 y (:wat::i64::+ x 1)] y))

; test 4: empty bindings — [] legal
(:wat::core::defn :t::test4-empty [] -> :wat::core::i64
  (:wat::core::let [] (:wat::i64::+ 1 1)))

; test 5: empty body — (let [x 1]) returns nil
(:wat::core::defn :t::test5-empty-body [] -> :wat::core::nil
  (:wat::core::let [x 1]))

; test 6: destructure binding [[a b] (Tuple ...)]
(:wat::core::defn :t::test6-destructure [] -> :wat::core::i64
  (:wat::core::let [[a b] (:wat::core::Tuple 3 4)]
    (:wat::i64::+ a b)))

; test 10: multi-form let body — non-finals for side effect
(:wat::core::defn :t::test10-multi-body [] -> :wat::core::i64
  (:wat::core::let [x 1]
    (:wat::i64::+ x 99)
    (:wat::i64::+ x 50)
    (:wat::i64::+ x 41)))

; test 12: multi-form fn body
(:wat::core::defn :t::test12-fn-body [] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::i64::+ x 99)
     (:wat::i64::+ x 50)
     (:wat::i64::+ x 41))
   1))

; test 13: multi-form defn body — defn forwards N body forms
(:wat::core::defn :t::triple-body13
  [x <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::i64::+ x 99)
  (:wat::i64::+ x 50)
  (:wat::i64::+ x 41))
(:wat::core::defn :t::test13-defn-body [] -> :wat::core::i64 (:t::triple-body13 1))

; test 14: single body let regression
(:wat::core::defn :t::test14-single-let [] -> :wat::core::i64
  (:wat::core::let [x 10 y 20] (:wat::i64::+ x y)))

; test 15: single body fn regression
(:wat::core::defn :t::test15-single-fn [] -> :wat::core::i64
  ((:wat::core::fn [x <- :wat::core::i64 y <- :wat::core::i64]
     -> :wat::core::i64
     (:wat::i64::+ x y))
   7 8))
