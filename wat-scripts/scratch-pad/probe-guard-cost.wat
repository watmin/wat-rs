;; probe-guard-cost.wat — what does the serve loop's request size guard cost per hop?
;;
;; wat/service.wat:1946 — the SERVER-side guarded-arm is UNCONDITIONAL:
;;     (:wat::core::let [n (:wat::string::length (:wat::edn::write req))] (if (> n cap) ...))
;; Every dispatched request is serialised to EDN purely to measure its length, at BOTH loci.
;; (The CLIENT-side twin at :2299 is gated by `peer-wire?`, so thread-tier pays zero there.)
;;
;; A do-nothing hop measures 143us thread / 183us process (probe-hop-cost.wat). This asks how
;; much of that is the guard, by timing the guard's exact expression on the exact record shape.

(:wat::core::defrecord :probe::PingRequest [])
(:wat::core::defrecord :probe::FatRequest [a <- :wat::core::String  b <- :wat::core::i64])

(:wat::core::defn :probe::guard-empty-ns [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::i64  _i <- :wat::core::i64] -> :wat::core::i64
            (:wat::i64::+ acc (:wat::string::length (:wat::edn::write (:probe::PingRequest)))))
          0
          (:wat::core::range 0 n))
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) n)))

(:wat::core::defn :probe::guard-fat-ns [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
            (:wat::i64::+ acc
              (:wat::string::length
                (:wat::edn::write (:probe::FatRequest :a "a modest message body" :b i)))))
          0
          (:wat::core::range 0 n))
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) n)))

;; The control: the same loop doing arithmetic only, so the foldl/closure overhead is subtracted.
(:wat::core::defn :probe::loop-floor-ns [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::let
    [t0 (:wat::time::epoch-nanos (:wat::time::now))
     _  (:wat::core::foldl
          (:wat::core::fn [acc <- :wat::core::i64  i <- :wat::core::i64] -> :wat::core::i64
            (:wat::i64::+ acc i))
          0
          (:wat::core::range 0 n))
     t1 (:wat::time::epoch-nanos (:wat::time::now))]
    (:wat::i64::/ (:wat::i64::- t1 t0) n)))

(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::format "floor={f} empty={e} fat={g}"
    :f (:probe::loop-floor-ns 200) :e (:probe::guard-empty-ns 200) :g (:probe::guard-fat-ns 200)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::format "per-call ns:  loop-floor={f}  guard(empty req)={e}  guard(2-field req)={g}"
      :f (:probe::loop-floor-ns 20000)
      :e (:probe::guard-empty-ns 20000)
      :g (:probe::guard-fat-ns 20000))))
