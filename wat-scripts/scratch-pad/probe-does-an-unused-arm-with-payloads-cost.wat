;; probe-does-an-unused-arm-with-payloads-cost.wat
;;
;; Sibling to probe-does-an-unused-arm-cost.wat, closing its one gap: that probe
;; and grok's six-arm probe BOTH use a Pure enum whose arms bind nothing, while
;; sqs.wat's arms bind payloads (`(PutResponse::Transient _e)`,
;; `(RequestMalformed _p _e _g)`).
;;
;; If the interpreter does work per unused arm proportional to its PATTERN rather
;; than its body, a payload-binding probe is the only one that can see it.
;;
;; ONE VARIABLE: the taken arm is identical; the unused arms are payload-binding
;; and large in one function, payload-binding and tiny in the other.

(:wat::config::set-redef! true)

(:wat::core::defn :up::now [] -> :wat::core::i64
  (:wat::time::epoch-nanos (:wat::time::now)))

(:wat::core::defenum :up::R :wat::enum::Pure
  :Success []
  :Transient  [err <- :wat::core::String]
  :Constraint [err <- :wat::core::String]
  :Fatal      [err <- :wat::core::String]
  :TooLarge   [bytes <- :wat::core::i64  cap <- :wat::core::i64]
  :Malformed  [path <- :wat::core::String  expected <- :wat::core::String  got <- :wat::core::String])

(:wat::core::defn :up::tiny [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match (:up::R::Success)
    ((:up::R::Success) (:wat::i64::+ acc 1))
    ((:up::R::Transient _a) acc)
    ((:up::R::Constraint _a) acc)
    ((:up::R::Fatal _a) acc)
    ((:up::R::TooLarge _a _b) acc)
    ((:up::R::Malformed _a _b _c) acc)))

(:wat::core::defn :up::huge [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match (:up::R::Success)
    ((:up::R::Success) (:wat::i64::+ acc 1))
    ((:up::R::Transient a)
      (:wat::core::let
        [x1 (:wat::core::count a) x2 (:wat::i64::+ x1 i)
         x3 (:wat::core::format "{p}-{q}" :p a :q x2)
         x4 (:wat::core::Vector :- [:wat::core::String] a x3 "z")
         x5 (:wat::core::foldl :up::add 0 (:wat::core::range 0 6))]
        (:wat::i64::+ acc (:wat::i64::+ x5 (:wat::core::count x4)))))
    ((:up::R::Constraint a)
      (:wat::core::let
        [y1 (:wat::core::count a) y2 (:wat::i64::* y1 3)
         y3 (:wat::core::format "{p}/{q}" :p a :q y2)
         y4 (:wat::core::Vector :- [:wat::core::String] y3 a)
         y5 (:wat::core::foldl :up::add 0 (:wat::core::range 0 9))]
        (:wat::i64::+ acc (:wat::i64::+ y5 (:wat::core::count y4)))))
    ((:up::R::Fatal a)
      (:wat::core::let
        [z1 (:wat::core::count a) z2 (:wat::i64::* z1 7)
         z3 (:wat::core::format "{p}|{q}" :p a :q z2)
         z4 (:wat::core::foldl :up::add 0 (:wat::core::range 0 12))]
        (:wat::i64::+ acc (:wat::i64::+ z4 (:wat::core::count z3)))))
    ((:up::R::TooLarge a b)
      (:wat::core::let
        [w1 (:wat::i64::+ a b) w2 (:wat::core::format "{p}~{q}" :p a :q b)
         w3 (:wat::core::foldl :up::add 0 (:wat::core::range 0 15))]
        (:wat::i64::+ acc (:wat::i64::+ w3 (:wat::core::count w2)))))
    ((:up::R::Malformed a b c)
      (:wat::core::let
        [v1 (:wat::core::format "{p}{q}{r}" :p a :q b :r c)
         v2 (:wat::core::Vector :- [:wat::core::String] a b c v1)
         v3 (:wat::core::foldl :up::add 0 (:wat::core::range 0 18))]
        (:wat::i64::+ acc (:wat::i64::+ v3 (:wat::core::count v2)))))))

(:wat::core::defn :up::add [acc <- :wat::core::i64 x <- :wat::core::i64] -> :wat::core::i64
  (:wat::i64::+ acc x))

(:wat::core::defn :up::run [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::range 0 8000)
     n  (:wat::core::count xs)
     t0 (:up::now)  _a (:wat::core::foldl :up::tiny 0 xs)
     t1 (:up::now)  _b (:wat::core::foldl :up::huge 0 xs)
     t2 (:up::now)  _c (:wat::core::foldl :up::tiny 0 xs)
     t3 (:up::now)
     tiny-avg (:wat::i64::/ (:wat::i64::+ (:wat::i64::- t1 t0) (:wat::i64::- t3 t2)) 2)]
    (:wat::core::format
      "n={n};tiny1-ns/op={a};huge-ns/op={b};tiny2-ns/op={c};huge-minus-tiny-ns/op={d}"
      :n n
      :a (:wat::i64::/ (:wat::i64::- t1 t0) n)
      :b (:wat::i64::/ (:wat::i64::- t2 t1) n)
      :c (:wat::i64::/ (:wat::i64::- t3 t2) n)
      :d (:wat::i64::/ (:wat::i64::- (:wat::i64::- t2 t1) tiny-avg) n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:up::run)))
