;; probe-what-a-closure-costs-in-a-wide-scope.wat
;;
;; probe-what-a-closure-costs.wat measured ~1.6us for two closures and declared
;; the hypothesis dead. That probe was FLAWED: its closures captured NOTHING and
;; sat in a two-binding scope. sqs.wat's `nap`/`once-put` capture `rows` and
;; `store` inside a deep, wide let.
;;
;; If closure construction copies the enclosing environment, the cost scales with
;; how much is in scope — and the narrow probe cannot see it.
;;
;; Refutation: if wide and narrow cost the same, capture is by reference and the
;; hypothesis is dead for good.

(:wat::config::set-redef! true)

(:wat::core::defn :wc::now [] -> :wat::core::i64
  (:wat::time::epoch-nanos (:wat::time::now)))

;; NARROW — two captureless closures, nothing else in scope.
(:wat::core::defn :wc::narrow [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [nap  (:wat::core::fn [] -> :wat::core::nil nil)
     once (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool (:wat::i64::> x 0))]
    (:wat::i64::+ acc 1)))

;; WIDE — the same two closures, but 16 live bindings in scope and both closures
;; CAPTURE from it. This is the sqs.wat shape.
(:wat::core::defn :wc::wide [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [b01 (:wat::i64::+ i 1)  b02 (:wat::i64::+ i 2)  b03 (:wat::i64::+ i 3)
     b04 (:wat::i64::+ i 4)  b05 (:wat::i64::+ i 5)  b06 (:wat::i64::+ i 6)
     b07 (:wat::i64::+ i 7)  b08 (:wat::i64::+ i 8)  b09 (:wat::i64::+ i 9)
     b10 (:wat::core::range 0 8)
     b11 (:wat::core::Vector :- [:wat::core::String] "a" "b" "c")
     b12 (:wat::core::format "{a}" :a i)
     b13 (:wat::i64::+ i 13) b14 (:wat::i64::+ i 14) b15 (:wat::i64::+ i 15)
     b16 (:wat::i64::+ i 16)
     nap  (:wat::core::fn [] -> :wat::core::nil nil)
     once (:wat::core::fn [x <- :wat::core::i64] -> :wat::core::bool
            (:wat::i64::> (:wat::i64::+ x b01) (:wat::core::count b11)))]
    (:wat::i64::+ acc b16)))

;; WIDE-NO-CLOSURE — identical bindings, no closures. Isolates the closures.
(:wat::core::defn :wc::wide-bare [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [b01 (:wat::i64::+ i 1)  b02 (:wat::i64::+ i 2)  b03 (:wat::i64::+ i 3)
     b04 (:wat::i64::+ i 4)  b05 (:wat::i64::+ i 5)  b06 (:wat::i64::+ i 6)
     b07 (:wat::i64::+ i 7)  b08 (:wat::i64::+ i 8)  b09 (:wat::i64::+ i 9)
     b10 (:wat::core::range 0 8)
     b11 (:wat::core::Vector :- [:wat::core::String] "a" "b" "c")
     b12 (:wat::core::format "{a}" :a i)
     b13 (:wat::i64::+ i 13) b14 (:wat::i64::+ i 14) b15 (:wat::i64::+ i 15)
     b16 (:wat::i64::+ i 16)]
    (:wat::i64::+ acc b16)))

(:wat::core::defn :wc::run [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::range 0 8000)
     n  (:wat::core::count xs)
     t0 (:wc::now)  _a (:wat::core::foldl :wc::narrow    0 xs)
     t1 (:wc::now)  _b (:wat::core::foldl :wc::wide      0 xs)
     t2 (:wc::now)  _c (:wat::core::foldl :wc::wide-bare 0 xs)
     t3 (:wc::now)]
    (:wat::core::format
      "n={n};narrow-us={a};wide-us={b};wide-bare-us={c};closure-in-wide-ns/op={d}"
      :n n
      :a (:wat::i64::/ (:wat::i64::- t1 t0) 1000)
      :b (:wat::i64::/ (:wat::i64::- t2 t1) 1000)
      :c (:wat::i64::/ (:wat::i64::- t3 t2) 1000)
      :d (:wat::i64::/ (:wat::i64::- (:wat::i64::- t2 t1) (:wat::i64::- t3 t2)) n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wc::run)))
