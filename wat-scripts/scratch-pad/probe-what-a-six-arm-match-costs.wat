;; probe-what-a-six-arm-match-costs.wat
;;
;; V1 (closures off happy path) and V2 (body in `if`, not `let`) left ~600–800 ms
;; on publish. Both still run a 6-arm PutResponse match every send. The old
;; send was 2 arms (Success / _). If a 6-arm match on the real store type is
;; ~80 µs, 8000 sends is the 934 ms.

(:wat::config::set-redef! true)

(:wat::core::defn :sa::now [] -> :wat::core::i64
  (:wat::time::epoch-nanos (:wat::time::now)))

(:wat::core::defenum :sa::R :wat::enum::Pure
  :Success []
  :Transient []
  :Constraint []
  :Fatal []
  :TooLarge []
  :Malformed [])

(:wat::core::defn :sa::six [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match (:sa::R::Success)
    ((:sa::R::Success) (:wat::i64::+ acc 1))
    ((:sa::R::Transient) acc)
    ((:sa::R::Constraint) acc)
    ((:sa::R::Fatal) acc)
    ((:sa::R::TooLarge) acc)
    ((:sa::R::Malformed) acc)))

(:wat::core::defn :sa::two [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match (:sa::R::Success)
    ((:sa::R::Success) (:wat::i64::+ acc 1))
    (_ acc)))

;; Same 6 arms, but each non-Success is an assertion (sqs.wat's named-death shape).
(:wat::core::defn :sa::six-assert [acc <- :wat::core::i64 i <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::match (:sa::R::Success)
    ((:sa::R::Success) (:wat::i64::+ acc 1))
    ((:sa::R::Transient)
      (:wat::kernel::assertion-failed! "sa Transient" :wat::core::None :wat::core::None))
    ((:sa::R::Constraint)
      (:wat::kernel::assertion-failed! "sa Constraint" :wat::core::None :wat::core::None))
    ((:sa::R::Fatal)
      (:wat::kernel::assertion-failed! "sa Fatal" :wat::core::None :wat::core::None))
    ((:sa::R::TooLarge)
      (:wat::kernel::assertion-failed! "sa TooLarge" :wat::core::None :wat::core::None))
    ((:sa::R::Malformed)
      (:wat::kernel::assertion-failed! "sa Malformed" :wat::core::None :wat::core::None))))

(:wat::core::defn :sa::run [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::range 0 8000)
     n  (:wat::core::count xs)
     t0 (:sa::now)  _a (:wat::core::foldl :sa::two        0 xs)
     t1 (:sa::now)  _b (:wat::core::foldl :sa::six        0 xs)
     t2 (:sa::now)  _c (:wat::core::foldl :sa::six-assert 0 xs)
     t3 (:sa::now)]
    (:wat::core::format
      "n={n};two-us={a};six-us={b};six-assert-us={c};two-ns/op={ta};six-ns/op={tb};six-assert-ns/op={tc};six-minus-two-ns/op={d}"
      :n n
      :a (:wat::i64::/ (:wat::i64::- t1 t0) 1000)
      :b (:wat::i64::/ (:wat::i64::- t2 t1) 1000)
      :c (:wat::i64::/ (:wat::i64::- t3 t2) 1000)
      :ta (:wat::i64::/ (:wat::i64::- t1 t0) n)
      :tb (:wat::i64::/ (:wat::i64::- t2 t1) n)
      :tc (:wat::i64::/ (:wat::i64::- t3 t2) n)
      :d (:wat::i64::/ (:wat::i64::- (:wat::i64::- t2 t1) (:wat::i64::- t1 t0)) n))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:sa::run)))
